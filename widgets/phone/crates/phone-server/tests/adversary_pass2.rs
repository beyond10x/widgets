//! Adversarial cases against correction round 1, written against `35f55cb`.
//!
//! Two subjects, both of them things the correction's own prose asserts and no case measures.
//!
//! **Which direction a gate gates.** `bridge.rs` now carries two flags and one `regate()`, and
//! every case about them — `tests/bridge.rs:muting_the_browser_leg_is_local_to_it` and both cases
//! in `tests/adversary_gates.rs` — asserts the *flag* (`MediaSession::is_muted`) and never the
//! audio. `sipx_media::MediaSession::set_muted` gates the session's **outbound** audio
//! (`sipx-media-1.0.1/src/session.rs:2676-2692`: "Every audio frame the send loop takes off the
//! queue is replaced by … silence … Reception is not affected in any way"). The browser leg's
//! outbound is the audio going **to the page**, so gating it silences the far end in the page's
//! ear and leaves the page's microphone crossing the bridge untouched. These cases measure the
//! audio rather than the flag, which is the only way the direction can be wrong and green.
//!
//! **Where the rate refusal sits relative to the answer.** `browser::bridgeable`'s doc says the
//! refusal happens "before ICE, before DTLS, before any key exists, and before the SIP call is
//! placed". `session::Phone::open` says `ConfirmBridge` — the answer, and the one command that
//! moves the page's bridge to `Live` — *before* it asks whether the pair is bridgeable, so the
//! page is handed a session description for a bridge this server is in the act of refusing.
//!
//! The harness is `tests/bridge.rs`'s, unchanged: a plain `sipx_media` session standing in for the
//! browser leg, a real placed SIP call over loopback, and audio measured at the far end.

mod common;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{answer_one, destination, endpoint, LOOPBACK, PATIENCE};
use phone_server::bridge::Bridged;
use phone_server::browser::Refusal;
use phone_server::session::{Config, OpenFailed, Phone};
use phone_server::sip::{LegFact, SipLeg};
use phone_server::wire::ToBrowser;
use sipx_media::{Codec, MediaSession};
use sipx_transport::Handle;
use tokio::sync::mpsc;

/// 20 ms of a loud square wave at 8 kHz — `tests/bridge.rs`'s tone, so that what is measured here
/// is the gate and not a different signal.
fn tone() -> Vec<i16> {
    (0..160)
        .map(|index| if (index / 20) % 2 == 0 { 8000 } else { -8000 })
        .collect()
}

