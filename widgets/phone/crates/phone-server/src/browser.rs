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

/// How long the whole browser bring-up may take before this server gives up on it.
///
/// Above [`HANDSHAKE_TIMEOUT`] on purpose, so that sipx's own budget fires first where it applies
/// and this one catches only what that budget does not cover.
///
/// It has to exist because the budget above does not bound everything.
/// `sipx_media`'s `browser::prepare` passes it into `prepare_inner` and then awaits the supervisor
/// task with no timeout of its own (`crates/sipx-media/src/browser.rs:769` in 1.1.0), so a leg that
/// never nominates an ICE pair leaves the await pending for ever. Measured 2026-09-05 against both
/// 1.0.1 and the unreleased 1.1.0 tree: a browser whose `RTCPeerConnection` went to `failed` left
/// `Phone::open` awaiting for 46 seconds and counting, with no INVITE placed, nothing logged, and a
/// media socket still bound per attempt.
///
/// Without this, that is invisible: the page has already been told `ConfirmBridge`, so its bridge
/// sits in `Live` and its call in `Requested` with nothing to move either.
/// `story:a-browser-leg-that-never-comes-up-hangs-the-phone` carries the measurement.
pub const BRING_UP: Duration = Duration::from_secs(15);

/// The **audio** sampling rate of the SIP leg, which is G.711 and therefore always 8 kHz.
///
/// `sip::SipLeg::options` builds `DialOptions::new`, whose default `MediaPolicy` is G.711 — so this
/// is not a configuration this crate could get out of step with, it is the other leg's only rate.
///
/// Audio rate and not RTP clock rate, and the difference is not academic: G.722's RTP clock is
/// 8 000 while its audio runs at 16 000 (RFC 3551 §4.5.2), so a gate written against
/// `Codec::clock_rate` alone would admit G.722 and then mis-rate it by two. The bridge moves
/// decoded *samples*, so the sample rate is the number that has to match.
pub const SIP_LEG_AUDIO_RATE: u32 = 8_000;

/// What refused the bridge.
///
/// Three, and only the middle one is a profile boundary. Text that does not parse as a session
/// description has no `m=` line to be wrong about, and calling that a `MediaSectionCount` would be
/// a diagnostic nobody could act on. And the last is not about the description at all: the
/// description is admissible and *this server* cannot carry what it agreed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    /// The offer is not a session description at all.
    #[error("the offer is not a session description")]
    NotSdp,
    /// It is one, and the browser-audio profile does not admit it.
    #[error(transparent)]
    Profile(#[from] ProfileError),
    /// The two legs agreed on codecs at different clock rates, and this server has no resampler.
    ///
    /// **Refused rather than bridged, and that is the decision this variant exists to make
    /// visible.** `sipx_media::Bridge`'s transcoding path decodes one leg and re-encodes for the
    /// other with no rate conversion (`sipx-media-1.0.1/src/bridge.rs:151-168`), so 500 ms spoken
    /// on a 48 kHz leg arrives at an 8 kHz leg as 2 880 ms — the far end hears the caller sped up
    /// six times. A bridge that mis-rates audio is worse than a call that does not start.
    ///
    /// **The way through is the page's, and it is not a weaker profile.** Measured against
    /// `sipx_sdp::browser_audio`: a browser offer whose `m=` line lists PCMU *before* Opus is
    /// answered PCMU-first, both sides resolve `selected_audio_payload` to 0, and `validate_answer`
    /// accepts the exchange — with Opus still present in the vocabulary, which is all
    /// `docs/specs/webrtc-audio.md` §1 requires of it. This server cannot make that choice for the
    /// page: the same measurement shows an answer reordered to PCMU-first against an Opus-first
    /// offer is refused by the profile's own `validate_answer` with `CodecSetIncomplete`, because
    /// the answer must preserve the offer's format order. So selection belongs to whoever authors
    /// the offer, and `story:browser-media-adapter` is where that lands.
    ///
    /// Opus with correct rate conversion is separate work, deliberately not done here.
    #[error(
        "the browser leg agreed {browser} at {browser_rate} Hz and the SIP leg is G.711 at \
         {sip_rate} Hz; this server has no resampler and `sipx_media::Bridge` does not convert \
         rates, so bridging the pair would deliver the audio at the wrong rate. Offer PCMU before \
         Opus in the `m=` line and both legs run at {sip_rate} Hz"
    )]
    NoRateConversion {
        /// What the browser leg agreed to, named rather than a payload number.
        browser: &'static str,
        /// Its RTP clock, in Hz.
        browser_rate: u32,
        /// The SIP leg's, in Hz.
        sip_rate: u32,
    },
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
    // Both sides of the ICE check, named before the verdict. `ProfileError::IceRequired` is raised
    // for our own gathered candidates and for the offer's, and the two are different faults with
    // different owners — a server that logs one word for both leaves the reader guessing, which it
    // did for an evening.
    tracing::debug!(
        local_candidates = local.candidates.len(),
        local_port = local.port,
        offered_candidates = offered
            .media
            .iter()
            .map(|media| media
                .attributes
                .iter()
                .filter(|a| a.name == "candidate")
                .count())
            .sum::<usize>(),
        "answering a browser offer"
    );
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

