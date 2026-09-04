//! `softphone.control` — the surface a human or an agent drives identically.

use softphone_types::control::{self, obligations};
use softphone_types::obligation::UnmetObligation;

use crate::Behaviour;

fn stamp<S: control::call_state::Marker>(call: control::Call<S>) -> control::CallSnapshot {
    control::CallSnapshot {
        state: S::STATE,
        data: call.into_data(),
    }
}

/// The refusal for a call this store has never seen.
///
/// `Ended` for the reason the crate docs give: the model declares no not-found outcome, and a
/// terminal state is the one state no declared move starts in.
fn no_such_call() -> control::CallStateConflict {
    control::CallStateConflict {
        state: control::CallState::Ended,
    }
}

impl obligations::ConfigureEndpointBehavior for Behaviour {
    fn configure_endpoint(
        &mut self,
        input: control::ConfigureEndpoint,
    ) -> Result<control::ConfigureEndpointOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let endpoint_id = control::EndpointId(store.mint());
        store.endpoints.push(control::PhoneEndpointSnapshot {
            state: control::PhoneEndpointState::Enabled,
            data: control::PhoneEndpointData {
                endpoint_id: endpoint_id.clone(),
                label: input.label.clone(),
                default_binding: input.default_binding,
            },
        });
        Ok(control::ConfigureEndpointOutcome::Configured {
            endpoint_configured: control::EndpointConfigured {
                endpoint_id,
                label: input.label,
                default_binding: input.default_binding,
            },
        })
    }
}

impl obligations::DialBehavior for Behaviour {
    fn dial(&mut self, input: control::Dial) -> Result<control::DialOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let call_id = control::CallId(store.mint());
        store.calls.push(control::CallSnapshot {
            state: control::CallState::Requested,
            data: control::CallData {
                call_id: call_id.clone(),
                endpoint_id: input.endpoint_id.clone(),
                direction: control::CallDirection::Outbound,
                remote: input.remote.clone(),
                // No media yet, which is what the entity's invariant allows before `Active`.
                session_id: None,
                muted: false,
                held: false,
            },
        });
        store.timing(&call_id);
        Ok(control::DialOutcome::Dialled {
            call_dialled: control::CallDialled {
                call_id,
                endpoint_id: input.endpoint_id,
                remote: input.remote,
            },
        })
    }
}

impl obligations::OfferCallBehavior for Behaviour {
    fn offer_call(
        &mut self,
        input: control::OfferCall,
    ) -> Result<control::OfferCallOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let call_id = control::CallId(store.mint());
        store.calls.push(control::CallSnapshot {
            state: control::CallState::Requested,
            data: control::CallData {
                call_id: call_id.clone(),
                endpoint_id: input.endpoint_id.clone(),
                direction: control::CallDirection::Inbound,
                remote: input.remote.clone(),
                session_id: None,
                muted: false,
                held: false,
            },
        });
        store.timing(&call_id);
        Ok(control::OfferCallOutcome::Offered {
            call_offered: control::CallOffered {
                call_id,
                endpoint_id: input.endpoint_id,
                remote: input.remote,
            },
        })
    }
}

impl obligations::RingCallBehavior for Behaviour {
    fn ring_call(
        &mut self,
        input: control::RingCall,
    ) -> Result<control::RingCallOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .calls
            .iter()
            .position(|c| c.data.call_id == input.call_id)
        else {
            return Ok(control::RingCallOutcome::WrongState {
                error: no_such_call(),
            });
        };
        match store.calls.remove(index).refine() {
            control::AnyCall::Requested(call) => {
                store.calls.insert(index, stamp(call.ring()));
                Ok(control::RingCallOutcome::Ringing {
                    call_ringing: control::CallRinging {
                        call_id: input.call_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.calls.insert(index, other.snapshot());
                Ok(control::RingCallOutcome::WrongState {
                    error: control::CallStateConflict { state },
                })
            }
        }
    }
}

/// `accept`, which two commands take: a person answering an inbound call, and the kernel reporting
/// that the far end answered an outbound one. Both are the same move from the same two states.
fn accept(
    store: &mut crate::Store,
    call_id: &control::CallId,
) -> Result<(), Option<control::CallState>> {
    let Some(index) = store.calls.iter().position(|c| &c.data.call_id == call_id) else {
        return Err(None);
    };
    match store.calls.remove(index).refine() {
        control::AnyCall::Requested(call) => {
            store.calls.insert(index, stamp(call.accept()));
        }
        control::AnyCall::Ringing(call) => {
            store.calls.insert(index, stamp(call.accept()));
        }
        other => {
            let state = other.state();
            store.calls.insert(index, other.snapshot());
            return Err(Some(state));
        }
    }
    store.timing(call_id);
    let answered_at = store.tick();
    if let Some(timing) = store.timings.iter_mut().find(|t| &t.call == call_id) {
        timing.answered_at = Some(answered_at);
    }
    Ok(())
}

impl obligations::AnswerBehavior for Behaviour {
    fn answer(
        &mut self,
        input: control::Answer,
    ) -> Result<control::AnswerOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        match accept(&mut store, &input.call_id) {
            Ok(()) => Ok(control::AnswerOutcome::Answered {
                call_answered: control::CallAnswered {
                    call_id: input.call_id,
                },
            }),
            Err(state) => Ok(control::AnswerOutcome::WrongState {
                error: state
                    .map_or_else(no_such_call, |state| control::CallStateConflict { state }),
            }),
        }
    }
}

impl obligations::ConfirmAnswerBehavior for Behaviour {
    fn confirm_answer(
        &mut self,
        input: control::ConfirmAnswer,
    ) -> Result<control::ConfirmAnswerOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        match accept(&mut store, &input.call_id) {
            Ok(()) => Ok(control::ConfirmAnswerOutcome::Confirmed {
                call_confirmed: control::CallConfirmed {
                    call_id: input.call_id,
                },
            }),
            Err(state) => Ok(control::ConfirmAnswerOutcome::WrongState {
                error: state
                    .map_or_else(no_such_call, |state| control::CallStateConflict { state }),
            }),
        }
    }
}

