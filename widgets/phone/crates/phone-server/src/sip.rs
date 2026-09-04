//! The SIP leg: an ordinary `sipx_call` outbound call, and the facts it reports.
//!
//! Nothing in this module knows there is a browser. It places one call, watches it, and says what
//! happened — and each of those four things is one command
//! `softphone.control.Kernel` may issue and no other actor may. That split is the specification's,
//! not this crate's: a page asserting that the far end rang would be the fail-closed rule
//! inverted.

use std::net::IpAddr;
use std::time::Duration;

use sipx_call::{Call, CallEvent, CallEvents, DialOptions};
use sipx_sip::Uri;
use sipx_transport::{Handle, Target};
use tokio::sync::mpsc::UnboundedSender;

use crate::wire::{EndCause, Id, ToBrowser};

/// How long one keypress sounds.
///
/// RFC 4733 §2.5.1.4 puts no minimum on an event's duration, and a PBX collecting digits does:
/// 120 ms is what a person's press lasts and what every gateway is tuned for.
pub const DIGIT_DURATION: Duration = Duration::from_millis(120);

/// One fact about the far leg.
///
/// Three, because those are the commands about an outbound call that the specification grants to
/// `softphone.control.Kernel` and that this arrangement has anything to say about.
///
/// Two absences are decisions. There is no "media connected": the browser watches its own
/// `RTCPeerConnection`, so `softphone.control.MediaConnected` is the page's to issue and reporting
/// it from here would be this process claiming to see something it cannot. And a call ending is
/// [`Self::Ended`] carrying a cause rather than a hang-up, because `softphone.control.HangUp` is
/// granted to `Human` and `Agent` and to no kernel — hanging up is a local decision, and every way
/// the far leg can go away is `FailCall` with one of six classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegFact {
    /// A provisional response arrived: `softphone.control.RingCall`.
    Ringing,
    /// The 2xx and its ACK crossed: `softphone.control.ConfirmAnswer`.
    Answered,
    /// The leg is gone, in one of the six classes: `softphone.control.FailCall`.
    Ended(EndCause),
}

impl LegFact {
    /// The command this fact arrives at the page as.
    #[must_use]
    pub fn command(self, call_id: &Id) -> ToBrowser {
        match self {
            Self::Ringing => ToBrowser::RingCall {
                call_id: call_id.clone(),
            },
            Self::Answered => ToBrowser::ConfirmAnswer {
                call_id: call_id.clone(),
            },
            Self::Ended(cause) => ToBrowser::FailCall {
                call_id: call_id.clone(),
                cause,
            },
        }
    }
}

/// Where to place the call, and as whom.
#[derive(Debug, Clone)]
pub struct Destination {
    /// Whom to call.
    pub to: Uri,
    /// The transport destination of the first hop. Resolved by the caller: `sipx_call` does not
    /// resolve a Route URI or override the target it is given.
    pub via: Target,
    /// This server's own address of record.
    pub from: String,
    /// Where this server receives RTP for the SIP leg.
    pub media_address: IpAddr,
}

/// A call that could not be placed, already reported as a [`LegFact::Failed`].
#[derive(Debug, thiserror::Error)]
#[error("the SIP leg failed: {source} (reported as {cause:?})")]
pub struct LegFailed {
    /// The class the page is told.
    pub cause: EndCause,
    /// What sipx actually said.
    #[source]
    pub source: sipx_call::Error,
}

/// The SIP half of one bridge.
///
/// It owns the fact channel and not the call: a call is returned to the caller, which is what
/// lets the same `Call` be handed to [`crate::bridge`] without this type holding a lock over it.
#[derive(Debug, Clone)]
pub struct SipLeg {
    facts: UnboundedSender<LegFact>,
}

impl SipLeg {
    /// A leg that reports into this channel.
    #[must_use]
    pub fn new(facts: UnboundedSender<LegFact>) -> Self {
        Self { facts }
    }

    /// Place the call, reporting [`LegFact::Ringing`] and then [`LegFact::Answered`].
    ///
    /// # Errors
    ///
    /// [`LegFailed`], after a [`LegFact::Failed`] carrying the class the page is told.
    pub async fn place(
        &self,
        endpoint: &Handle,
        destination: &Destination,
        options: &DialOptions,
    ) -> Result<Call, LegFailed> {
        let mut dialing =
            sipx_call::dial_early(endpoint, destination.via.clone(), &destination.to, options)
                .await
                .map_err(|error| self.failed(error))?;

        // Every fact the page is told comes from sipx's own event stream rather than from
        // anything inferred here, and this is where the one consumer of it is taken. It continues
        // on the confirmed call, so the watcher started now also sees the call end — and a far end
        // that answered without ringing emits no `Ringing`, so the page is not told it rang.
        if let Some(events) = dialing.events() {
            let leg = self.clone();
            tokio::spawn(async move { leg.watch(events).await });
        }

        dialing.answered().await.map_err(|error| self.failed(error))
    }

