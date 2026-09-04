//! Adversarial cases against the two local gates and against the identity this server puts in
//! every answer it authors.
//!
//! Written against `5115fa3`. Nothing here is a new capability: every case drives the crate's own
//! public entry points with the messages `wire::FromBrowser` declares, in orders the control
//! channel admits.

mod common;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{answer_one, destination, endpoint, LOOPBACK, PATIENCE};
use phone_server::bridge::Bridged;
use phone_server::session::{apply, Config, Live, Phone};
use phone_server::sip::{LegFact, SipLeg};
use phone_server::wire::{FromBrowser, ToBrowser};
use sipx_media::{Codec, MediaSession};
use sipx_transport::Handle;
use tokio::sync::mpsc;

/// A plain session on loopback, standing in for the browser leg — the same substitution
/// `tests/bridge.rs` makes, for the same reason.
async fn session(remote: SocketAddr) -> MediaSession {
    MediaSession::start(
        SocketAddr::new(LOOPBACK, 0),
        sipx_media::Config::new(remote, Codec::Pcmu),
    )
    .await
    .expect("a loopback media session")
}

/// One placed SIP call and the far end of it, both real.
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

/// 20 ms of a loud square wave at 8 kHz, so that a gate can be measured and not just read.
fn tone() -> Vec<i16> {
    (0..160)
        .map(|index| if (index / 20) % 2 == 0 { 8000 } else { -8000 })
        .collect()
}

/// A live bridge, and the browser leg's session kept aside so its gate can be read.
async fn live_bridge() -> (Live, Arc<MediaSession>, sipx_call::Call, Handle, Handle) {
    let (near_call, far_call, far, near) = placed_call().await;
    let browser = Arc::new(session(SocketAddr::new(LOOPBACK, 9)).await);
    let bridge = Bridged::connect(Arc::clone(&browser), &near_call);
    (
        Live {
            call: near_call,
            bridge,
        },
        browser,
        far_call,
        far,
        near,
    )
}

