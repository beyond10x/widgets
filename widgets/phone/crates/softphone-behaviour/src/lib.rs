//! Behaviour behind every obligation the `softphone` specification declares.
//!
//! The specification determines the contract; this crate is the algorithm. Nothing generated is
//! edited: the component ports are generic over a behaviour bundle, so one type implementing every
//! `*Behavior` and `*Query` trait in `softphone_types::<domain>::obligations` is all a realization
//! needs, and [`system`] assembles the eight ports over it.
//!
//! # One store, eight bundles
//!
//! Each generated port owns its bundle by value, so the eight cannot share state unless the bundle
//! is a handle. [`Behaviour`] is that handle — `Rc<RefCell<Store>>` — and the single-threaded
//! browser this runs in is why `Rc` rather than `Arc`. A behaviour never calls back into the
//! system, so no borrow taken inside one is live when the next begins.
//!
//! # What the model does not decide, and this does
//!
//! Three things, each named where it happens rather than buried:
//!
//! * **Identity.** A `creates` outcome mints one. There is no entropy through this boundary, so
//!   ids are a counter rendered as a UUID — deterministic, which is also what makes the scenario
//!   suite byte-stable.
//! * **Time.** There is no clock either. `Store::tick` hands out one instant per command, counted
//!   from the epoch. A deployment with a clock replaces it; nothing in the model reads it.
//! * **What a call's log entry says.** `RecordCall` carries a call id and nothing else, so this
//!   crate keeps what the events did not: when a call started, when it was answered, and how it
//!   ended. That is storage, which is exactly the reason a view is an obligation.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::cell::RefCell;
use std::rc::Rc;

use softphone_types::primitives::{Timestamp, Uuid};
use softphone_types::{bridge, control, directory, history, local, media, presentation, sip};

mod bridge_impl;
mod control_impl;
mod directory_impl;
mod history_impl;
mod local_impl;
mod media_impl;
mod presentation_impl;
mod sip_impl;

/// Everything the running phone holds, and the little the model does not say how to hold.
///
/// One `Vec` per entity, of the generated snapshot — state beside data, which is the shape the
/// generated lifecycle refines back into a typed instance. A `Vec` and a linear scan because this
/// is one phone: the largest collection here is a call log somebody can read.
#[derive(Debug, Default)]
pub struct Store {
    /// `softphone.media.MediaSession`.
    pub sessions: Vec<media::MediaSessionSnapshot>,
    /// `softphone.control.PhoneEndpoint`.
    pub endpoints: Vec<control::PhoneEndpointSnapshot>,
    /// `softphone.control.Call`.
    pub calls: Vec<control::CallSnapshot>,
    /// `softphone.bridge.BridgeSession`.
    pub bridges: Vec<bridge::BridgeSessionSnapshot>,
    /// `softphone.sip.Registration`.
    pub registrations: Vec<sip::RegistrationSnapshot>,
    /// `softphone.sip.SipDialog`.
    pub dialogs: Vec<sip::SipDialogSnapshot>,
    /// `softphone.local.LoopbackDevice`.
    pub devices: Vec<local::LoopbackDeviceSnapshot>,
    /// `softphone.history.CallRecord`.
    pub records: Vec<history::CallRecordSnapshot>,
    /// `softphone.directory.Contact`.
    pub contacts: Vec<directory::ContactSnapshot>,
    /// `softphone.directory.ContactAddress`.
    pub addresses: Vec<directory::ContactAddressSnapshot>,
    /// `softphone.presentation.CallTile`.
    pub tiles: Vec<presentation::CallTileSnapshot>,
    /// `softphone.presentation.Console`.
    pub consoles: Vec<presentation::ConsoleSnapshot>,
    /// `softphone.presentation.Keypad`.
    pub keypads: Vec<presentation::KeypadSnapshot>,

    /// When each call began and, if it did, when it was answered.
    ///
    /// Not the model's: `Call` carries neither, and `CallRecord` requires both. Kept here because
    /// the alternative is a field on the entity that no command writes.
    pub timings: Vec<Timing>,
    /// How each call ended, for the log entry a binding will ask for afterwards.
    pub endings: Vec<Ending>,