impl obligations::MediaConnectedBehavior for Behaviour {
    fn media_connected(
        &mut self,
        input: control::MediaConnected,
    ) -> Result<control::MediaConnectedOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .calls
            .iter()
            .position(|c| c.data.call_id == input.call_id)
        else {
            return Ok(control::MediaConnectedOutcome::WrongState {
                error: no_such_call(),
            });
        };
        match store.calls.remove(index).refine() {
            control::AnyCall::Answering(call) => {
                store.calls.insert(index, stamp(call.activate()));
                Ok(control::MediaConnectedOutcome::Established {
                    call_established: control::CallEstablished {
                        call_id: input.call_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.calls.insert(index, other.snapshot());
                Ok(control::MediaConnectedOutcome::WrongState {
                    error: control::CallStateConflict { state },
                })
            }
        }
    }
}

/// `end`, which three commands take, from any of the four states a live call rests in.
fn end(
    store: &mut crate::Store,
    call_id: &control::CallId,
) -> Result<(), Option<control::CallState>> {
    let Some(index) = store.calls.iter().position(|c| &c.data.call_id == call_id) else {
        return Err(None);
    };
    match store.calls.remove(index).refine() {
        control::AnyCall::Requested(call) => store.calls.insert(index, stamp(call.end())),
        control::AnyCall::Ringing(call) => store.calls.insert(index, stamp(call.end())),
        control::AnyCall::Answering(call) => store.calls.insert(index, stamp(call.end())),
        control::AnyCall::Active(call) => store.calls.insert(index, stamp(call.end())),
        other => {
            let state = other.state();
            store.calls.insert(index, other.snapshot());
            return Err(Some(state));
        }
    }
    Ok(())
}

impl obligations::HangUpBehavior for Behaviour {
    fn hang_up(
        &mut self,
        input: control::HangUp,
    ) -> Result<control::HangUpOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        match end(&mut store, &input.call_id) {
            Ok(()) => {
                // What the log will need and no event carries: this side ended it.
                store.ended(&input.call_id, control::EndCause::Local, None);
                Ok(control::HangUpOutcome::HungUp {
                    call_hung_up: control::CallHungUp {
                        call_id: input.call_id,
                    },
                })
            }
            // The one refusal in this domain that carries no error: the specification declares
            // `wrong-state` here without one.
            Err(_) => Ok(control::HangUpOutcome::WrongState),
        }
    }
}

impl obligations::RejectBehavior for Behaviour {
    fn reject(
        &mut self,
        input: control::Reject,
    ) -> Result<control::RejectOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        match end(&mut store, &input.call_id) {
            Ok(()) => {
                store.ended(
                    &input.call_id,
                    control::EndCause::Refused,
                    Some(input.status.clone()),
                );
                Ok(control::RejectOutcome::Rejected {
                    call_rejected: control::CallRejected {
                        call_id: input.call_id,
                        status: input.status,
                    },
                })
            }
            Err(state) => Ok(control::RejectOutcome::WrongState {
                error: state
                    .map_or_else(no_such_call, |state| control::CallStateConflict { state }),
            }),
        }
    }
}