/// `softphone.control.Call` carries `muted` and `held` as two independent Boolean fields
/// (`ess/domains/control.yaml:84-87`, `:119-129`). Unholding restores what hold suspended; it does
/// not answer the mute question, and a page whose `muted` is still `true` must not be sending.
#[tokio::test(flavor = "multi_thread")]
async fn unholding_does_not_clear_the_pages_mute() {
    let (mut live, browser, far_call, _far, _near) = live_bridge().await;
    let call_id = "c1".to_owned();

    // The page has to be speaking for "the far end cannot hear it" to mean anything, and symmetric
    // RTP needs a first packet before the browser leg knows where to answer.
    let page = session(browser.local_addr()).await;
    let speaking = tokio::spawn(async move {
        loop {
            if !page.send(tone()).await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    apply(
        &FromBrowser::Mute {
            call_id: call_id.clone(),
            muted: true,
        },
        &mut live,
    )
    .await;
    // Correction round 2 replaced the assertion that used to stand here. It read
    // `browser.is_muted()` — "the page asked for muted, so the browser leg's outbound audio is
    // gated" — and that is the inversion the adversary's own second pass found: the browser leg's
    // outbound is what the *page* hears, so gating it on mute silenced the far end in the page's
    // ear and left the microphone crossing the bridge. The two gates are now the other way round,
    // and this case asserts what is audible rather than what a flag was set to, because a gate on
    // the wrong leg satisfies the flag either way.
    assert!(
        live.bridge.is_browser_muted(),
        "the page's mute is recorded"
    );
    assert!(
        live.call.is_muted(),
        "and it is the SIP leg that is gated, because that is the leg the microphone leaves by"
    );

    apply(
        &FromBrowser::Hold {
            call_id: call_id.clone(),
            held: true,
        },
        &mut live,
    )
    .await;
    apply(
        &FromBrowser::Hold {
            call_id,
            held: false,
        },
        &mut live,
    )
    .await;

    assert!(
        live.bridge.is_browser_muted(),
        "unholding restored the mute the page never lifted: `muted` and `held` are two fields in \
         the specification, so unholding must not answer the mute question"
    );
    assert!(
        !live.bridge.is_held(),
        "and hold really was lifted, so this is not passing because nothing changed"
    );

    // The measurement the flags cannot make: after an unhold, a page that is still muted is still
    // not heard.
    let heard = tokio::time::timeout(
        PATIENCE,
        far_call.record_at_least(4_000, Duration::from_secs(3)),
    )
    .await
    .expect("the far end stops recording within the patience");
    speaking.abort();
    let loudest = heard.iter().copied().map(i16::abs).max().unwrap_or(0);
    assert!(
        loudest <= 1_000,
        "unholding let a muted page be heard again: {} samples at the far end, loudest {loudest}",
        heard.len(),
    );
}

/// The other direction of the same aliasing: clearing mute while the call is held un-gates a leg
/// hold is supposed to be holding.
#[tokio::test(flavor = "multi_thread")]
async fn clearing_mute_while_held_does_not_resume_a_held_leg() {
    let (mut live, browser, _far_call, _far, _near) = live_bridge().await;
    let call_id = "c1".to_owned();

    apply(
        &FromBrowser::Hold {
            call_id: call_id.clone(),
            held: true,
        },
        &mut live,
    )
    .await;
    assert!(browser.is_muted(), "a held leg stops sending");

    apply(
        &FromBrowser::Mute {
            call_id,
            muted: false,
        },
        &mut live,
    )
    .await;

    assert!(
        browser.is_muted(),
        "a held call that was never unheld is still held: clearing `muted` must not resume the \
         browser leg's outbound audio"
    );
    assert!(
        live.call.is_muted(),
        "and the SIP leg's half of the hold is untouched, which is what makes the two halves \
         disagree observable"
    );
}

/// An offer of the shape a browser sends. The fixture is `tests/browser_profile.rs`'s, restated
/// because an integration test cannot import another one.
///
/// **PCMU before Opus in the `m=` line, and that one token is all that changed in correction round
/// 2.** This case is about the `o=` line and the codec order is incidental to it — but pass 2 of
/// the adversary required that an Opus-first offer be refused *before* `ConfirmBridge` is sent,
/// and this case reads the answer out of that very command. With Opus first there is now no answer
/// to read and the case fails on its own helper rather than on its subject. Every codec the
/// profile requires is still present with its own `rtpmap`, so this is a reordering and not a
/// weakening; `tests/browser_profile.rs:a_pcmu_first_offer_is_the_shape_this_server_can_bridge`
/// measures that the reordered shape is on the profile.
fn browser_offer() -> String {
    [
        "v=0",
        "o=- 4611731400430051336 2 IN IP4 192.0.2.10",
        "s=-",
        "t=0 0",
        "a=ice-options:trickle",
        "m=audio 51234 UDP/TLS/RTP/SAVPF 0 8 111 13 101",
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

/// Open one bridge far enough to read the answer this server authored, and abandon it there.
///
/// `Phone::open` sends `ConfirmBridge` before it starts the handshake, so the answer is available
/// without a browser on the other end — which is the only part of the leg this case is about.
async fn answer_of_one_bridge(phone: &Phone, bridge: &str) -> String {
    let told: Mutex<Vec<ToBrowser>> = Mutex::new(Vec::new());
    let bridge_id = bridge.to_owned();
    let session_id = format!("{bridge}-session");
    let opened = tokio::time::timeout(
        Duration::from_secs(3),
        phone.open(
            &bridge_id,
            &session_id,
            "sip:1000@127.0.0.1",
            &browser_offer(),
            &|command| told.lock().expect("nothing panics here").push(command),
        ),
    )
    .await;
    // Either outcome is fine: the handshake needs a browser. What is being read is the answer that
    // already crossed the wire.
    drop(opened);
    told.into_inner()
        .expect("nothing panics here")
        .into_iter()
        .find_map(|command| match command {
            ToBrowser::ConfirmBridge { answer, .. } => Some(answer),
            _ => None,
        })
        .expect("`ConfirmBridge` carries the answer this server authored")
}

/// RFC 4566 §5.2: the tuple of `<username> <sess-id> <nettype> <addrtype> <unicast-address>` has
/// to be a globally unique identifier for the session. This server advertises one address for
/// every phone (`Config::media_address`), so the session id is the only part that can differ — and
/// `main.rs` runs one `Phone` per connection over one process, so two bridges at once is the
/// ordinary case rather than a constructed one.
#[tokio::test(flavor = "multi_thread")]
async fn two_bridges_do_not_advertise_the_same_sdp_origin() {
    let (signalling, _unsolicited) = endpoint().await;
    let config = Config {
        sip_bind: SocketAddr::new(LOOPBACK, 0),
        sip_proxy: SocketAddr::new(LOOPBACK, 5060),
        from: "sip:phone-server@127.0.0.1".to_owned(),
        media_address: LOOPBACK,
        media_bind: LOOPBACK,
    };
    let phone = Phone::new(config, signalling);

    let first = tokio::time::timeout(PATIENCE, answer_of_one_bridge(&phone, "b1"))
        .await
        .expect("the first answer is authored inside the patience");
    let second = tokio::time::timeout(PATIENCE, answer_of_one_bridge(&phone, "b2"))
        .await
        .expect("the second answer is authored inside the patience");

    let origin = |answer: &str| {
        answer
            .lines()
            .find(|line| line.starts_with("o="))
            .expect("an answer has an origin line")
            .to_owned()
    };
    assert_ne!(
        origin(&first),
        origin(&second),
        "two bridges from one server share one SDP session identity, so nothing downstream can \
         tell the two sessions apart"
    );
}
