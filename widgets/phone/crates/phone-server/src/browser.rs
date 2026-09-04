//! The browser leg: a `sipx_media` session with no SIP in it at all.
//!
//! The profile is `sipx`'s `docs/specs/webrtc-audio.md` §1 and this crate does not restate it —
//! `sipx_sdp::browser_audio` *is* it, and asking that module is how the rule stays one rule.
//! Exactly one active `audio` section, `UDP/TLS/RTP/SAVPF`, RTP and RTCP on one ICE component
//! with `a=rtcp-mux`, the fingerprint verified before any key is installed, and Opus primary with
//! PCMU, PCMA, CN and telephone-event present.
//!
//! **Fail-closed, and that is the whole design of this module.** A missing capability is never
//! permission to retry with something weaker, so every refusal leaves here as a
//! `softphone.bridge.FailBridge` and there is no path that answers an offer this profile rejects.
//!
//! What is *not* here: the page half. Nothing in this repository opens an `RTCPeerConnection`,
//! and `story:browser-media-adapter` owns that. This module takes an offer as an input.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use sipx_media::dtls::openssl::Identity;
use sipx_media::ice::Gathering;
use sipx_media::{Codec, Config, MediaPort, MediaSession};
use sipx_sdp::browser_audio::{
    validate, BrowserAudioDescription, BrowserAudioLocal, BrowserAudioRole, ProfileError,
};
use sipx_sdp::fingerprint::{Setup, SetupCapabilities};
use sipx_sdp::ice::Credentials;

use crate::wire::{BridgeCause, TerminationReason};

/// How long a DTLS handshake with a browser may take before the bridge is refused.
///
/// A budget rather than a retry count: a page whose handshake does not complete is a page that
/// will not hear anything, and waiting longer only delays telling it so.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// What refused the bridge.
///
/// Two, and the first is not a profile boundary: text that does not parse as a session description
/// has no `m=` line to be wrong about, and calling that a `MediaSectionCount` would be a diagnostic
/// nobody could act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    /// The offer is not a session description at all.
    #[error("the offer is not a session description")]
    NotSdp,
    /// It is one, and the browser-audio profile does not admit it.
    #[error(transparent)]
    Profile(#[from] ProfileError),
}

/// A bridge this server would not have, and what the page is told about it.
///
/// The two first fields are the two `softphone.bridge.FailBridge` inputs that classify a failure:
/// whose doing it was, and what the media layer records. Every refusal here is `Refused` — the
/// server would not have it — and the reason is the neutral word for *why*.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("{source} (reported as {cause:?}/{reason:?})")]
pub struct BridgeRefused {
    /// Whose doing.
    pub cause: BridgeCause,
    /// What the media layer is told.
    pub reason: TerminationReason,
    /// What refused it.
    #[source]
    pub source: Refusal,
}

/// An offer this server will bridge: the answer to send, and what the peer said.
///
/// The peer's half is kept rather than re-derived. It carries the facts the media half needs — the
/// fingerprint to verify against, the default destination, the payload number that was agreed —
/// and validating the same offer twice would leave room for the two readings to differ.
#[derive(Debug, Clone)]
pub struct Accepted {
    /// The SDP to send back, as `softphone.bridge.ConfirmBridge`'s answer.
    pub answer: String,
    /// The peer's validated description.
    pub remote: BrowserAudioDescription,
    /// This side's DTLS role, complementary to the peer's `a=setup`.
    pub role: sipx_media::dtls::Role,
}

impl Accepted {
    /// Where the peer's default destination is, as a socket address.
    ///
    /// A fact and never a nomination: ICE decides the path, and this is only where to send until
    /// it has.
    #[must_use]
    pub fn remote_addr(&self) -> SocketAddr {
        SocketAddr::new(self.remote.address, self.remote.port)
    }

    /// The codec the two sides agreed on, and the number it goes out under.
    ///
    /// # Errors
    ///
    /// [`BridgeRefused`] when the agreed payload is one this build cannot encode — which is the
    /// fail-closed rule again, and the reason `sipx-media`'s `opus` feature is not optional here.
    pub fn codec(&self) -> Result<(Codec, u8), BridgeRefused> {
        let payload = self.remote.selected_audio_payload;
        let codec = if payload == self.remote.payloads.opus {
            Codec::Opus
        } else if payload == self.remote.payloads.pcmu {
            Codec::Pcmu
        } else if payload == self.remote.payloads.pcma {
            Codec::Pcma
        } else {
            return Err(refusal_of(ProfileError::CodecSetIncomplete));
        };
        Ok((codec, payload))
    }
}