impl obligations::FailCallBehavior for Behaviour {
    fn fail_call(
        &mut self,
        input: control::FailCall,
    ) -> Result<control::FailCallOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        match end(&mut store, &input.call_id) {
            Ok(()) => {
                store.ended(&input.call_id, input.cause, None);
                Ok(control::FailCallOutcome::Failed {
                    call_failed: control::CallFailed {
                        call_id: input.call_id,
                        cause: input.cause,
                    },
                })
            }
            Err(state) => Ok(control::FailCallOutcome::WrongState {
                error: state
                    .map_or_else(no_such_call, |state| control::CallStateConflict { state }),
            }),
        }
    }
}

impl obligations::AttachMediaBehavior for Behaviour {
    fn attach_media(
        &mut self,
        input: control::AttachMedia,
    ) -> Result<control::AttachMediaOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        // `updates`, and the specification declares one outcome: there is no refusal to answer
        // with, so a call this store does not hold leaves nothing written and the event still says
        // what was asked for.
        if let Some(call) = store
            .calls
            .iter_mut()
            .find(|c| c.data.call_id == input.call_id)
        {
            call.data.session_id = Some(input.session_id.clone());
        }
        Ok(control::AttachMediaOutcome::Attached {
            media_attached: control::MediaAttached {
                call_id: input.call_id,
                session_id: input.session_id,
            },
        })
    }
}

impl obligations::SendDigitsBehavior for Behaviour {
    fn send_digits(
        &mut self,
        input: control::SendDigits,
    ) -> Result<control::SendDigitsOutcome, UnmetObligation> {
        // Digits change no field of the call. What carries them is `softphone.media.SendSignal`,
        // and the control layer's part is stating the intent.
        Ok(control::SendDigitsOutcome::Sent {
            digits_sent: control::DigitsSent {
                call_id: input.call_id,
                digits: input.digits,
            },
        })
    }
}

impl obligations::SetMutedBehavior for Behaviour {
    fn set_muted(
        &mut self,
        input: control::SetMuted,
    ) -> Result<control::SetMutedOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        if let Some(call) = store
            .calls
            .iter_mut()
            .find(|c| c.data.call_id == input.call_id)
        {
            call.data.muted = input.muted;
        }
        Ok(control::SetMutedOutcome::Changed {
            mute_changed: control::MuteChanged {
                call_id: input.call_id,
                muted: input.muted,
            },
        })
    }
}

impl obligations::SetHeldBehavior for Behaviour {
    fn set_held(
        &mut self,
        input: control::SetHeld,
    ) -> Result<control::SetHeldOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        if let Some(call) = store
            .calls
            .iter_mut()
            .find(|c| c.data.call_id == input.call_id)
        {
            call.data.held = input.held;
        }
        Ok(control::SetHeldOutcome::Changed {
            hold_changed: control::HoldChanged {
                call_id: input.call_id,
                held: input.held,
            },
        })
    }
}

fn call_row(snapshot: &control::CallSnapshot) -> control::CallRow {
    control::CallRow {
        call_id: snapshot.data.call_id.clone(),
        endpoint_id: snapshot.data.endpoint_id.clone(),
        direction: snapshot.data.direction,
        remote: snapshot.data.remote.clone(),
        session_id: snapshot.data.session_id.clone(),
        muted: snapshot.data.muted,
        held: snapshot.data.held,
        state: snapshot.state,
    }
}

impl obligations::EndpointByIdQuery for Behaviour {
    fn endpoint_by_id(&self) -> Result<Vec<control::EndpointById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .endpoints
            .iter()
            .map(|e| control::EndpointById {
                endpoint_id: e.data.endpoint_id.clone(),
                label: e.data.label.clone(),
                default_binding: e.data.default_binding,
                state: e.state,
            })
            .collect())
    }
}

impl obligations::CallByIdQuery for Behaviour {
    fn call_by_id(&self) -> Result<Vec<control::CallById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .calls
            .iter()
            .map(|c| {
                let row = call_row(c);
                control::CallById {
                    call_id: row.call_id,
                    endpoint_id: row.endpoint_id,
                    direction: row.direction,
                    remote: row.remote,
                    session_id: row.session_id,
                    muted: row.muted,
                    held: row.held,
                    state: row.state,
                }
            })
            .collect())
    }
}

impl obligations::ActiveCallsQuery for Behaviour {
    fn active_calls(&self) -> Result<Vec<control::ActiveCalls>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .calls
            .iter()
            .filter(|c| c.state == control::CallState::Active)
            .map(|c| {
                let row = call_row(c);
                control::ActiveCalls {
                    call_id: row.call_id,
                    endpoint_id: row.endpoint_id,
                    direction: row.direction,
                    remote: row.remote,
                    session_id: row.session_id,
                    muted: row.muted,
                    held: row.held,
                    state: row.state,
                }
            })
            .collect())
    }
}
