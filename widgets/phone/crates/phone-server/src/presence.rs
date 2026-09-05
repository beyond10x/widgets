//! The routing table: which handles are here, and how to reach each one.
//!
//! This is the state `architecture-decision-record:server-holds-a-routing-table` permits, and the
//! only state this process holds. Everything in it is answerable by looking at the sockets this
//! process owns, and none of it outlives them: a handle exists exactly as long as the connection
//! that claimed it. There is no last-seen, no availability and no record of a phone that is not
//! connected right now, which is why `softphone.presence` models present and gone and nothing else.
//!
//! What is *not* here is a call. Which phone is ringing which, and what state that call is in, stays
//! in the browser — one authority, as the accepted ADR requires.
//!
//! **A handle is claimed, not assigned.** Two phones asking for one handle is a conflict only this
//! process can see, so `announce` answers with a verdict and the page's `Presence` stays
//! `Announcing` until it arrives.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::UnboundedSender;

use crate::wire::ToBrowser;

/// One phone in the table.
struct Present {
    /// What other phones are told to show instead of the handle.
    label: String,
    /// The one queue everything this phone is told goes through.
    say: UnboundedSender<ToBrowser>,
}

/// What became of a claim.
#[derive(Debug, PartialEq, Eq)]
pub enum Claimed {
    /// The handle is this phone's, and these are the phones already here.
    ///
    /// The roster is answered rather than pushed one message at a time, because a page that has
    /// just announced needs the set it is joining and the caller is what turns it into commands.
    Accepted(Vec<(String, String)>),
    /// Somebody else holds it.
    Refused,
}

/// Every phone currently connected to this process.
///
/// Cloneable and shared: one table per process, and every connection holds the same one. The lock
/// is a `std::sync::Mutex` rather than tokio's on purpose — nothing here awaits while holding it,
/// and an unbounded `send` does not block, so there is no point at which this could be held across
/// a suspension.
#[derive(Clone, Default)]
pub struct Registry {
    phones: Arc<Mutex<HashMap<String, Present>>>,
}

impl Registry {
    /// Claim a handle, and tell every other phone about this one.
    ///
    /// The fan-out happens here rather than at the call site so that a phone can never appear in
    /// the table without the others being told — the two are one critical section.
    pub fn announce(&self, handle: &str, label: &str, say: UnboundedSender<ToBrowser>) -> Claimed {
        let mut phones = self.phones.lock().expect("the registry lock is never poisoned");
        if phones.contains_key(handle) {
            return Claimed::Refused;
        }
        let others: Vec<(String, String)> = phones
            .iter()
            .map(|(handle, present)| (handle.clone(), present.label.clone()))
            .collect();
        for present in phones.values() {
            // A closed channel is a phone whose connection is going away and whose `depart` has not
            // run yet. Dropping the message is right: the row is about to leave the table anyway.
            let _ = present.say.send(ToBrowser::NotePresent {
                handle: handle.to_owned(),
                label: label.to_owned(),
            });
        }
        phones.insert(
            handle.to_owned(),
            Present {
                label: label.to_owned(),
                say,
            },
        );
        Claimed::Accepted(others)
    }

    /// Give a handle back, and tell everyone still here.
    ///
    /// Idempotent: a connection that both withdrew and then closed calls this twice, and the second
    /// call must not tell every phone that somebody left a second time — `softphone.presence`
    /// refuses a second `NoteGone` with a `wrong-state`, and a server that produced one would be
    /// making every page report a conflict for something it did correctly.
    pub fn depart(&self, handle: &str) {
        let mut phones = self.phones.lock().expect("the registry lock is never poisoned");
        if phones.remove(handle).is_none() {
            return;
        }
        for present in phones.values() {
            let _ = present.say.send(ToBrowser::NoteGone {
                handle: handle.to_owned(),
            });
        }
    }

    /// How many phones are here. For tests and for a log line; nothing decides on it.
    #[must_use]
    pub fn len(&self) -> usize {
        self.phones
            .lock()
            .expect("the registry lock is never poisoned")
            .len()
    }

    /// Whether the table is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
