//! One phone: one control channel, one browser leg, one SIP leg, one bridge.
//!
//! **This is the untested half of the crate, and it is worth saying so here rather than only in a
//! report.** Each seam below it is exercised by `tests/` — the profile boundary, the SIP leg's
//! facts, the forwarding — but bringing a browser leg up needs a browser: a DTLS peer that presents
//! the certificate whose fingerprint the offer named. `story:browser-media-adapter` owns the page
//! half, and the first thing that will exercise this file is a real page.
//!
//! It holds no phone state, which is the ADR's requirement rather than a simplification. There is
//! no call table here, no registration and no event log: the page ran `softphone.control.Dial` and
//! `softphone.bridge.ConnectBridge` before it said anything, so every identifier is one the page
//! chose, and every fact leaves as a command the page's own system already accepts.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use sipx_media::dtls::openssl::Identity;
use sipx_sdp::ice::Credentials;
use sipx_transport::{Handle, Target, TransportKind};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::bridge::Bridged;
use crate::browser::{answer_offer, BridgeRefused, BrowserLeg};
use crate::sip::{Destination, LegFact, LegFailed, SipLeg, DIGIT_DURATION};
use crate::wire::{BridgeCause, FromBrowser, Id, TerminationReason, ToBrowser};

/// Where this server is, and who it calls through.
#[derive(Debug, Clone)]
pub struct Config {
    /// Where this server's SIP signalling binds.
    pub sip_bind: SocketAddr,
    /// The first SIP hop every call goes to — a PBX or a proxy.
    ///
    /// Configured rather than read out of what the page sent: the page names *who* to call and
    /// this server knows *where*, which is the division that keeps a deployment fact out of a
    /// browser.
    pub sip_proxy: SocketAddr,
    /// This server's own address of record.
    pub from: String,
    /// The address this server advertises for RTP, on both legs.
    pub media_address: IpAddr,
    /// The interface media sockets bind on.
    pub media_bind: IpAddr,
}