/// A plain session on loopback, standing in for one end of the browser leg.
async fn session(remote: SocketAddr) -> MediaSession {
    MediaSession::start(
        SocketAddr::new(LOOPBACK, 0),
        sipx_media::Config::new(remote, Codec::Pcmu),
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

/// The loudest sample in a recording, which is what says whether a gate held.
fn loudest(recorded: &[i16]) -> i16 {
    recorded.iter().copied().map(i16::abs).max().unwrap_or(0)
}

/// `softphone.control.SetMuted` is the microphone: a muted page is a page the **far end** cannot
/// hear.
///
/// `ess/domains/control.yaml` puts `muted` on the call beside `held` and says of holding — not of
/// muting — that "this side stops sending and rendering"; `sipx-call`'s own `Call::mute` heads its
/// section "outbound audio only" and this crate's `set_browser_muted` repeats it. Outbound *of the
/// browser leg* is the direction that carries the far end's voice to the page, and the page's
/// microphone arrives on that session's **reception**, which `set_muted` explicitly does not
/// touch. So the audio the far end hears is the measurement, and no existing case takes it: they
/// read the flag the gate was set to.
#[tokio::test(flavor = "multi_thread")]
async fn a_muted_page_is_not_heard_by_the_far_end() {
    let (near_call, far_call, _far, _near) = placed_call().await;

    // Started pointing at a port nothing is on: symmetric RTP learns the real one from the first
    // packet that arrives, exactly as `tests/bridge.rs` does it.
    let browser_leg = Arc::new(session(SocketAddr::new(LOOPBACK, 9)).await);
    let page = session(browser_leg.local_addr()).await;
    let bridged = Bridged::connect(Arc::clone(&browser_leg), &near_call);
    assert!(bridged.is_connected());

    // The page mutes before it says a word, so nothing raced the gate into place.
    assert!(
        !bridged.set_browser_muted(true),
        "the leg was not muted before this"
    );
    assert!(
        bridged.is_browser_muted(),
        "and the page's mute is recorded"
    );

    let speaking = tokio::spawn(async move {
        loop {
            if !page.send(tone()).await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    // A bound on failure rather than a window: a far end that hears nothing returns short, and a
    // far end that hears the muted page returns as soon as half a second has arrived.
    let heard = tokio::time::timeout(
        PATIENCE,
        far_call.record_at_least(4_000, Duration::from_secs(3)),
    )
    .await
    .expect("the far end stops recording within the patience");
    speaking.abort();

    assert!(
        loudest(&heard) <= 1_000,
        "a muted page is being heard by the far end: {} samples at the far end, loudest {} — \
         `MediaSession::set_muted` gates the session's *outbound* audio, and the browser leg's \
         outbound is what the page hears, so the microphone the page muted still crosses the \
         bridge into the SIP leg",
        heard.len(),
        loudest(&heard),
    );
}

/// The other half of the same inversion, and it contradicts a sentence in the code.
///
/// `bridge.rs`'s `set_browser_muted` says: "What the browser hears is unaffected — muting gates
/// this side's outbound audio and nothing else." Both halves of that sentence cannot be true at
/// once, because this side's outbound audio *is* what the browser hears. This case takes the
/// second half at its word and measures the first.
///
/// The unmuted recording is the control: if the far end's voice does not reach the page at all,
/// this case fails on the control's message and measures nothing about mute.
#[tokio::test(flavor = "multi_thread")]
async fn muting_does_not_change_what_the_browser_hears() {
    let (near_call, far_call, _far, _near) = placed_call().await;

    let browser_leg = Arc::new(session(SocketAddr::new(LOOPBACK, 9)).await);
    let page = Arc::new(session(browser_leg.local_addr()).await);
    let bridged = Bridged::connect(Arc::clone(&browser_leg), &near_call);
    assert!(bridged.is_connected());

    // The page has to say something before the browser leg knows where to send: symmetric RTP
    // learns the address from the first packet. It keeps talking for the whole case.
    let talking = Arc::clone(&page);
    let speaking = tokio::spawn(async move {
        loop {
            if !talking.send(tone()).await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    });

    // Six seconds of the far end talking without a pause, so that every recording below is of
    // live audio rather than of the tail of a clip that has finished.
    let clip: Vec<i16> = std::iter::repeat_with(tone).take(300).flatten().collect();
    let far = Arc::new(far_call);
    let playing = tokio::spawn({
        let far = Arc::clone(&far);
        async move { far.play(&clip).await }
    });

    let open = tokio::time::timeout(
        PATIENCE,
        page.record_at_least(4_000, Duration::from_secs(3)),
    )
    .await
    .expect("the page stops recording within the patience");
    assert!(
        loudest(&open) > 1_000,
        "the control: with the leg open the page hears the far end at all ({} samples, loudest \
         {}). Without this the case measures nothing about muting",
        open.len(),
        loudest(&open),
    );

    assert!(!bridged.set_browser_muted(true), "now the page mutes");
    // A second of audio thrown away, because the queues between the far end and the page held
    // frames that were taken off the wire before the gate closed. Without this the measurement
    // below reads pre-mute audio and passes for the wrong reason — it did, when this case was
    // first written without the drain.
    let _drained = tokio::time::timeout(
        PATIENCE,
        page.record_at_least(8_000, Duration::from_secs(3)),
    )
    .await
    .expect("the page stops recording within the patience");
    let muted = tokio::time::timeout(
        PATIENCE,
        page.record_at_least(4_000, Duration::from_secs(3)),
    )
    .await
    .expect("the page stops recording within the patience");
    playing.abort();
    speaking.abort();

    assert!(
        loudest(&muted) > 1_000,
        "muting the page's microphone stopped the page hearing the far end: {} samples, loudest \
         {} against {} before the mute — the gate is on the browser leg's outbound audio, which \
         is the render direction, so `SetMuted` deafens the page instead of muting it",
        muted.len(),
        loudest(&muted),
        loudest(&open),
    );
}

/// An offer of the shape a browser sends, Opus first. `tests/browser_profile.rs`'s fixture,
/// restated because an integration test cannot import another one.
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

/// A refusal that has already answered the offer is not fail-closed.
///
/// `softphone.bridge.ConfirmBridge` is the command that moves the page's `BridgeSession` to
/// `Live` (`ess/domains/bridge.yaml`), and `bridge.yaml`'s own actor comment says the page "may
/// not assert that the far side accepted it" — which is why the command is the kernel's. This
/// server issues it, carrying the SDP answer, and *then* refuses the pair: a page that runs the
/// command it was sent installs a remote description and starts ICE and DTLS against a media port
/// this server has already dropped.
///
/// Ordinary rather than constructed: the offer is the one `tests/browser_profile.rs` calls "the
/// shape a browser sends", every browser puts Opus first in it, and
/// `tests/adversary_gates.rs:two_bridges_do_not_advertise_the_same_sdp_origin` drives this same
/// path today to read the answer out of the `ConfirmBridge` this case says should not be there.
#[tokio::test(flavor = "multi_thread")]
async fn an_unbridgeable_pair_is_refused_before_the_answer_reaches_the_page() {
    let (signalling, _unsolicited) = endpoint().await;
    let config = Config {
        sip_bind: SocketAddr::new(LOOPBACK, 0),
        sip_proxy: SocketAddr::new(LOOPBACK, 5060),
        from: "sip:phone-server@127.0.0.1".to_owned(),
        media_address: LOOPBACK,
        media_bind: LOOPBACK,
    };
    let phone = Phone::new(config, signalling);

    let told: Mutex<Vec<ToBrowser>> = Mutex::new(Vec::new());
    let failure = tokio::time::timeout(
        PATIENCE,
        phone.open(
            &"b1".to_owned(),
            &"b1-session".to_owned(),
            "sip:1000@127.0.0.1",
            &browser_offer(),
            &|command| told.lock().expect("nothing panics here").push(command),
        ),
    )
    .await
    .expect("the refusal does not wait for a handshake")
    .map(|_| ())
    .expect_err("an Opus browser leg against a G.711 SIP leg is not a bridge this server has");

    // The refusal is the one the correction added, and not some other failure on the way.
    let OpenFailed::Browser(refused) = failure else {
        panic!("the open failed for something other than the codec pair: {failure}");
    };
    assert!(
        matches!(refused.source, Refusal::NoRateConversion { .. }),
        "and it is the rate refusal: {:?}",
        refused.source
    );

    let said: Vec<&'static str> = told
        .lock()
        .expect("nothing panics here")
        .iter()
        .map(ToBrowser::command)
        .collect();
    assert!(
        said.is_empty(),
        "the page was told {said:?} for a bridge this server then refused — \
         `softphone.bridge.ConfirmBridge` carries the answer and moves the bridge to `Live`, so \
         the page installs a remote description and starts ICE and DTLS against a media port that \
         is already gone. A refusal that has answered the offer first is not fail-closed"
    );
}