    /// End the call from this side.
    ///
    /// It reports nothing: `hang_up` emits `CallEvent::Ended(LocalHangup)` and the watcher turns
    /// that into the fact. One producer of facts, so the page never gets two `FailCall`s for one
    /// call.
    pub async fn hang_up(&self, call: &mut Call) {
        if let Err(error) = call.hang_up().await {
            tracing::warn!(%error, "the BYE did not go cleanly; the leg is gone either way");
        }
    }

    /// Watch a placed call, reporting each fact as sipx reports it.
    ///
    /// Every other event on the stream is deliberately dropped: `PlaybackFinished`, `Hold`,
    /// `TransferRequested` and the rest are either not this arrangement's or belong to a command
    /// no kernel actor may issue. A fact this server cannot turn into one of those three commands
    /// is a fact the page has no use for.
    pub async fn watch(&self, mut events: CallEvents) {
        while let Some(event) = events.recv().await {
            match event {
                CallEvent::Ringing { .. } => self.report(LegFact::Ringing),
                CallEvent::Answered => self.report(LegFact::Answered),
                CallEvent::Ended(cause) => {
                    self.report(LegFact::Ended(end_cause(&cause)));
                    return;
                }
                _ => {}
            }
        }
    }

    /// Report one fact, or note that nobody is listening any more.
    fn report(&self, fact: LegFact) {
        if self.facts.send(fact).is_err() {
            tracing::debug!(?fact, "the control channel is gone; nobody is told");
        }
    }

    /// Classify a dial failure, report it, and hand it back.
    fn failed(&self, error: sipx_call::Error) -> LegFailed {
        let cause = failure_cause(&error);
        self.report(LegFact::Ended(cause));
        LegFailed {
            cause,
            source: error,
        }
    }

    /// Send digits on the SIP leg.
    ///
    /// Through the media session's signal path as RFC 4733 telephone events, never as audio — the
    /// control layer states the intent and `softphone.media.SendSignal` is what carries it.
    pub async fn send_digits(&self, call: &Call, digits: &str, per_digit: Duration) -> bool {
        call.send_digits(digits, per_digit).await
    }

    /// Options for one outbound call: G.711, no ICE, the plain SIP leg.
    #[must_use]
    pub fn options(destination: &Destination) -> DialOptions {
        DialOptions::new(destination.from.clone(), destination.media_address)
    }
}

/// The class the page is told when a call that was up ends.
///
/// `sipx_call::EndCause` is `#[non_exhaustive]`, so every variant it declares today is named here
/// and the wildcard says what an unknown one becomes. `RemoteCancel` and `RemoteBye` collapse to
/// one class on purpose: the specification's `EndCause` has six words and neither "the far end
/// gave up before answering" nor "the far end hung up afterwards" is more than `Remote`.
#[must_use]
pub fn end_cause(cause: &sipx_call::EndCause) -> EndCause {
    match cause {
        sipx_call::EndCause::LocalHangup => EndCause::Local,
        sipx_call::EndCause::RemoteBye | sipx_call::EndCause::RemoteCancel => EndCause::Remote,
        sipx_call::EndCause::Rejected { .. } => EndCause::Refused,
        sipx_call::EndCause::Timeout => EndCause::Timeout,
        // A cause sipx adds later is signalling's until somebody classifies it. `Sip` rather than
        // a silent `Remote`, because guessing whose fault it was is the one thing a log cannot
        // recover from.
        _ => EndCause::Sip,
    }
}

/// The class the page is told when a call could not be placed.
///
/// `sipx_call::Error` is `#[non_exhaustive]` and has some forty variants; the ones a `dial_early`
/// on this path can actually produce are named, and the wildcard is `Sip` for the same reason.
#[must_use]
pub fn failure_cause(error: &sipx_call::Error) -> EndCause {
    match error {
        // The far end said no, with a status.
        sipx_call::Error::Rejected { .. } => EndCause::Refused,
        // Nothing came back, or our own answer deadline expired. `place` never cancels an
        // invitation itself — it does not use `dial_until` — so a cancellation on this path is
        // that deadline and nothing else.
        sipx_call::Error::NoResponse
        | sipx_call::Error::Cancelled(_)
        | sipx_call::Error::InvitationCancelled
        | sipx_call::Error::SessionExpired
        | sipx_call::Error::SignallingTeardownTimeout(_) => EndCause::Timeout,
        // The media path could not be set up or agreed: a port, a codec, a key, an address.
        sipx_call::Error::Media(_)
        | sipx_call::Error::NoCommonCodec
        | sipx_call::Error::MediaRangeCollision { .. }
        | sipx_call::Error::UnspecifiedMediaAddress
        | sipx_call::Error::Sdp(_)
        | sipx_call::Error::Profile(_)
        | sipx_call::Error::Relay(_)
        | sipx_call::Error::Dtls(_)
        | sipx_call::Error::DtlsUnavailable
        | sipx_call::Error::DtlsSetup(_)
        | sipx_call::Error::CallAudioProcessing(_) => EndCause::Media,
        // Everything else is signalling: a transport, a malformed response, an authentication
        // challenge nobody answered, a dialog that never formed.
        _ => EndCause::Sip,
    }
}