/// A bridge that did not come up, in the classes the page is told apart.
///
/// Every variant carries what `softphone.bridge.FailBridge` needs, which is why the enumeration is
/// here rather than one opaque error: a page that is told only "it failed" cannot tell a refused
/// offer from a far end that never answered, and those are different things to show a person.
#[derive(Debug, thiserror::Error)]
pub enum OpenFailed {
    /// The offer, or the handshake against it, did not meet the profile.
    #[error(transparent)]
    Browser(#[from] BridgeRefused),
    /// The far leg never came up.
    #[error(transparent)]
    FarLeg(#[from] LegFailed),
    /// This server could not bind a media port for the browser leg.
    #[error("no media port for the browser leg")]
    NoMediaPort(#[source] std::io::Error),
    /// This server could not produce a DTLS identity.
    #[error("no DTLS identity for the browser leg")]
    NoIdentity,
    /// What the page asked to dial is not a URI.
    #[error("`{0}` is not a URI this server can dial")]
    NotDialable(String),
}

impl OpenFailed {
    /// The `softphone.bridge.FailBridge` this failure reaches the page as.
    #[must_use]
    pub fn fail_bridge(&self, bridge_id: &Id, session_id: &Id) -> ToBrowser {
        let (cause, reason) = match self {
            Self::Browser(refused) => (refused.cause, refused.reason),
            // The far end, and not this server, is why there is nothing to bridge to.
            Self::FarLeg(_) => (BridgeCause::Remote, TerminationReason::Cancelled),
            Self::NoMediaPort(_) => (BridgeCause::Refused, TerminationReason::MediaOverload),
            Self::NoIdentity => (BridgeCause::Refused, TerminationReason::AuthorityRevoked),
            Self::NotDialable(_) => (BridgeCause::Refused, TerminationReason::ProtocolError),
        };
        ToBrowser::FailBridge {
            bridge_id: bridge_id.clone(),
            session_id: session_id.clone(),
            cause,
            reason,
        }
    }
}

/// Everything holding one bridged call up.
///
/// Dropping it stops the audio: the forwarding goes with `bridge`, and the browser leg's session
/// with it.
#[derive(Debug)]
pub struct Live {
    /// The call the SIP leg placed.
    pub call: sipx_call::Call,
    /// The forwarding between the two legs.
    pub bridge: Bridged,
}

/// One phone over one signalling endpoint.
#[derive(Debug)]
pub struct Phone {
    config: Config,
    signalling: Handle,
}

impl Phone {
    /// A phone over this signalling endpoint.
    #[must_use]
    pub const fn new(config: Config, signalling: Handle) -> Self {
        Self { config, signalling }
    }

    /// Bring one bridge up: answer the offer, place the call, forward the audio.
    ///
    /// The order is the specification's. The answer goes out before the call is placed, because
    /// `softphone.bridge.ConfirmBridge` is what moves a bridge to `Live` and a page whose bridge is
    /// still `Connecting` has nowhere to put a ringing call.
    ///
    /// # Errors
    ///
    /// [`OpenFailed`], which the caller turns into one `softphone.bridge.FailBridge`. Nothing here
    /// answers a weaker offer, and nothing places a call for a bridge that was refused: a refused
    /// bridge has no far leg to cancel.
    pub async fn open(
        &self,
        bridge_id: &Id,
        session_id: &Id,
        destination: &str,
        offer: &str,
        say: &impl Fn(ToBrowser),
    ) -> Result<(Live, UnboundedReceiver<LegFact>), OpenFailed> {
        let identity = Identity::generate().map_err(|_| OpenFailed::NoIdentity)?;
        let leg = BrowserLeg::bind(SocketAddr::new(self.config.media_bind, 0), identity)
            .await
            .map_err(OpenFailed::NoMediaPort)?;

        let to = sipx_sip::Uri::parse(bytes::Bytes::from(destination.to_owned()))
            .map_err(|_| OpenFailed::NotDialable(destination.to_owned()))?;

        let (local, ice) = leg
            .local(self.config.media_address, ice_credentials(), 1)
            .await?;
        let accepted = answer_offer(offer, &local)?;
        say(ToBrowser::ConfirmBridge {
            bridge_id: bridge_id.clone(),
            session_id: session_id.clone(),
            answer: accepted.answer.clone(),
        });

        let (codec, payload) = accepted.codec()?;
        let browser = Arc::new(
            leg.start(
                accepted.remote_addr(),
                codec,
                payload,
                ice,
                accepted.remote.fingerprint.clone(),
                accepted.role,
            )
            .await?,
        );

        let (facts, reports) = unbounded_channel();
        let destination = Destination {
            to,
            via: Target::new(self.config.sip_proxy, TransportKind::Udp),
            from: self.config.from.clone(),
            media_address: self.config.media_address,
        };
        let options = SipLeg::options(&destination);
        let call = SipLeg::new(facts)
            .place(&self.signalling, &destination, &options)
            .await?;

        let bridge = Bridged::connect(browser, &call);
        // The fact receiver comes back beside the bridge rather than inside it: there is exactly
        // one consumer of it, and handing it to the caller is what says so.
        Ok((Live { call, bridge }, reports))
    }
}

/// Turn each fact the SIP leg reports into the command the page runs.
///
/// One place, so the mapping from a leg's facts to the specification's commands exists once.
pub async fn relay(call_id: Id, mut reports: UnboundedReceiver<LegFact>, say: impl Fn(ToBrowser)) {
    while let Some(fact) = reports.recv().await {
        say(fact.command(&call_id));
    }
}

/// What the page asked for, applied to a live bridge.
///
/// `OpenBridge` is absent on purpose: it is the one message that creates everything, and it belongs
/// to [`Phone::open`]. Hanging up reports nothing here either — the call emits its own `Ended`, and
/// [`relay`] is the one producer of what the page is told.
pub async fn apply(message: &FromBrowser, live: &mut Live) {
    match message {
        FromBrowser::OpenBridge { .. } => {
            tracing::warn!("a second offer on a channel that already holds a bridge; ignored");
        }
        FromBrowser::Hangup { .. } => {
            if let Err(error) = live.call.hang_up().await {
                tracing::warn!(%error, "the BYE did not go cleanly; the leg is gone either way");
            }
        }
        FromBrowser::Digits { digits, .. } => {
            if !live.call.send_digits(digits, DIGIT_DURATION).await {
                tracing::warn!("the SIP leg would not carry the digits");
            }
        }
        FromBrowser::Mute { muted, .. } => {
            live.bridge.set_browser_muted(*muted);
        }
        FromBrowser::Hold { held, .. } => {
            live.bridge.set_held(*held, &live.call);
        }
    }
}

/// Fresh ICE credentials for one generation — RFC 8839 §5.4 wants them random.
///
/// Hex of a fixed length: longer than §5.4's minimum and inside the 32 characters it permits to be
/// sent, so the constructor cannot refuse them.
fn ice_credentials() -> Credentials {
    let ufrag = format!("{:08x}", rand::random::<u32>());
    let pwd = format!(
        "{:016x}{:08x}",
        rand::random::<u64>(),
        rand::random::<u32>()
    );
    Credentials::new(ufrag, pwd).expect("hex of a fixed length is within RFC 8839 §5.4")
}
