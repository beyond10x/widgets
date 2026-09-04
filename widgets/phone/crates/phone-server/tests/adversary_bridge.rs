//! The bridge on the codec pair this server actually produces.
//!
//! `tests/bridge.rs` records half a second of audio at the far end across a **PCMU browser leg**
//! and a PCMU SIP leg, so the forwarding it proves is `sipx_media::Bridge`'s pass-through path.
//! The server did not produce that pair when this file was written: a browser offer negotiates
//! **Opus** while `sip.rs` places a G.711 call, so every real bridge took `Bridge`'s *transcoding*
//! path — and that path carries buffers between legs with no rate conversion, so half a second
//! spoken arrived as 2 880 ms.
//!
//! Written against `5115fa3`. **Rewritten in correction round 1 under the authorisation in that
//! round's brief** ("if it does not: refuse an Opus acceptance at the seam … then the adversary's
//! case must assert the refusal instead, and you may rewrite it to do so only in that branch,
//! keeping its message"). The branch was established by measurement, not by reading:
//!
//! | measured | result |
//! | --- | --- |
//! | an Opus-first offer, answered by `browser_audio::answer` | answer is Opus-first, both sides select payload 111 |
//! | a **PCMU-first** offer, answered the same way | answer is PCMU-first, both sides select payload 0, `validate_answer` Ok |
//! | an Opus-first offer answered with a hand-reordered PCMU-first answer | `validate_answer` **Err(CodecSetIncomplete)** |
//!
//! So the profile does admit a PCMU selection — but only when the *offer* lists PCMU first, because
//! the answer must preserve the offer's format order. This server cannot make that choice for the
//! page. It therefore refuses the pair it cannot carry, and the case below asserts that refusal
//! and then measures the pair the server does negotiate. The original assertion and its message
//! are kept verbatim; only the codec of the leg they run over has changed.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use common::{answer_one, destination, endpoint, LOOPBACK, PATIENCE};
use phone_server::bridge::Bridged;
use phone_server::browser::{bridgeable, BrowserLeg, Refusal};
use phone_server::sip::{LegFact, SipLeg};
use sipx_media::dtls::openssl::Identity;
use sipx_media::{Codec, MediaSession};
use sipx_transport::Handle;
use tokio::sync::mpsc;

/// 20 ms of a loud square wave at 8 kHz — `tests/bridge.rs`'s tone, on G.711's clock.
fn tone_8k() -> Vec<i16> {
    (0..160)
        .map(|index| if (index / 20) % 2 == 0 { 8000 } else { -8000 })
        .collect()
}

/// 20 ms of the same wave at 48 kHz — Opus's clock, for the case that measures the mis-rating.
fn tone_48k() -> Vec<i16> {
    (0..960)
        .map(|index| if (index / 120) % 2 == 0 { 8000 } else { -8000 })
        .collect()
}

/// How many 20 ms frames the page speaks: half a second, the same half second
/// `tests/bridge.rs` measures.
const FRAMES: usize = 25;

/// A session on loopback in one codec.
async fn session(remote: SocketAddr, codec: Codec) -> MediaSession {
    MediaSession::start(
        SocketAddr::new(LOOPBACK, 0),
        sipx_media::Config::new(remote, codec),
    )
    .await
    .expect("a loopback media session")
}

/// One placed SIP call and the far end of it, both real and both G.711.
async fn placed_call() -> (sipx_call::Call, sipx_call::Call, Handle, Handle) {
    let (far, mut far_incoming) = endpoint().await;
    let (near, _near_incoming) = endpoint().await;
    let destination = destination(far.local_addr());
    let options = SipLeg::options(&destination);
    let (report, mut facts) = mpsc::unbounded_channel();
    let leg = SipLeg::new(report);

    let (near_call, far_call) = tokio::join!(
        leg.place(&near, &destination, &options),
        answer_one(&far, &mut far_incoming),
    );
    let near_call = near_call.expect("the far end answers");
    assert_eq!(facts.recv().await, Some(LegFact::Ringing));
    assert_eq!(facts.recv().await, Some(LegFact::Answered));
    (near_call, far_call, far, near)
}

