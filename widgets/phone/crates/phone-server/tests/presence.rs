//! The routing table, over channels rather than sockets.
//!
//! Nothing here binds anything. A phone in the table is a handle and a queue, so what the table
//! decides is decidable with two `mpsc` channels — and that is the whole of the state this process
//! is permitted to hold, which is what makes it worth testing on its own.
//!
//! Every case reads the commands a phone was *told*, because that is what a page acts on. A table
//! that held the right rows and told nobody would pass a test written against `len()` and leave
//! every roster on every page wrong.

use phone_server::presence::{Claimed, Registry};
use phone_server::wire::ToBrowser;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// One phone's end of the wire.
fn phone() -> (UnboundedSender<ToBrowser>, UnboundedReceiver<ToBrowser>) {
    unbounded_channel()
}

/// Everything a phone has been told so far, drained.
fn told(receiver: &mut UnboundedReceiver<ToBrowser>) -> Vec<ToBrowser> {
    let mut all = Vec::new();
    while let Ok(message) = receiver.try_recv() {
        all.push(message);
    }
    all
}

#[test]
fn the_first_phone_joins_an_empty_room_and_is_told_nothing() {
    let registry = Registry::default();
    let (say, mut heard) = phone();

    let claimed = registry.announce("ada@phone.dev.test", "Ada", say);

    assert_eq!(
        claimed,
        Claimed::Accepted(Vec::new()),
        "an empty table answers an empty roster, not a refusal"
    );
    assert!(
        told(&mut heard).is_empty(),
        "a phone is never told about itself: it would appear in its own roster"
    );
    assert_eq!(registry.len(), 1);
}

#[test]
fn the_second_phone_learns_of_the_first_and_the_first_of_it() {
    let registry = Registry::default();
    let (ada_says, mut ada_hears) = phone();
    let (grace_says, mut grace_hears) = phone();

    registry.announce("ada@phone.dev.test", "Ada", ada_says);
    let claimed = registry.announce("grace@phone.dev.test", "Grace", grace_says);

    // Both directions, and they arrive by different routes: the joining phone gets the set it is
    // joining as a return value, and the phones already there get a command each.
    assert_eq!(
        claimed,
        Claimed::Accepted(vec![(
            "ada@phone.dev.test".to_owned(),
            "Ada".to_owned()
        )]),
        "the joining phone is answered with who is already here"
    );
    assert_eq!(
        told(&mut ada_hears),
        vec![ToBrowser::NotePresent {
            handle: "grace@phone.dev.test".to_owned(),
            label: "Grace".to_owned(),
        }],
        "and the phone already here is told about the arrival"
    );
    assert!(
        told(&mut grace_hears).is_empty(),
        "the arriving phone is told nothing on this channel; its roster came back from `announce`"
    );
}

#[test]
fn a_handle_somebody_holds_is_refused_and_changes_nothing() {
    let registry = Registry::default();
    let (ada_says, mut ada_hears) = phone();
    let (impostor_says, _impostor_hears) = phone();

    registry.announce("ada@phone.dev.test", "Ada", ada_says);
    let claimed = registry.announce("ada@phone.dev.test", "Not Ada", impostor_says);

    assert_eq!(claimed, Claimed::Refused);
    assert_eq!(registry.len(), 1, "the table still holds one phone");
    assert!(
        told(&mut ada_hears).is_empty(),
        "and the phone that holds the handle is told nothing — a refused claim is not an arrival"
    );
}

#[test]
fn a_refused_claim_does_not_take_the_handle_from_its_holder() {
    // The case that matters if `announce` ever inserts before it checks: the second phone's channel
    // would replace the first's, and every later message for that handle would go to the impostor.
    let registry = Registry::default();
    let (ada_says, mut ada_hears) = phone();
    let (impostor_says, _impostor_hears) = phone();

    registry.announce("ada@phone.dev.test", "Ada", ada_says);
    registry.announce("ada@phone.dev.test", "Not Ada", impostor_says);
    let (grace_says, _grace_hears) = phone();
    registry.announce("grace@phone.dev.test", "Grace", grace_says);

    assert_eq!(
        told(&mut ada_hears),
        vec![ToBrowser::NotePresent {
            handle: "grace@phone.dev.test".to_owned(),
            label: "Grace".to_owned(),
        }],
        "the arrival still reaches the original holder's channel"
    );
}

#[test]
fn a_departure_is_told_once_and_only_to_those_still_here() {
    let registry = Registry::default();
    let (ada_says, mut ada_hears) = phone();
    let (grace_says, mut grace_hears) = phone();
    registry.announce("ada@phone.dev.test", "Ada", ada_says);
    registry.announce("grace@phone.dev.test", "Grace", grace_says);
    let _ = told(&mut ada_hears);

    registry.depart("grace@phone.dev.test");

    assert_eq!(
        told(&mut ada_hears),
        vec![ToBrowser::NoteGone {
            handle: "grace@phone.dev.test".to_owned(),
        }]
    );
    assert!(
        told(&mut grace_hears).is_empty(),
        "the phone that left is not told about itself leaving"
    );
    assert_eq!(registry.len(), 1);
}

#[test]
fn departing_twice_tells_nobody_twice() {
    // `main.rs` calls `depart` on withdrawal and again when the connection closes, so this runs on
    // every page that logs out before closing its tab. A second `NoteGone` is a `wrong-state` at
    // every other page — `softphone.presence.PeerPhone` has one `depart` out of `Present` — so the
    // idempotence is what keeps a correct sequence from reporting a conflict.
    let registry = Registry::default();
    let (ada_says, mut ada_hears) = phone();
    let (grace_says, _grace_hears) = phone();
    registry.announce("ada@phone.dev.test", "Ada", ada_says);
    registry.announce("grace@phone.dev.test", "Grace", grace_says);
    let _ = told(&mut ada_hears);

    registry.depart("grace@phone.dev.test");
    registry.depart("grace@phone.dev.test");

    assert_eq!(
        told(&mut ada_hears).len(),
        1,
        "one departure, however many times it is reported"
    );
}

#[test]
fn a_handle_is_free_again_once_its_holder_leaves() {
    let registry = Registry::default();
    let (first_says, _first_hears) = phone();
    let (second_says, _second_hears) = phone();

    registry.announce("ada@phone.dev.test", "Ada", first_says);
    registry.depart("ada@phone.dev.test");
    let claimed = registry.announce("ada@phone.dev.test", "Ada again", second_says);

    assert_eq!(
        claimed,
        Claimed::Accepted(Vec::new()),
        "a handle nobody holds is claimable — the table keeps no record of a phone that left"
    );
    assert!(registry.is_empty() || registry.len() == 1);
}

#[test]
fn a_phone_whose_channel_is_gone_does_not_stop_the_others_being_told() {
    // A connection closing drops its receiver before `depart` runs, so `send` fails for that phone.
    // The others must still hear the arrival.
    let registry = Registry::default();
    let (dead_says, dead_hears) = phone();
    let (ada_says, mut ada_hears) = phone();
    registry.announce("dead@phone.dev.test", "Gone", dead_says);
    registry.announce("ada@phone.dev.test", "Ada", ada_says);
    let _ = told(&mut ada_hears);
    drop(dead_hears);

    let (grace_says, _grace_hears) = phone();
    registry.announce("grace@phone.dev.test", "Grace", grace_says);

    assert_eq!(
        told(&mut ada_hears),
        vec![ToBrowser::NotePresent {
            handle: "grace@phone.dev.test".to_owned(),
            label: "Grace".to_owned(),
        }],
        "one unreachable phone does not silence the table"
    );
}