    minted: u64,
    ticks: u64,
}

/// When one call started, and when it was answered.
#[derive(Debug, Clone)]
pub struct Timing {
    /// The call.
    pub call: control::CallId,
    /// The instant `Dial` or `OfferCall` created it.
    pub started_at: Timestamp,
    /// The instant somebody accepted it, if anybody did.
    pub answered_at: Option<Timestamp>,
}

/// How one call ended.
#[derive(Debug, Clone)]
pub struct Ending {
    /// The call.
    pub call: control::CallId,
    /// The class the log records.
    pub cause: control::EndCause,
    /// The SIP status a rejection carried, if it was a rejection.
    pub status: Option<control::RejectStatus>,
    /// The instant it ended.
    pub ended_at: Timestamp,
}

impl Store {
    /// A fresh identity.
    ///
    /// A counter in UUID clothing. There is no entropy through this boundary and inventing a
    /// weaker source would be worse than being obvious about it.
    pub fn mint(&mut self) -> Uuid {
        self.minted += 1;
        Uuid(format!("00000000-0000-4000-8000-{:012x}", self.minted))
    }

    /// The next instant, one second after the last.
    ///
    /// There is no clock here. A deployment that has one replaces this; the model reads it nowhere,
    /// so nothing else changes.
    pub fn tick(&mut self) -> Timestamp {
        self.ticks += 1;
        let total = self.ticks;
        Timestamp(format!(
            "1970-01-01T{:02}:{:02}:{:02}Z",
            total / 3600 % 24,
            total / 60 % 60,
            total % 60
        ))
    }

    /// The timing of one call, created on first use.
    fn timing(&mut self, call: &control::CallId) -> &mut Timing {
        if let Some(index) = self.timings.iter().position(|t| &t.call == call) {
            return &mut self.timings[index];
        }
        let started_at = self.tick();
        self.timings.push(Timing {
            call: call.clone(),
            started_at,
            answered_at: None,
        });
        let last = self.timings.len() - 1;
        &mut self.timings[last]
    }

    /// Records how a call ended, replacing any earlier answer.
    fn ended(
        &mut self,
        call: &control::CallId,
        cause: control::EndCause,
        status: Option<control::RejectStatus>,
    ) {
        let ended_at = self.tick();
        self.endings.retain(|e| &e.call != call);
        self.endings.push(Ending {
            call: call.clone(),
            cause,
            status,
            ended_at,
        });
    }
}

/// One handle onto one [`Store`], cloned once per generated port.
#[derive(Debug, Clone, Default)]
pub struct Behaviour {
    store: Rc<RefCell<Store>>,
}

impl Behaviour {
    /// A realization over a fresh, empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The store, for a test that wants to look at it directly.
    #[must_use]
    pub fn store(&self) -> Rc<RefCell<Store>> {
        Rc::clone(&self.store)
    }
}

/// The `softphone` system, every obligation satisfied, sharing one store.
///
/// The eight arguments are the eight components the specification declares; each gets a handle onto
/// the same [`Store`], which is what makes them one phone rather than eight.
#[must_use]
pub fn system() -> softphone_system::System<
    Behaviour,
    Behaviour,
    Behaviour,
    Behaviour,
    Behaviour,
    Behaviour,
    Behaviour,
    Behaviour,
> {
    let behaviour = Behaviour::new();
    softphone_system::System::new(
        bridge_binding::BridgeBinding::new(behaviour.clone()),
        call_history::CallHistory::new(behaviour.clone()),
        local_binding::LocalBinding::new(behaviour.clone()),
        media_session::MediaSession::new(behaviour.clone()),
        phone_console::PhoneConsole::new(behaviour.clone()),
        phone_control::PhoneControl::new(behaviour.clone()),
        phone_directory::PhoneDirectory::new(behaviour.clone()),
        sip_binding::SipBinding::new(behaviour),
    )
}
