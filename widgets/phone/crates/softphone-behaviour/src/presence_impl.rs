//! `softphone.presence` — this phone's standing, and the roster it is told.

use softphone_types::obligation::UnmetObligation;
use softphone_types::presence::{self, obligations};

use crate::Behaviour;

fn stamp_presence<S: presence::presence_state::Marker>(
    presence: presence::Presence<S>,
) -> presence::PresenceSnapshot {
    presence::PresenceSnapshot {
        state: S::STATE,
        data: presence.into_data(),
    }
}

fn stamp_phone<S: presence::peer_phone_state::Marker>(
    phone: presence::PeerPhone<S>,
) -> presence::PeerPhoneSnapshot {
    presence::PeerPhoneSnapshot {
        state: S::STATE,
        data: phone.into_data(),
    }
}

/// The conflict reported for a presence that is not there at all.
///
/// `Withdrawn` and not a fourth word, for the reason the other domains give: the error carries a
/// declared state and there is none for "no such row", so the terminal one is the honest answer —
/// nothing can be done to it, which is what the caller needs to know.
fn no_such_presence() -> presence::PresenceStateConflict {
    presence::PresenceStateConflict {
        state: presence::PresenceState::Withdrawn,
    }
}

/// The same, for a phone this page was never told about.
fn no_such_phone() -> presence::PeerPhoneStateConflict {
    presence::PeerPhoneStateConflict {
        state: presence::PeerPhoneState::Gone,
    }
}

impl obligations::AnnouncePresenceBehavior for Behaviour {
    fn announce_presence(
        &mut self,
        input: presence::AnnouncePresence,
    ) -> Result<presence::AnnouncePresenceOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let presence_id = presence::PresenceId(store.mint());
        store.presences.push(stamp_presence(presence::Presence::new(
            presence::PresenceData {
                presence_id: presence_id.clone(),
                handle: input.handle.clone(),
                label: input.label.clone(),
                // Nothing has ended, so there is no cause. The entity's invariant is what says
                // a `Withdrawn` one always has.
                cause: None,
            },
        )));
        Ok(presence::AnnouncePresenceOutcome::Announcing {
            presence_announced: presence::PresenceAnnounced {
                presence_id,
                handle: input.handle,
                label: input.label,
            },
        })
    }
}

impl obligations::ConfirmPresenceBehavior for Behaviour {
    fn confirm_presence(
        &mut self,
        input: presence::ConfirmPresence,
    ) -> Result<presence::ConfirmPresenceOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .presences
            .iter()
            .position(|p| p.data.presence_id == input.presence_id)
        else {
            return Ok(presence::ConfirmPresenceOutcome::WrongState {
                error: no_such_presence(),
            });
        };
        match store.presences.remove(index).refine() {
            presence::AnyPresence::Announcing(announcing) => {
                store
                    .presences
                    .insert(index, stamp_presence(announcing.confirm()));
                Ok(presence::ConfirmPresenceOutcome::Confirmed {
                    presence_confirmed: presence::PresenceConfirmed {
                        presence_id: input.presence_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.presences.insert(index, other.snapshot());
                Ok(presence::ConfirmPresenceOutcome::WrongState {
                    error: presence::PresenceStateConflict { state },
                })
            }
        }
    }
}

/// `withdraw`, which two commands take: the page leaving, and the kernel reporting this phone
/// unreachable. One move; what differs is the cause each records.
fn withdraw(
    store: &mut crate::Store,
    presence_id: &presence::PresenceId,
    cause: presence::PresenceCause,
) -> Result<(), Option<presence::PresenceState>> {
    let Some(index) = store
        .presences
        .iter()
        .position(|p| &p.data.presence_id == presence_id)
    else {
        return Err(None);
    };
    let mut moved = match store.presences.remove(index).refine() {
        presence::AnyPresence::Announcing(announcing) => stamp_presence(announcing.withdraw()),
        presence::AnyPresence::Present(present) => stamp_presence(present.withdraw()),
        other => {
            let state = other.state();
            store.presences.insert(index, other.snapshot());
            return Err(Some(state));
        }
    };
    // Written with the move rather than after it: the invariant is that a `Withdrawn` presence has
    // a cause, and a snapshot inserted without one would be a value the type says cannot exist.
    moved.data.cause = Some(cause);
    store.presences.insert(index, moved);
    Ok(())
}

impl obligations::FailPresenceBehavior for Behaviour {
    fn fail_presence(
        &mut self,
        input: presence::FailPresence,
    ) -> Result<presence::FailPresenceOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        match withdraw(&mut store, &input.presence_id, input.cause) {
            Ok(()) => Ok(presence::FailPresenceOutcome::Failed {
                presence_failed: presence::PresenceFailed {
                    presence_id: input.presence_id,
                    cause: input.cause,
                },
            }),
            Err(state) => Ok(presence::FailPresenceOutcome::WrongState {
                error: state.map_or_else(no_such_presence, |state| {
                    presence::PresenceStateConflict { state }
                }),
            }),
        }
    }
}

impl obligations::WithdrawPresenceBehavior for Behaviour {
    fn withdraw_presence(
        &mut self,
        input: presence::WithdrawPresence,
    ) -> Result<presence::WithdrawPresenceOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        // `Local` is not read from the input and cannot be: the specification sets it as a literal,
        // because this command *is* the local decision and a caller supplying the class could
        // report its own departure as a transport failure.
        match withdraw(
            &mut store,
            &input.presence_id,
            presence::PresenceCause::Local,
        ) {
            Ok(()) => Ok(presence::WithdrawPresenceOutcome::Withdrawn {
                presence_withdrawn: presence::PresenceWithdrawn {
                    presence_id: input.presence_id,
                },
            }),
            Err(state) => Ok(presence::WithdrawPresenceOutcome::WrongState {
                error: state.map_or_else(no_such_presence, |state| {
                    presence::PresenceStateConflict { state }
                }),
            }),
        }
    }
}

