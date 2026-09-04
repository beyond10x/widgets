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
use phone_server::session::{self, Live};
use phone_server::sip::{LegFact, SipLeg};
use phone_server::wire::{BridgeCause, TerminationReason, ToBrowser};
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
async fn muting_the_page_stops_the_far_end_hearing_it_and_nothing_else() {
    // This case asserted `!near_call.is_muted()` under the name
    // `muting_the_browser_leg_is_local_to_it`, with the message "the SIP leg keeps sending: mute is
    // local to the browser leg". Correction round 2 established that both halves of that were the
    // wrong way round — `MediaSession::set_muted` gates a session's *outbound* audio, so the leg
    // that carries the page's microphone to the far end is the SIP leg, and gating the browser leg
    // instead silenced the far end in the page's ear while the microphone kept crossing. What is
    // local about muting is that no signalling moves and the page keeps hearing the far end, which
    // is what this now asserts.
    let (near_call, _far_call, _far, _near) = placed_call().await;
    let browser_leg = Arc::new(session(SocketAddr::new(LOOPBACK, 9)).await);
    let bridged = Bridged::connect(Arc::clone(&browser_leg), &near_call);

    // `set_muted` answers what the gate was *before*, so a caller can report real transitions
    // without a read-then-write that races a second muter.
    assert!(
        !bridged.set_browser_muted(true),
        "the page was not muted before this"
    );
    assert!(
        bridged.set_browser_muted(true),
        "muting an already-muted page is not a transition, and says so"
    );

    assert!(
        near_call.is_muted(),
        "the microphone leaves by the SIP leg, so that is the leg mute gates"
    );
    assert!(
        !browser_leg.is_muted(),
        "and the browser leg keeps sending, because that is the direction the page listens to — \
         mute is not hold, and a muted page still hears the far end"
    );
    assert!(!bridged.is_held(), "nothing here held the call");
    assert!(bridged.is_connected(), "a muted leg is still a bridged leg");
}

/// Hold gates both directions, which is what makes it not mute.
///
/// `softphone.control` says of holding — and not of muting — that "this side stops sending and
/// rendering". Two gates, so two assertions; the case exists because hold worked by accident while
/// mute was inverted, and `set_held` calling `Call::mute` itself was the accident.
#[tokio::test(flavor = "multi_thread")]
async fn holding_gates_both_directions() {
    let (near_call, _far_call, _far, _near) = placed_call().await;
    let browser_leg = Arc::new(session(SocketAddr::new(LOOPBACK, 9)).await);
    let bridged = Bridged::connect(Arc::clone(&browser_leg), &near_call);

    assert!(!bridged.set_held(true), "the call was not held before this");
    assert!(
        near_call.is_muted(),
        "held: the far end hears silence from this side"
    );
    assert!(
        browser_leg.is_muted(),
        "held: and the page hears silence from the far end"
    );

    assert!(bridged.set_held(false), "and hold is lifted");
    assert!(
        !near_call.is_muted() && !browser_leg.is_muted(),
        "unholding a call nobody muted opens both directions again"
    );
}

/// The guard the round-2 pass asked for: the SIP leg's audio rate is what the constant says.
///
/// `browser::SIP_LEG_AUDIO_RATE` is the number `browser::bridgeable` refuses a browser leg against,
/// and it is right only because `SipLeg::options` leaves `DialOptions::new`'s default codec set
/// alone. Nothing asserted that, so widening `SipLeg::options` to a codec set with G.722 or L16 in
/// it would silently reopen the mis-rating this crate refuses — the constant would go on saying
/// 8 000 while the leg ran at 16 000 or 44 100. This reads the rate off a leg that was really
/// negotiated with a real far end.
#[tokio::test(flavor = "multi_thread")]
async fn the_sip_legs_negotiated_audio_rate_is_what_the_constant_says() {
    let (near_call, far_call, _far, _near) = placed_call().await;
    assert_eq!(
        near_call.media().audio_rate(),
        phone_server::browser::SIP_LEG_AUDIO_RATE,
        "the SIP leg negotiated a rate `browser::bridgeable` compares browser legs against, and \
         the two no longer agree"
    );
    assert_eq!(
        far_call.media().audio_rate(),
        phone_server::browser::SIP_LEG_AUDIO_RATE,
        "and the far end agrees, so this is the negotiated rate and not one side's preference"
    );
}

/// A bridge that comes up and then dies tells the page so.
///
/// The row *the bridge is gone → `softphone.bridge.FailBridge`* has been in this unit's brief
/// since the first round and was unimplemented until the second: `fail_bridge` was reachable only
/// from `OpenFailed`, so a bridge that died after coming up left the page's `BridgeSession` in
/// `Live` with nothing ever moving it.
///
/// The death here is the ordinary one — the SIP leg goes away, which stops its media session, and
/// sipx's `Bridge` takes both directions down when either stops.
#[tokio::test(flavor = "multi_thread")]
async fn a_bridge_that_dies_mid_call_tells_the_page() {
    let (mut near_call, _far_call, _far, _near) = placed_call().await;
    let browser_leg = Arc::new(session(SocketAddr::new(LOOPBACK, 9)).await);
    let bridge = Bridged::connect(Arc::clone(&browser_leg), &near_call);
    assert!(bridge.is_connected(), "it came up first");

    // Hang up the SIP leg: its media session stops, so the bridge direction reading from it ends.
    near_call.hang_up().await.expect("the BYE goes out");
    let live = Live {
        call: near_call,
        bridge,
    };

    let told = tokio::time::timeout(
        PATIENCE,
        session::bridge_lost(&live, &"b9".to_owned(), &"s9".to_owned()),
    )
    .await
    .expect("the loss is noticed within the patience, and not only on the next page message");

    assert_eq!(
        told,
        ToBrowser::FailBridge {
            bridge_id: "b9".to_owned(),
            session_id: "s9".to_owned(),
            cause: BridgeCause::Transport,
            reason: TerminationReason::TransportLost,
        },
        "the forwarding stopped without this server closing it, which is what `Transport` is for"
    );
    assert!(
        !live.bridge.is_connected(),
        "and it really is gone rather than the watch having answered early"
    );
}
