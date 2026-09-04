//! `softphone.media` — the neutral session.

use softphone_types::media::{self, obligations};
use softphone_types::obligation::UnmetObligation;

use crate::Behaviour;

/// The typed instance back into the shape the store holds.
fn stamp<S: media::media_session_state::Marker>(
    session: media::MediaSession<S>,
) -> media::MediaSessionSnapshot {
    media::MediaSessionSnapshot {
        state: S::STATE,
        data: session.into_data(),
    }
}

impl obligations::OpenSessionBehavior for Behaviour {
    fn open_session(
        &mut self,
        input: media::OpenSession,
    ) -> Result<media::OpenSessionOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let session_id = media::MediaSessionId(store.mint());
        let data = media::MediaSessionData {
            session_id: session_id.clone(),
            session_ref: input.session_ref.clone(),
            binding: input.binding,
            profile: input.profile.clone(),
            participant: input.participant.clone(),
            // Absent until `terminate` supplies it, which is the entity's own invariant.
            termination: None,
        };
        store.sessions.push(stamp(media::MediaSession::new(data)));
        Ok(media::OpenSessionOutcome::Opened {
            session_opened: media::SessionOpened {
                session_id,
                session_ref: input.session_ref,
                binding: input.binding,
                profile: input.profile,
                participant: input.participant,
            },
        })
    }
}

impl obligations::ActivateSessionBehavior for Behaviour {
    fn activate_session(
        &mut self,
        input: media::ActivateSession,
    ) -> Result<media::ActivateSessionOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .sessions
            .iter()
            .position(|s| s.data.session_id == input.session_id)
        else {
            // An instance this store never saw. See the crate docs: the model declares no
            // not-found outcome, so this is refused as a move from a terminal state — the one
            // state no declared move starts in.
            return Ok(media::ActivateSessionOutcome::WrongState {
                error: media::MediaSessionStateConflict {
                    state: media::MediaSessionState::Terminated,
                },
            });
        };
        let session = store.sessions.remove(index);
        match session.refine() {
            media::AnyMediaSession::Requested(requested) => {
                store.sessions.insert(index, stamp(requested.establish()));
                Ok(media::ActivateSessionOutcome::Activated {
                    session_activated: media::SessionActivated {
                        session_id: input.session_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.sessions.insert(index, other.snapshot());
                Ok(media::ActivateSessionOutcome::WrongState {
                    error: media::MediaSessionStateConflict { state },
                })
            }
        }
    }
}

impl obligations::TerminateSessionBehavior for Behaviour {
    fn terminate_session(
        &mut self,
        input: media::TerminateSession,
    ) -> Result<media::TerminateSessionOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .sessions
            .iter()
            .position(|s| s.data.session_id == input.session_id)
        else {
            return Ok(media::TerminateSessionOutcome::WrongState {
                error: media::MediaSessionStateConflict {
                    state: media::MediaSessionState::Terminated,
                },
            });
        };
        let session = store.sessions.remove(index);
        let terminated = |mut snapshot: media::MediaSessionSnapshot,
                          reason: media::TerminationReason| {
            // `sets: termination: input.reason`, and the entity's invariant is why it is written
            // in the same step as the move rather than after it.
            snapshot.data.termination = Some(reason);
            snapshot
        };
        match session.refine() {
            media::AnyMediaSession::Requested(requested) => {
                let moved = terminated(stamp(requested.terminate()), input.reason);
                store.sessions.insert(index, moved);
            }
            media::AnyMediaSession::Live(live) => {
                let moved = terminated(stamp(live.terminate()), input.reason);
                store.sessions.insert(index, moved);
            }
            other => {
                let state = other.state();
                store.sessions.insert(index, other.snapshot());
                return Ok(media::TerminateSessionOutcome::WrongState {
                    error: media::MediaSessionStateConflict { state },
                });
            }
        }
        Ok(media::TerminateSessionOutcome::Terminated {
            session_terminated: media::SessionTerminated {
                session_id: input.session_id,
                reason: input.reason,
            },
        })
    }
}

impl obligations::SendSignalBehavior for Behaviour {
    fn send_signal(
        &mut self,
        input: media::SendSignal,
    ) -> Result<media::SendSignalOutcome, UnmetObligation> {
        // A signal changes no field of the session — the outcome declares no subject — so this is
        // the whole behaviour: the event says the digit left.
        Ok(media::SendSignalOutcome::Sent {
            signal_sent: media::SignalSent {
                session_id: input.session_id,
                signal: input.signal,
            },
        })
    }
}

impl obligations::ReceiveSignalBehavior for Behaviour {
    fn receive_signal(
        &mut self,
        input: media::ReceiveSignal,
    ) -> Result<media::ReceiveSignalOutcome, UnmetObligation> {
        Ok(media::ReceiveSignalOutcome::Received {
            signal_received: media::SignalReceived {
                session_id: input.session_id,
                signal: input.signal,
            },
        })
    }
}

impl obligations::InterruptOutputBehavior for Behaviour {
    fn interrupt_output(
        &mut self,
        input: media::InterruptOutput,
    ) -> Result<media::InterruptOutputOutcome, UnmetObligation> {
        Ok(media::InterruptOutputOutcome::Interrupted {
            output_interrupted: media::OutputInterrupted {
                session_id: input.session_id,
            },
        })
    }
}

impl obligations::MediaSessionByIdQuery for Behaviour {
    fn media_session_by_id(&self) -> Result<Vec<media::MediaSessionById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .sessions
            .iter()
            .map(|s| media::MediaSessionById {
                session_id: s.data.session_id.clone(),
                session_ref: s.data.session_ref.clone(),
                binding: s.data.binding,
                profile: s.data.profile.clone(),
                participant: s.data.participant.clone(),
                termination: s.data.termination,
                state: s.state,
            })
            .collect())
    }
}

impl obligations::LiveSessionsQuery for Behaviour {
    fn live_sessions(&self) -> Result<Vec<media::LiveSessions>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .sessions
            .iter()
            .filter(|s| s.state == media::MediaSessionState::Live)
            .map(|s| media::LiveSessions {
                session_id: s.data.session_id.clone(),
                session_ref: s.data.session_ref.clone(),
                binding: s.data.binding,
                profile: s.data.profile.clone(),
                participant: s.data.participant.clone(),
                termination: s.data.termination,
                state: s.state,
            })
            .collect())
    }
}