impl obligations::NotePresentBehavior for Behaviour {
    fn note_present(
        &mut self,
        input: presence::NotePresent,
    ) -> Result<presence::NotePresentOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        // The handle is the identity, so a phone announced twice is one row and not two. The server
        // replaying its roster after a reconnection is the case that makes this matter.
        if let Some(existing) = store
            .phones
            .iter_mut()
            .find(|p| p.data.handle == input.handle)
        {
            existing.data.label = input.label.clone();
            existing.state = presence::PeerPhoneState::Present;
        } else {
            store.phones.push(stamp_phone(presence::PeerPhone::new(
                presence::PeerPhoneData {
                    handle: input.handle.clone(),
                    label: input.label.clone(),
                },
            )));
        }
        Ok(presence::NotePresentOutcome::Noted {
            peer_present: presence::PeerPresent {
                handle: input.handle,
                label: input.label,
            },
        })
    }
}

impl obligations::NoteGoneBehavior for Behaviour {
    fn note_gone(
        &mut self,
        input: presence::NoteGone,
    ) -> Result<presence::NoteGoneOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .phones
            .iter()
            .position(|p| p.data.handle == input.handle)
        else {
            return Ok(presence::NoteGoneOutcome::WrongState {
                error: no_such_phone(),
            });
        };
        match store.phones.remove(index).refine() {
            presence::AnyPeerPhone::Present(present) => {
                store.phones.insert(index, stamp_phone(present.depart()));
                Ok(presence::NoteGoneOutcome::Noted {
                    peer_gone: presence::PeerGone {
                        handle: input.handle,
                    },
                })
            }
            other => {
                let state = other.state();
                store.phones.insert(index, other.snapshot());
                Ok(presence::NoteGoneOutcome::WrongState {
                    error: presence::PeerPhoneStateConflict { state },
                })
            }
        }
    }
}

impl obligations::MyPresenceQuery for Behaviour {
    fn my_presence(&self) -> Result<Vec<presence::MyPresence>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .presences
            .iter()
            .filter(|p| p.state == presence::PresenceState::Present)
            .map(|p| presence::MyPresence {
                presence_id: p.data.presence_id.clone(),
                handle: p.data.handle.clone(),
                label: p.data.label.clone(),
                cause: p.data.cause,
                state: p.state,
            })
            .collect())
    }
}

impl obligations::PresentPhonesQuery for Behaviour {
    fn present_phones(&self) -> Result<Vec<presence::PresentPhones>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .phones
            .iter()
            .filter(|p| p.state == presence::PeerPhoneState::Present)
            .map(|p| presence::PresentPhones {
                handle: p.data.handle.clone(),
                label: p.data.label.clone(),
                state: p.state,
            })
            .collect())
    }
}

impl obligations::PhoneByHandleQuery for Behaviour {
    fn phone_by_handle(&self) -> Result<Vec<presence::PhoneByHandle>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .phones
            .iter()
            .map(|p| presence::PhoneByHandle {
                handle: p.data.handle.clone(),
                label: p.data.label.clone(),
                state: p.state,
            })
            .collect())
    }
}
