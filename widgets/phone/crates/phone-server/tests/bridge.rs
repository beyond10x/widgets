//! The bridge, hermetically: audio really crossing from the browser leg to the far end.
//!
//! The browser leg here is a plain `sipx_media` session rather than a keyed browser-audio one,
//! and that substitution is the whole limit of this file: DTLS, ICE and the fingerprint check are
//! not exercised, because no test in this repository can open an `RTCPeerConnection`. What *is*
//! exercised is the thing the design rests on — that both legs are sipx media sessions, so
//! forwarding between them is `sipx_media::Bridge` and **no PCM is pushed into a live call by
//! hand**. If that were false this file could not be written at all.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use common::{answer_one, destination, endpoint, LOOPBACK, PATIENCE};
use phone_server::bridge::Bridged;
use phone_server::sip::{LegFact, SipLeg};
use sipx_media::{Codec, Config, MediaSession};
use sipx_transport::Handle;
use tokio::sync::mpsc;

/// 20 ms of a loud square wave at 8 kHz — audible, and nothing like silence after µ-law.
fn tone() -> Vec<i16> {
    (0..160)
        .map(|index| if (index / 20) % 2 == 0 { 8000 } else { -8000 })
        .collect()
}

/// A plain session on loopback, standing in for one end of the browser leg.
async fn session(remote: SocketAddr) -> MediaSession {
    MediaSession::start(
        SocketAddr::new(LOOPBACK, 0),
        Config::new(remote, Codec::Pcmu),
    )
    .await
    .expect("a loopback media session")
}

/// One placed SIP call and the far end of it, both real.
///
/// The two endpoints come back with the calls and are held by the caller for as long as the calls
/// live: dropping a `Handle` takes its signalling with it, and a call whose endpoint has gone
/// cannot be hung up.
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

#[tokio::test(flavor = "multi_thread")]
async fn two_g711_legs_are_bridged_without_transcoding() {
    let (near_call, _far_call, _far, _near) = placed_call().await;

    // Started pointing at a port nothing is on: symmetric RTP learns the real one from the first
    // packet that arrives, which is what makes an answerer's session possible at all.
    let browser = Arc::new(session(SocketAddr::new(LOOPBACK, 9)).await);
    let bridged = Bridged::connect(Arc::clone(&browser), &near_call);

    assert!(
        bridged.is_connected(),
        "both directions are running, because half a bridge is worse than no call"
    );
    assert!(
        !bridged.is_transcoding(),
        "a G.711 browser leg and a G.711 SIP leg pass payloads straight across"
    );
    // And a fact about the seam this arrangement actually uses, pinned so that changing it is a
    // decision: `Call::is_bridged` reports `sipx_call::CallBridge` membership, which is the path
    // from *two calls*. The browser leg is a `MediaSession` with no dialog around it, so the
    // forwarding is `sipx_media::Bridge` and the call does not know it is in one. If this ever
    // goes true, somebody has moved the bridge up a layer and both legs are now calls.
    assert!(
        !near_call.is_bridged(),
        "the bridge is the media coupling; `CallBridge` is the two-call path and is not this one"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn audio_crosses_the_bridge_to_the_far_end() {
    let (near_call, far_call, _far, _near) = placed_call().await;

    let browser_leg = Arc::new(session(SocketAddr::new(LOOPBACK, 9)).await);
    let page = session(browser_leg.local_addr()).await;
    let bridged = Bridged::connect(Arc::clone(&browser_leg), &near_call);
    assert!(bridged.is_connected());

    // The page talks, in 20 ms packets, for as long as this test is listening.
    let speaking = tokio::spawn(async move {
        loop {
            if !page.send(tone()).await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    // Half a second of audio at the far end of the SIP leg, three hops from the page: page →
    // browser leg → (bridge) → this call's near side → here.
    let heard = tokio::time::timeout(PATIENCE, far_call.record_at_least(4_000, PATIENCE))
        .await
        .expect("the far end records within the patience");
    speaking.abort();

    let loudest = heard.iter().copied().map(i16::abs).max().unwrap_or(0);
    assert!(
        loudest > 1_000,
        "the far end heard the page: {} samples, loudest {loudest}",
        heard.len()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn muting_the_browser_leg_is_local_to_it() {
    let (near_call, _far_call, _far, _near) = placed_call().await;
    let browser_leg = Arc::new(session(SocketAddr::new(LOOPBACK, 9)).await);
    let bridged = Bridged::connect(Arc::clone(&browser_leg), &near_call);

    // `set_muted` answers what the gate was *before*, so a caller can report real transitions
    // without a read-then-write that races a second muter.
    assert!(
        !bridged.set_browser_muted(true),
        "the browser leg was not muted before this"
    );
    assert!(
        bridged.set_browser_muted(true),
        "muting an already-muted leg is not a transition, and says so"
    );
    assert!(
        !near_call.is_muted(),
        "the SIP leg keeps sending: mute is local to the browser leg and the far end is told \
         nothing"
    );
    assert!(bridged.is_connected(), "a muted leg is still a bridged leg");
}