/// Answer one browser offer on the browser-audio profile, or refuse it.
///
/// There is one call to `sipx_sdp::browser_audio::answer` and no other path out of this function:
/// an offer the profile refuses produces a refusal, never a weaker answer. That is the whole
/// fail-closed rule, and it is enforced by there being nothing else here to return.
///
/// # Errors
///
/// [`BridgeRefused`] for every offer the profile does not admit.
pub fn answer_offer(offer: &str, local: &BrowserAudioLocal) -> Result<Accepted, BridgeRefused> {
    let offered = sipx_sdp::parse(offer).map_err(|_| BridgeRefused {
        cause: BridgeCause::Refused,
        reason: TerminationReason::ProtocolError,
        source: Refusal::NotSdp,
    })?;
    let answer = sipx_sdp::browser_audio::answer(&offered, local)
        .map_err(refusal_of)?
        .to_string_sdp();
    let remote = validate(&offered, BrowserAudioRole::Offerer).map_err(refusal_of)?;
    let role = match local
        .setup
        .answer_to(remote.setup)
        .map_err(|_| refusal_of(ProfileError::SetupRole))?
    {
        Setup::Active => sipx_media::dtls::Role::Client,
        Setup::Passive => sipx_media::dtls::Role::Server,
        // `answer_to` returns only those two; the rest are its error cases, and reaching here
        // would mean sipx had changed that contract rather than that this server should guess.
        _ => return Err(refusal_of(ProfileError::SetupRole)),
    };
    Ok(Accepted {
        answer,
        remote,
        role,
    })
}

/// The `softphone.bridge.FailBridge` classification for one profile refusal.
///
/// `ProfileError` is `#[non_exhaustive]`, so every variant it declares today is named here rather
/// than a few being classified and the rest falling through unnamed. The cause is always
/// `Refused`: whatever the boundary, what happened is that this server would not have the bridge.
#[must_use]
pub fn refusal_of(error: ProfileError) -> BridgeRefused {
    let reason = match error {
        // The description said something this profile does not admit. The peer broke the protocol
        // that was agreed, whichever capability it was.
        ProfileError::OpusUnavailable
        | ProfileError::InsecureSignalling
        | ProfileError::MediaSectionCount
        | ProfileError::WrongProtocol
        | ProfileError::RtcpMuxRequired
        | ProfileError::IceRequired
        | ProfileError::SetupRole
        | ProfileError::FingerprintRequired
        | ProfileError::CodecSetIncomplete
        | ProfileError::WeakerMedia
        | ProfileError::ProfileRemoved
        | ProfileError::FingerprintMismatch
        | ProfileError::NoSrtpProfile => TerminationReason::ProtocolError,
        // The description was admissible and the path never came up.
        ProfileError::NoNominatedPair | ProfileError::DtlsTimeout => {
            TerminationReason::TransportLost
        }
        ProfileError::Cancelled => TerminationReason::Cancelled,
        // A boundary sipx adds later. `ProtocolError` rather than a silent `Completed`, because a
        // bridge that was refused did not complete.
        _ => TerminationReason::ProtocolError,
    };
    BridgeRefused {
        cause: BridgeCause::Refused,
        reason,
        source: Refusal::Profile(error),
    }
}

/// A bound browser-audio component, before ICE and DTLS have run on it.
///
/// Split from [`answer_offer`] because the two halves fail differently and at different times:
/// the SDP boundary is pure and refuses before anything is bound, and this half owns a socket.
#[derive(Debug)]
pub struct BrowserLeg {
    port: MediaPort,
    identity: Identity,
}

impl BrowserLeg {
    /// Bind one component for this bridge.
    ///
    /// # Errors
    ///
    /// Whatever the operating system said about the port, unchanged: a server that cannot bind
    /// has nothing to tell the page about the profile.
    pub async fn bind(bind: SocketAddr, identity: Identity) -> std::io::Result<Self> {
        Ok(Self {
            port: MediaPort::bind(bind).await?,
            identity,
        })
    }

