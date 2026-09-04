//! The browser leg's SDP contract, hermetically: fail-closed against a browser-shaped offer.
//!
//! **The negotiation itself is not provable here** and this file does not pretend otherwise.
//! Nothing in this repository opens an `RTCPeerConnection`, so no DTLS handshake, no ICE
//! nomination and no key installation happens in any of these cases. What *is* provable without a
//! browser is the boundary: which offers this server answers, which it refuses, and that a refusal
//! is a refusal rather than a retry with something weaker. That is `docs/specs/webrtc-audio.md`
//! §1's own rule, and it is the half a page cannot be relied on to enforce.

use std::net::{IpAddr, Ipv4Addr};

use phone_server::browser::{answer_offer, Refusal};
use phone_server::wire::{BridgeCause, TerminationReason};
use sipx_sdp::browser_audio::{validate, BrowserAudioLocal, BrowserAudioRole, ProfileError};
use sipx_sdp::fingerprint::{Fingerprint, HashFunc, SetupCapabilities};
use sipx_sdp::ice::{Candidate, Credentials};

/// A fingerprint of the right shape. Its bytes are a fixture and never checked against a
/// certificate here — the check that matters happens during the handshake, and that is exactly
/// the part no test in this repository can reach.
fn fingerprint(byte: u8) -> Fingerprint {
    Fingerprint {
        func: HashFunc::Sha256,
        digest: vec![byte; 32],
    }
}

/// What this server would put in an answer: one component, one candidate, a SHA-256 fingerprint.
fn local() -> BrowserAudioLocal {
    BrowserAudioLocal {
        address: IpAddr::V4(Ipv4Addr::new(198, 51, 100, 7)),
        port: 40_002,
        session_id: 7,
        session_version: 0,
        direction: sipx_sdp::Direction::SendRecv,
        ice: Credentials::new("Sv3T", "ourPasswordIsTwentyTwoPlus")
            .expect("credentials RFC 8839 §5.4 admits"),
        candidates: vec![
            Candidate::parse("1 1 UDP 2130706431 198.51.100.7 40002 typ host")
                .expect("a host candidate"),
        ],
        fingerprint: fingerprint(0xAB),
        setup: SetupCapabilities::both(),
    }
}

/// An offer of the shape a browser sends: one audio section on `UDP/TLS/RTP/SAVPF`, RTCP muxed on
/// component one, a SHA-256 fingerprint, `setup:actpass`, and Opus primary with PCMU, PCMA, CN and
/// telephone-event present.
fn browser_offer() -> String {
    [
        "v=0",
        "o=- 4611731400430051336 2 IN IP4 192.0.2.10",
        "s=-",
        "t=0 0",
        "a=ice-options:trickle",
        "m=audio 51234 UDP/TLS/RTP/SAVPF 111 0 8 13 101",
        "c=IN IP4 192.0.2.10",
        "a=sendrecv",
        "a=rtcp-mux",
        "a=ice-ufrag:F7gI",
        "a=ice-pwd:x9cm1YzichV2XlhiMu8gQz",
        "a=fingerprint:sha-256 \
         AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:\
         AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB",
        "a=setup:actpass",
        "a=candidate:1 1 UDP 2130706431 192.0.2.10 51234 typ host",
        "a=rtpmap:111 opus/48000/2",
        "a=rtpmap:0 PCMU/8000",
        "a=rtpmap:8 PCMA/8000",
        "a=rtpmap:13 CN/8000",
        "a=rtpmap:101 telephone-event/8000",
        "a=fmtp:101 0-16",
    ]
    .join("\r\n")
        + "\r\n"
}

/// The offer with one line removed or replaced — how a weakened offer is built for these cases.
fn offer_without(line_starts_with: &str) -> String {
    let offer = browser_offer();
    let kept: Vec<&str> = offer
        .lines()
        .filter(|line| !line.starts_with(line_starts_with))
        .collect();
    assert_ne!(
        kept.len(),
        offer.lines().count(),
        "the case removes a line the fixture actually has: {line_starts_with}"
    );
    kept.join("\r\n") + "\r\n"
}

/// The offer with one line replaced by another.
fn offer_with(line_starts_with: &str, replacement: &str) -> String {
    let offer = browser_offer();
    let mut replaced = false;
    let kept: Vec<String> = offer
        .lines()
        .map(|line| {
            if line.starts_with(line_starts_with) {
                replaced = true;
                replacement.to_owned()
            } else {
                line.to_owned()
            }
        })
        .collect();
    assert!(replaced, "the case replaces a line the fixture has");
    kept.join("\r\n") + "\r\n"
}

