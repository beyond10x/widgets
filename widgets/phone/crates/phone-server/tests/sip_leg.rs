//! The SIP leg, hermetically: one real outbound call over loopback UDP.
//!
//! No network, no second machine and no external process. Both endpoints are ordinary
//! `sipx_transport::bind` handles on `127.0.0.1:0`, and the far side plays exactly the role
//! `sipx answer` plays — 180, then 200, then hold the call until the BYE. That is `sipx_call`'s
//! own supported `answer` role, driven from the library rather than from a subprocess, which is
//! also the only way to be sure which version answered: the `sipx` binary installed on this
//! machine is `0.8.0` and the library this crate is pinned to is `1.0.1`.
//!
//! `sipx_transport::in_process_pair` was the other candidate and is not used. It is
//! `#[doc(hidden)]` and its own documentation says downstream tests should use the higher-level
//! testkit harness — and that harness drives `sipx_call::dial` itself, so a test built on it would
//! be a test of sipx rather than of this crate's leg.

mod common;

use std::net::SocketAddr;
use std::time::Duration;

use common::{answer_and_wait, answer_one, destination, endpoint, PATIENCE};
use phone_server::sip::{LegFact, SipLeg};
use phone_server::wire::EndCause;
use sipx_call::DialOptions;
use tokio::sync::mpsc;

/// The next fact the leg reported, or a failure naming what was expected instead.
async fn fact(facts: &mut mpsc::UnboundedReceiver<LegFact>, expected: &str) -> LegFact {
    tokio::time::timeout(PATIENCE, facts.recv())
        .await
        .unwrap_or_else(|_| panic!("no fact within {PATIENCE:?}; expected {expected}"))
        .unwrap_or_else(|| panic!("the fact channel closed; expected {expected}"))
}

#[tokio::test(flavor = "multi_thread")]
async fn the_sip_leg_reports_ringing_then_answered_then_ended() {
    let (far, far_incoming) = endpoint().await;
    let (near, _near_incoming) = endpoint().await;
    let far_addr = far.local_addr();
    let far_side = tokio::spawn(answer_and_wait(far, far_incoming));

    let (report, mut facts) = mpsc::unbounded_channel();
    let leg = SipLeg::new(report);
    let destination = destination(far_addr);
    let options = SipLeg::options(&destination);

    let mut call = tokio::time::timeout(PATIENCE, leg.place(&near, &destination, &options))
        .await
        .expect("the call is placed within the patience")
        .expect("the far end answers, so the leg does not fail");

    // The two facts the page turns into `softphone.control.RingCall` and then `ConfirmAnswer`,
    // in that order: a call is not answered before it rang.
    assert_eq!(fact(&mut facts, "Ringing").await, LegFact::Ringing);
    assert_eq!(fact(&mut facts, "Answered").await, LegFact::Answered);

    leg.hang_up(&mut call).await;
    assert_eq!(
        fact(&mut facts, "Ended").await,
        LegFact::Ended(EndCause::Local)
    );
    assert!(call.is_ended(), "the leg is gone once it has been hung up");

    let far_call = tokio::time::timeout(PATIENCE, far_side)
        .await
        .expect("the far side finishes")
        .expect("the far side did not panic");
    assert!(
        far_call.is_ended(),
        "the far end saw the BYE, so the call really was ended on the wire"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_call_nobody_answers_is_reported_as_ended_rather_than_hanging() {
    // An endpoint that receives the INVITE and says nothing at all: the far end is there and the
    // call is not. `place` has to give up inside its own deadline and report a class.
    let (far, _far_incoming) = endpoint().await;
    let (near, _near_incoming) = endpoint().await;

    let (report, mut facts) = mpsc::unbounded_channel();
    let leg = SipLeg::new(report);
    let destination = destination(far.local_addr());
    let options = SipLeg::options(&destination).with_timeout(Duration::from_millis(400));

    let failure = tokio::time::timeout(PATIENCE, leg.place(&near, &destination, &options))
        .await
        .expect("the leg gives up inside its own deadline rather than ours")
        .expect_err("nobody answered");

    assert_eq!(
        fact(&mut facts, "Ended").await,
        LegFact::Ended(failure.cause),
        "the fact the page is told and the error the caller is given name the same class"
    );
    assert_eq!(
        failure.cause,
        EndCause::Timeout,
        "a far end that never answers is the specification's `Timeout` class"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn digits_leave_on_the_sip_leg_as_telephone_events() {
    let (far, mut far_incoming) = endpoint().await;
    let (near, _near_incoming) = endpoint().await;
    let far_addr = far.local_addr();

    let (report, mut facts) = mpsc::unbounded_channel();
    let leg = SipLeg::new(report);
    let destination = destination(far_addr);
    let options = SipLeg::options(&destination);

    // Both sides at once: `place` does not return until the far end has answered.
    let (call, far_call) = tokio::join!(
        leg.place(&near, &destination, &options),
        answer_one(&far, &mut far_incoming),
    );
    let call = call.expect("the far end answers");
    assert_eq!(fact(&mut facts, "Ringing").await, LegFact::Ringing);
    assert_eq!(fact(&mut facts, "Answered").await, LegFact::Answered);

    assert!(
        leg.send_digits(&call, "5", Duration::from_millis(120))
            .await,
        "the SIP leg carries the keypress"
    );

    let digit = tokio::time::timeout(PATIENCE, far_call.recv_digit())
        .await
        .expect("the keypress arrives within the patience")
        .expect("the far end read one full telephone event");
    assert_eq!(
        digit.to_string(),
        "5",
        "the digit goes out as an RFC 4733 event and not as audio"
    );
}

#[test]
fn a_destination_asks_for_no_deadline_unless_one_is_wanted() {
    let far = SocketAddr::new(common::LOOPBACK, 5060);
    let options: DialOptions = SipLeg::options(&destination(far));
    assert_eq!(
        options.timeout(),
        None,
        "the transaction layer's own 64·T1 is the default, not a deadline this crate invents"
    );
}