/// Whether this server can bridge a browser leg in this codec to its G.711 SIP leg.
///
/// One rule: the two legs must run one clock. `sipx_media::Bridge` moves audio between them
/// without rate conversion, so a pair that disagrees about the rate is a pair whose audio would
/// arrive at the wrong speed — and there is no resampler in this crate to fix it.
///
/// This is the seam, and the placement is deliberate. It is here rather than inside
/// [`answer_offer`] because the SDP is *not* what is wrong: Opus is on the profile,
/// `browser_audio::answer` produces a description the profile accepts back, and refusing to parse
/// a legal offer would put a limitation of this server into a module whose subject is the
/// specification. What this server cannot do is *run* the session, so the refusal is where a
/// session would be started — before ICE, before DTLS, before any key exists, and before the SIP
/// call is placed.
///
/// That sentence was true of this server and false of the system until correction round 2:
/// `session::Phone::open` sent `softphone.bridge.ConfirmBridge` — the answer, and the command that
/// moves the page's bridge to `Live` — before it asked this question, so the *page* started ICE and
/// DTLS against a port already on its way out. `open` now asks above the `say`, and the claim holds
/// for both ends.
///
/// # Errors
///
/// [`Refusal::NoRateConversion`], which names the pair and the way through.
pub fn bridgeable(codec: Codec) -> Result<(), BridgeRefused> {
    let rate = audio_rate(codec);
    if rate == SIP_LEG_AUDIO_RATE {
        return Ok(());
    }
    Err(BridgeRefused {
        cause: BridgeCause::Refused,
        // The nearest of the eight `softphone.media.TerminationReason` words, and none of them
        // says "this server has not implemented it". `ProtocolError` is what a page can act on —
        // the session did not start and will not on this pair — and `BridgeCause::Refused` beside
        // it is the load-bearing half: the server would not have it.
        reason: TerminationReason::ProtocolError,
        source: Refusal::NoRateConversion {
            browser: codec_name(codec),
            browser_rate: rate,
            sip_rate: SIP_LEG_AUDIO_RATE,
        },
    })
}

/// The rate of the samples a codec decodes to, which is what the bridge carries.
///
/// `Codec::clock_rate` is the RTP timestamp clock and is the wrong number here for G.722, whose
/// audio is twice it. `MediaSession::audio_rate` computes the same product on a live session; this
/// is that rule available before one exists.
#[must_use]
pub fn audio_rate(codec: Codec) -> u32 {
    codec
        .clock_rate()
        .saturating_mul(codec.samples_per_clock_unit())
}

/// A codec's own name, for a refusal a person reads.
///
/// `sipx_media::Codec` is `#[non_exhaustive]`, so every variant it declares today is named and the
/// wildcard covers one added later — which will be refused anyway unless its audio runs at
/// [`SIP_LEG_AUDIO_RATE`], and a refusal that cannot name the codec is still a refusal.
fn codec_name(codec: Codec) -> &'static str {
    match codec {
        Codec::Pcmu => "PCMU",
        Codec::Pcma => "PCMA",
        Codec::G722 => "G.722",
        Codec::L16 => "L16",
        Codec::Opus => "Opus",
        _ => "an unnamed codec",
    }
}

/// A bound browser-audio component, before ICE and DTLS have run on it.
///
/// Split from [`answer_offer`] because the two halves fail differently: the SDP boundary is pure
/// and this half owns a socket. It is *not* split because one runs before the other binds — the
/// answer has to advertise a bound port and its gathered candidates, so [`Self::bind`] and
/// [`Self::local`] both run before [`answer_offer`] can be called at all. A refusal after that
/// point leaks nothing: this type owns the `MediaPort`, and dropping the leg closes its sockets.
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
        // Before the socket does anything: a pair this server cannot bridge never becomes a
        // running session, so there is no ICE to abandon and no key to throw away.
        bridgeable(codec)?;
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