#[test]
fn a_browser_offer_is_answered_on_the_profile() {
    let local = local();
    let accepted =
        answer_offer(&browser_offer(), &local).expect("a browser-shaped offer is answered");
    let answer = accepted.answer;

    // The answer is one this profile would accept back, which is the only check worth making
    // here: it is `sipx_sdp::browser_audio`'s own rule and not a restatement of it.
    let parsed = sipx_sdp::parse(&answer).expect("the answer is SDP");
    let validated = validate(&parsed, BrowserAudioRole::Answerer)
        .expect("the answer crosses every browser-audio boundary");

    assert_eq!(
        validated.port, local.port,
        "the answer advertises the port this server bound"
    );
    assert_eq!(
        validated.fingerprint, local.fingerprint,
        "the answer carries this server's own certificate fingerprint"
    );
    assert_eq!(
        validated.payloads.opus, 111,
        "the offered payload numbers are preserved rather than renumbered"
    );
    assert!(
        !answer.contains("a=crypto"),
        "SDES is not on this profile; the keys come from DTLS"
    );
    assert!(
        answer.contains("a=rtcp-mux"),
        "RTP and RTCP share the one component"
    );
}

#[test]
fn a_browser_leg_negotiates_opus_which_is_why_the_bridge_transcodes() {
    // This is the measurement behind the `opus` feature in `widgets/phone/Cargo.toml`, and it is
    // here rather than in a comment because the comment would otherwise be the only evidence.
    //
    // The profile requires Opus in the vocabulary, `browser_audio::answer` preserves the *offered*
    // order, and a browser puts Opus first. So the browser leg runs Opus while the SIP leg is
    // G.711, `sipx_media::Bridge` transcodes across that pair, and `Codec::Opus` does not exist
    // without the feature. If a future offer ordering makes this PCMU, the bridge becomes
    // pass-through and the feature is arguable — and this case is where that shows up.
    let accepted = answer_offer(&browser_offer(), &local()).expect("the offer is answered");
    let (codec, payload) = accepted
        .codec()
        .expect("the agreed codec is one this build can encode");
    assert_eq!(codec, sipx_media::Codec::Opus);
    assert_eq!(
        payload, 111,
        "on the dynamic number the two sides agreed on"
    );
}

#[test]
fn every_weakened_offer_is_refused_and_none_is_answered() {
    // One case per way the profile can be missed. The point is not any single row: it is that
    // there is no row that comes back answered, because a missing capability is never permission
    // to retry with something weaker.
    let cases: Vec<(&str, String, ProfileError)> = vec![
        (
            "no DTLS fingerprint at all",
            offer_without("a=fingerprint"),
            ProfileError::FingerprintRequired,
        ),
        (
            "a SHA-1 fingerprint, which is not the profile's hash",
            offer_with(
                "a=fingerprint",
                "a=fingerprint:sha-1 AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB:AB",
            ),
            ProfileError::FingerprintRequired,
        ),
        (
            "plain RTP/AVP instead of UDP/TLS/RTP/SAVPF",
            offer_with("m=audio", "m=audio 51234 RTP/AVP 111 0 8 13 101"),
            ProfileError::WeakerMedia,
        ),
        (
            "SDES keys offered beside DTLS",
            offer_with("a=setup", "a=crypto:1 AES_CM_128_HMAC_SHA1_80 inline:abc"),
            ProfileError::WeakerMedia,
        ),
        (
            "no rtcp-mux, so RTCP would want a second component",
            offer_without("a=rtcp-mux"),
            ProfileError::RtcpMuxRequired,
        ),
        (
            "no ICE credentials",
            offer_without("a=ice-pwd"),
            ProfileError::IceRequired,
        ),
        (
            "no candidate, on a non-trickle channel where none is coming",
            offer_without("a=candidate"),
            ProfileError::IceRequired,
        ),
        (
            "no Opus, which the profile requires",
            offer_without("a=rtpmap:111"),
            ProfileError::CodecSetIncomplete,
        ),
        (
            "no telephone-event, so digits would have to go as audio",
            offer_without("a=rtpmap:101"),
            ProfileError::CodecSetIncomplete,
        ),
        (
            "no setup role, so no DTLS role can be resolved",
            offer_without("a=setup"),
            ProfileError::SetupRole,
        ),
    ];

    let local = local();
    for (what, offer, expected) in cases {
        let refusal = answer_offer(&offer, &local)
            .map(|accepted| {
                panic!("an offer with {what} was answered:\n{}", accepted.answer);
            })
            .expect_err("every weakened offer is refused");
        assert_eq!(
            refusal.source,
            Refusal::Profile(expected),
            "an offer with {what} is refused, but for the wrong stated reason"
        );
        assert_eq!(
            refusal.cause,
            BridgeCause::Refused,
            "a profile refusal is the server not having it, whatever the reason"
        );
    }
}

#[test]
fn text_that_is_not_sdp_at_all_is_refused_rather_than_ignored() {
    let refusal = answer_offer("not an offer", &local())
        .expect_err("an offer that does not parse is not an offer");
    assert_eq!(refusal.cause, BridgeCause::Refused);
    assert_eq!(refusal.source, Refusal::NotSdp);
    assert_eq!(
        refusal.reason,
        TerminationReason::ProtocolError,
        "unparseable SDP is the peer breaking the protocol, not a missing capability"
    );
}