    /// The port the answer advertises.
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.port.local_addr()
    }

    /// The local half of the answer: this component's address, credentials, candidates and
    /// fingerprint.
    ///
    /// `gather` runs between binding and answering, which is the only window in which the socket
    /// is exclusively ours — the media loops that read it do not exist yet.
    ///
    /// # Errors
    ///
    /// [`BridgeRefused`] when the local certificate has no usable fingerprint, which is the local
    /// half of the same fail-closed rule: no fingerprint, no keys, no bridge.
    pub async fn local(
        &self,
        advertised: IpAddr,
        credentials: Credentials,
        session_id: u64,
    ) -> Result<(BrowserAudioLocal, sipx_media::ice::LocalDescription), BridgeRefused> {
        let gathering = Gathering::new(credentials, false);
        let ice = self
            .port
            .gather_with_rtcp_mode(&gathering, sipx_sdp::RtcpMode::Mux)
            .await;
        let fingerprint = self
            .identity
            .fingerprint()
            .map_err(|_| refusal_of(ProfileError::FingerprintRequired))?;
        let local = BrowserAudioLocal {
            address: advertised,
            port: self.port.local_addr().port(),
            session_id,
            // One offer, one answer, and no re-offer on this path, so the version never moves.
            session_version: 0,
            direction: sipx_sdp::Direction::SendRecv,
            ice: ice.credentials().clone(),
            candidates: ice.candidates().to_vec(),
            fingerprint,
            setup: SetupCapabilities::both(),
        };
        Ok((local, ice))
    }

    /// Run ICE and DTLS on this component and start the fail-closed browser-audio runtime.
    ///
    /// Only a verified handshake installs SRTP keys: the fingerprint from signalling is checked
    /// against the certificate the peer actually presented before a key exists.
    ///
    /// # Errors
    ///
    /// [`BridgeRefused`] for every way the handshake can fail to produce a keyed session.
    pub async fn start(
        self,
        remote: SocketAddr,
        codec: Codec,
        payload_type: u8,
        ice: sipx_media::ice::LocalDescription,
        peer_fingerprint: sipx_sdp::fingerprint::Fingerprint,
        role: sipx_media::dtls::Role,
    ) -> Result<MediaSession, BridgeRefused> {
        let mut config = Config::new(remote, codec);
        // The number the two sides agreed on, which for a dynamic codec is not the one sipx would
        // have proposed: Opus has no payload type of its own (RFC 7587 §7).
        config.payload_type = Some(payload_type);
        config.rtcp_mode = sipx_sdp::RtcpMode::Mux;
        self.port
            .start_browser_audio(
                config,
                ice,
                // One generation: this path never re-offers, so there is no second one to number.
                0,
                self.identity,
                role,
                peer_fingerprint,
                HANDSHAKE_TIMEOUT,
            )
            .await
            .map_err(|error| refusal_of(profile_error_of(&error)))
    }
}

/// The profile boundary a browser-audio start error crossed.
///
/// `BrowserStartError` is sipx's own vocabulary for the media half; the page is told the profile
/// word for it, because `softphone.bridge` names no protocol beyond SDP. Every variant it declares
/// today is named; it is `#[non_exhaustive]`, so the wildcard says what a new one becomes.
#[must_use]
pub fn profile_error_of(error: &sipx_media::browser::BrowserStartError) -> ProfileError {
    use sipx_media::browser::BrowserStartError as Start;
    match error {
        Start::RtcpMuxRequired => ProfileError::RtcpMuxRequired,
        Start::IceFailed | Start::IceStopped => ProfileError::NoNominatedPair,
        Start::DtlsTimeout => ProfileError::DtlsTimeout,
        // A handshake that completed and produced no verified key is the fingerprint check
        // failing, which is the one refusal that must never be reported as a timeout: a mismatched
        // certificate is an attacker and a slow one is a network.
        Start::Dtls(_) => ProfileError::FingerprintMismatch,
        Start::Setup(_) | Start::Component(_) | Start::Worker(_) | Start::Adapter(_) => {
            ProfileError::NoSrtpProfile
        }
        _ => ProfileError::NoSrtpProfile,
    }
}