/// Speak `FRAMES` frames on a browser leg bridged to a placed call, and answer what the far end
/// received.
async fn spoken_across_a_bridge(codec: Codec, frame: Vec<i16>) -> (usize, i16, bool) {
    let (near_call, far_call, _far, _near) = placed_call().await;
    let browser_leg = Arc::new(session(SocketAddr::new(LOOPBACK, 9), codec).await);
    let page = session(browser_leg.local_addr(), codec).await;
    let bridged = Bridged::connect(Arc::clone(&browser_leg), &near_call);
    assert!(bridged.is_connected());
    let transcoding = bridged.is_transcoding();

    // The page speaks for exactly half a second and then stops, so what the far end has is a
    // count and not a rate that has to be inferred.
    let speaking = tokio::spawn(async move {
        for _ in 0..FRAMES {
            if !page.send(frame.clone()).await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        page
    });

    // A bound on failure and not a window: `record_at_least` hands back whatever arrived, and it
    // arrives only from the relay — a leg that stops speaking pads nothing, which is what makes
    // the count a measurement of the audio rather than of the wall clock.
    let heard = tokio::time::timeout(
        PATIENCE,
        far_call.record_at_least(40_000, Duration::from_secs(3)),
    )
    .await
    .expect("the far end stops recording within the patience");
    let _page = speaking.await.expect("the page did not panic");

    let loudest = heard.iter().copied().map(i16::abs).max().unwrap_or(0);
    (heard.len(), loudest, transcoding)
}

/// Half a second spoken on the browser leg is half a second heard at the far end.
///
/// The count is what says which happened: 500 ms at 8 kHz is 4 000 samples, and 500 ms of 48 kHz
/// buffers poured into an 8 kHz session is 24 000.
#[tokio::test(flavor = "multi_thread")]
async fn half_a_second_spoken_on_the_browser_leg_is_half_a_second_at_the_far_end() {
    // The pair this server negotiates is the one it will accept, and it accepts only one clock.
    bridgeable(Codec::Pcmu).expect("a G.711 browser leg is the pair this server bridges");

    let (heard, loudest, transcoding) = spoken_across_a_bridge(Codec::Pcmu, tone_8k()).await;
    assert!(
        !transcoding,
        "one clock on both legs, so the payloads pass straight across"
    );
    assert!(
        loudest > 1_000,
        "the far end heard the page at all: {heard} samples, loudest {loudest}"
    );
    assert!(
        heard <= 8_000,
        "the page spoke {} ms and the far end received {heard} samples, which is {} ms of G.711 — the \
         transcoding path carries 48 kHz buffers into an 8 kHz session without resampling, so the \
         far end hears the browser sped up",
        FRAMES * 20,
        heard / 8,
    );
}

/// The seam refuses every codec pair the bridge cannot carry, and names the way through.
///
/// Not just Opus. The rule is one clock on both legs, so G.722 is refused too — its RTP clock is
/// 8 000 and its *audio* is 16 000 (RFC 3551 §4.5.2), which is exactly the trap a gate written
/// against `Codec::clock_rate` alone would fall into.
#[tokio::test(flavor = "multi_thread")]
async fn the_media_seam_refuses_every_pair_it_cannot_carry() {
    for codec in [Codec::Opus, Codec::G722, Codec::L16] {
        let refused = bridgeable(codec).expect_err("a rate this server cannot convert");
        assert!(
            matches!(refused.source, Refusal::NoRateConversion { .. }),
            "{codec:?} is refused for the rate and not for something else: {:?}",
            refused.source
        );
        assert_eq!(
            refused.cause,
            phone_server::wire::BridgeCause::Refused,
            "the server would not have it, which is the half the page acts on"
        );
        assert!(
            refused.to_string().contains("Offer PCMU before Opus"),
            "the refusal names the way through, since a page can take it: {refused}"
        );
    }
    for codec in [Codec::Pcmu, Codec::Pcma] {
        bridgeable(codec).expect("G.711 is 8 kHz on both legs");
    }

    // And the gate is on the path that would start a session, not only on a free function: a
    // bound leg asked to run Opus refuses before it touches ICE, DTLS or a key.
    let leg = BrowserLeg::bind(
        SocketAddr::new(LOOPBACK, 0),
        Identity::generate().expect("a DTLS identity"),
    )
    .await
    .expect("a media port");
    let (_local, ice) = leg
        .local(
            LOOPBACK,
            sipx_sdp::ice::Credentials::new("Sv3T", "ourPasswordIsTwentyTwoPlus")
                .expect("credentials RFC 8839 §5.4 admits"),
            7,
        )
        .await
        .expect("host candidates gather without a STUN server");
    let refused = leg
        .start(
            SocketAddr::new(LOOPBACK, 9),
            Codec::Opus,
            111,
            ice,
            sipx_sdp::fingerprint::Fingerprint {
                func: sipx_sdp::fingerprint::HashFunc::Sha256,
                digest: vec![0xAB; 32],
            },
            sipx_media::dtls::Role::Server,
        )
        .await
        .expect_err("an Opus browser leg is not a leg this server will run");
    assert!(
        matches!(refused.source, Refusal::NoRateConversion { .. }),
        "and it refuses for the rate, before ICE or DTLS has run: {:?}",
        refused.source
    );
}

/// Why the seam refuses: an Opus leg bridged to G.711 by hand really does mis-rate the audio.
///
/// This is the original measurement, kept. It goes around the seam on purpose — it constructs both
/// sessions itself — because it is the evidence for the refusal rather than a test of it, and the
/// refusal is only worth having while this is true.
///
/// **If this case ever fails, `sipx_media::Bridge` has gained rate conversion and
/// `browser::bridgeable` should be revisited** rather than this assertion relaxed.
#[tokio::test(flavor = "multi_thread")]
async fn an_opus_leg_bridged_to_g711_mis_rates_audio_which_is_why_the_seam_refuses() {
    let (heard, loudest, transcoding) = spoken_across_a_bridge(Codec::Opus, tone_48k()).await;
    assert!(
        transcoding,
        "different clocks on the two legs, so the bridge decodes and re-encodes"
    );
    assert!(loudest > 1_000, "the far end heard something");
    assert!(
        heard > 8_000,
        "upstream has changed: the page spoke {} ms and the far end received {heard} samples, \
         which is {} ms — `sipx_media::Bridge` now appears to convert rates, so the refusal in \
         `browser::bridgeable` may no longer be needed",
        FRAMES * 20,
        heard / 8,
    );
}
