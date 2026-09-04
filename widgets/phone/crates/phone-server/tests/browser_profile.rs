//! The browser leg's SDP contract, hermetically: fail-closed against a browser-shaped offer.
//!
//! **The negotiation itself is not provable here** and this file does not pretend otherwise.
//! Nothing in this repository opens an `RTCPeerConnection`, so no DTLS handshake, no ICE
//! nomination and no key installation happens in any of these cases. What *is* provable without a
//! browser is the boundary: which offers this server answers, which it refuses, and that a refusal
//! is a refusal rather than a retry with something weaker. That is `docs/specs/webrtc-audio.md`
//! §1's own rule, and it is the half a page cannot be relied on to enforce.

use std::net::{IpAddr, Ipv4Addr};

use phone_server::browser::{answer_offer, bridgeable, Refusal};
use phone_server::wire::{BridgeCause, TerminationReason};
use sipx_sdp::browser_audio::{
    validate, validate_answer, BrowserAudioLocal, BrowserAudioRole, ProfileError,
};
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

/// The same offer with PCMU ahead of Opus in the `m=` line — the shape this server can bridge.
///
/// Nothing else changes: every codec the profile requires is still present with its own `rtpmap`,
/// so this is a reordering and not a weakening.
fn pcmu_first_browser_offer() -> String {
    let offer = browser_offer().replace(
        "UDP/TLS/RTP/SAVPF 111 0 8 13 101",
        "UDP/TLS/RTP/SAVPF 0 8 111 13 101",
    );
    assert!(
        offer.contains("SAVPF 0 8 111"),
        "the fixture's m= line was reordered"
    );
    offer
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
fn a_browser_offer_negotiates_opus_which_is_why_the_media_seam_refuses_it() {
    // This case carried the name `…_which_is_why_the_bridge_transcodes` and the claim that the
    // pair is transcoded. It is not: correction round 1 established that `sipx_media::Bridge`
    // transcodes *without rate conversion*, so the pair is refused instead. The measurement it was
    // written for is unchanged and is still worth having — it is what says the refusal is on the
    // ordinary path and not on an edge case.
    //
    // The profile requires Opus in the vocabulary and `browser_audio::answer` preserves the
    // *offered* order, so a browser that puts Opus first gets an Opus leg — against a G.711 SIP
    // leg, at a different clock. `browser::bridgeable` is what declines it.
    let accepted = answer_offer(&browser_offer(), &local()).expect("the offer is answered");
    let (codec, payload) = accepted
        .codec()
        .expect("the agreed codec is one this build can encode");
    assert_eq!(codec, sipx_media::Codec::Opus);
    assert_eq!(
        payload, 111,
        "on the dynamic number the two sides agreed on"
    );
    assert_eq!(
        phone_server::browser::audio_rate(codec),
        48_000,
        "and its audio runs at 48 kHz, six times the SIP leg's"
    );
    bridgeable(codec).expect_err("so the media seam will not run it");
}

/// The way through, and it is the page's to take: an offer that lists PCMU **before** Opus.
///
/// This is one of the two measurements correction round 1's branch decision rests on, and it is a
/// case rather than a sentence in a comment for that reason. Opus stays in the vocabulary, which is
/// all `docs/specs/webrtc-audio.md` §1 asks of it; what changes is which payload the exchange
/// selects, and both sides select it from the offer's order.
#[test]
fn a_pcmu_first_offer_is_the_shape_this_server_can_bridge() {
    let accepted =
        answer_offer(&pcmu_first_browser_offer(), &local()).expect("still on the profile");
    let (codec, payload) = accepted.codec().expect("PCMU is a codec every build has");
    assert_eq!(codec, sipx_media::Codec::Pcmu);
    assert_eq!(payload, 0);
    bridgeable(codec).expect("one clock on both legs, so this server bridges it");

    // The answer is one the profile accepts back, PCMU-first, with Opus still offered.
    let parsed = sipx_sdp::parse(&accepted.answer).expect("the answer is SDP");
    let checked = validate(&parsed, BrowserAudioRole::Answerer).expect("still on the profile");
    assert_eq!(checked.selected_audio_payload, 0);
    assert_eq!(
        checked.payloads.opus, 111,
        "Opus is present in the vocabulary, which is what the profile requires of it"
    );
}

/// And why this server cannot take that way through on the page's behalf.
///
/// The second measurement the branch decision rests on. An answer reordered to put PCMU first
/// against an Opus-first offer is refused by the profile's own `validate_answer`, because RFC 3264
/// offer/answer makes the answer's format order the offer's. So codec selection belongs to whoever
/// authors the offer — `story:browser-media-adapter` — and the only thing this server can honestly
/// do with an Opus-first offer is decline to run it.
#[test]
fn an_answer_cannot_select_pcmu_against_an_opus_first_offer() {
    let offered = sipx_sdp::parse(&browser_offer()).expect("the offer is SDP");
    let answer = answer_offer(&browser_offer(), &local())
        .expect("the offer is answered")
        .answer;
    let reordered = answer.replace(
        "UDP/TLS/RTP/SAVPF 111 0 8 13 101",
        "UDP/TLS/RTP/SAVPF 0 8 111 13 101",
    );
    assert_ne!(reordered, answer, "the m= line was actually reordered");

    let parsed = sipx_sdp::parse(&reordered).expect("the reordered answer is still SDP");
    assert_eq!(
        validate(&parsed, BrowserAudioRole::Answerer)
            .expect("and on its own it looks admissible")
            .selected_audio_payload,
        0,
        "read alone, a PCMU-first answer selects PCMU — which is what makes this tempting"
    );
    assert_eq!(
        validate_answer(&offered, &parsed, SetupCapabilities::both()).err(),
        Some(ProfileError::CodecSetIncomplete),
        "but against its own offer the profile refuses it, so the answerer has no say"
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
