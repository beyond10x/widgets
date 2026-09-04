//! `softphone.sip` — the leg the server holds.

use softphone_types::obligation::UnmetObligation;
use softphone_types::sip::{self, obligations};

use crate::Behaviour;

fn stamp_registration<S: sip::registration_state::Marker>(
    registration: sip::Registration<S>,
) -> sip::RegistrationSnapshot {
    sip::RegistrationSnapshot {
        state: S::STATE,
        data: registration.into_data(),
    }
}

fn stamp_dialog<S: sip::sip_dialog_state::Marker>(
    dialog: sip::SipDialog<S>,
) -> sip::SipDialogSnapshot {
    sip::SipDialogSnapshot {
        state: S::STATE,
        data: dialog.into_data(),
    }
}

fn no_such_registration() -> sip::RegistrationStateConflict {
    sip::RegistrationStateConflict {
        state: sip::RegistrationState::Ended,
    }
}

impl obligations::RegisterBehavior for Behaviour {
    fn register(&mut self, input: sip::Register) -> Result<sip::RegisterOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let registration_id = sip::RegistrationId(store.mint());
        store
            .registrations
            .push(stamp_registration(sip::Registration::new(
                sip::RegistrationData {
                    registration_id: registration_id.clone(),
                    aor: input.aor.clone(),
                    transport: input.transport,
                    expires: input.expires.clone(),
                },
            )));
        Ok(sip::RegisterOutcome::Requested {
            registration_requested: sip::RegistrationRequested {
                registration_id,
                aor: input.aor,
                transport: input.transport,
                expires: input.expires,
            },
        })
    }
}

impl obligations::ConfirmRegistrationBehavior for Behaviour {
    fn confirm_registration(
        &mut self,
        input: sip::ConfirmRegistration,
    ) -> Result<sip::ConfirmRegistrationOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .registrations
            .iter()
            .position(|r| r.data.registration_id == input.registration_id)
        else {
            return Ok(sip::ConfirmRegistrationOutcome::WrongState {
                error: no_such_registration(),
            });
        };
        match store.registrations.remove(index).refine() {
            sip::AnyRegistration::Requested(requested) => {
                store
                    .registrations
                    .insert(index, stamp_registration(requested.confirm()));
                Ok(sip::ConfirmRegistrationOutcome::Confirmed {
                    registration_confirmed: sip::RegistrationConfirmed {
                        registration_id: input.registration_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.registrations.insert(index, other.snapshot());
                Ok(sip::ConfirmRegistrationOutcome::WrongState {
                    error: sip::RegistrationStateConflict { state },
                })
            }
        }
    }
}

impl obligations::FailRegistrationBehavior for Behaviour {
    fn fail_registration(
        &mut self,
        input: sip::FailRegistration,
    ) -> Result<sip::FailRegistrationOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .registrations
            .iter()
            .position(|r| r.data.registration_id == input.registration_id)
        else {
            return Ok(sip::FailRegistrationOutcome::WrongState {
                error: no_such_registration(),
            });
        };
        match store.registrations.remove(index).refine() {
            sip::AnyRegistration::Requested(requested) => {
                store
                    .registrations
                    .insert(index, stamp_registration(requested.fail()));
            }
            sip::AnyRegistration::Registered(registered) => {
                store
                    .registrations
                    .insert(index, stamp_registration(registered.fail()));
            }
            other => {
                let state = other.state();
                store.registrations.insert(index, other.snapshot());
                return Ok(sip::FailRegistrationOutcome::WrongState {
                    error: sip::RegistrationStateConflict { state },
                });
            }
        }
        Ok(sip::FailRegistrationOutcome::Failed {
            registration_failed: sip::RegistrationFailed {
                registration_id: input.registration_id,
                cause: input.cause,
            },
        })
    }
}

impl obligations::RetryRegistrationBehavior for Behaviour {
    fn retry_registration(
        &mut self,
        input: sip::RetryRegistration,
    ) -> Result<sip::RetryRegistrationOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .registrations
            .iter()
            .position(|r| r.data.registration_id == input.registration_id)
        else {
            return Ok(sip::RetryRegistrationOutcome::WrongState {
                error: no_such_registration(),
            });
        };
        match store.registrations.remove(index).refine() {
            // `Failed` is not terminal, and this is the move that makes that true: one transient
            // drop must not kill a registration permanently.
            sip::AnyRegistration::Failed(failed) => {
                store
                    .registrations
                    .insert(index, stamp_registration(failed.retry()));
                Ok(sip::RetryRegistrationOutcome::Retried {
                    registration_retried: sip::RegistrationRetried {
                        registration_id: input.registration_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.registrations.insert(index, other.snapshot());
                Ok(sip::RetryRegistrationOutcome::WrongState {
                    error: sip::RegistrationStateConflict { state },
                })
            }
        }
    }
}

impl obligations::UnregisterBehavior for Behaviour {
    fn unregister(
        &mut self,
        input: sip::Unregister,
    ) -> Result<sip::UnregisterOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .registrations
            .iter()
            .position(|r| r.data.registration_id == input.registration_id)
        else {
            return Ok(sip::UnregisterOutcome::WrongState {
                error: no_such_registration(),
            });
        };
        match store.registrations.remove(index).refine() {
            sip::AnyRegistration::Registered(registered) => {
                store
                    .registrations
                    .insert(index, stamp_registration(registered.end()));
                Ok(sip::UnregisterOutcome::Ended {
                    registration_ended: sip::RegistrationEnded {
                        registration_id: input.registration_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.registrations.insert(index, other.snapshot());
                Ok(sip::UnregisterOutcome::WrongState {
                    error: sip::RegistrationStateConflict { state },
                })
            }
        }
    }
}

impl obligations::RefreshRegistrationBehavior for Behaviour {
    fn refresh_registration(
        &mut self,
        input: sip::RefreshRegistration,
    ) -> Result<sip::RefreshRegistrationOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        // `updates`, no move: a refresh extends the lifetime and changes no state.
        if let Some(registration) = store
            .registrations
            .iter_mut()
            .find(|r| r.data.registration_id == input.registration_id)
        {
            registration.data.expires = input.expires.clone();
        }
        Ok(sip::RefreshRegistrationOutcome::Refreshed {
            registration_refreshed: sip::RegistrationRefreshed {
                registration_id: input.registration_id,
                expires: input.expires,
            },
        })
    }
}

impl obligations::OpenDialogBehavior for Behaviour {
    fn open_dialog(
        &mut self,
        input: sip::OpenDialog,
    ) -> Result<sip::OpenDialogOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let dialog_id = sip::SipDialogId(store.mint());
        store
            .dialogs
            .push(stamp_dialog(sip::SipDialog::new(sip::SipDialogData {
                dialog_id: dialog_id.clone(),
                session_id: input.session_id.clone(),
                registration_id: input.registration_id.clone(),
                remote_uri: input.remote_uri.clone(),
                media_security: input.media_security,
                local_sdp: None,
                remote_sdp: None,
            })));
        Ok(sip::OpenDialogOutcome::Opened {
            sip_dialog_opened: sip::SipDialogOpened {
                dialog_id,
                session_id: input.session_id,
                registration_id: input.registration_id,
                remote_uri: input.remote_uri,
                media_security: input.media_security,
            },
        })
    }
}

impl obligations::RequestLocalMediaBehavior for Behaviour {
    fn request_local_media(
        &mut self,
        input: sip::RequestLocalMedia,
    ) -> Result<sip::RequestLocalMediaOutcome, UnmetObligation> {
        // Asking is not having: the outcome declares no subject, so nothing is written until an
        // offer arrives through `OfferLocalMedia`.
        Ok(sip::RequestLocalMediaOutcome::Requested {
            local_media_requested: sip::LocalMediaRequested {
                dialog_id: input.dialog_id,
            },
        })
    }
}

impl obligations::OfferLocalMediaBehavior for Behaviour {
    fn offer_local_media(
        &mut self,
        input: sip::OfferLocalMedia,
    ) -> Result<sip::OfferLocalMediaOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        if let Some(dialog) = store
            .dialogs
            .iter_mut()
            .find(|d| d.data.dialog_id == input.dialog_id)
        {
            dialog.data.local_sdp = Some(input.sdp.clone());
        }
        Ok(sip::OfferLocalMediaOutcome::Offered {
            local_media_offered: sip::LocalMediaOffered {
                dialog_id: input.dialog_id,
                sdp: input.sdp,
            },
        })
    }
}

impl obligations::ApplyRemoteMediaBehavior for Behaviour {
    fn apply_remote_media(
        &mut self,
        input: sip::ApplyRemoteMedia,
    ) -> Result<sip::ApplyRemoteMediaOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        if let Some(dialog) = store
            .dialogs
            .iter_mut()
            .find(|d| d.data.dialog_id == input.dialog_id)
        {
            dialog.data.remote_sdp = Some(input.sdp.clone());
        }
        Ok(sip::ApplyRemoteMediaOutcome::Applied {
            remote_media_applied: sip::RemoteMediaApplied {
                dialog_id: input.dialog_id,
                sdp: input.sdp,
            },
        })
    }
}

impl obligations::FailLocalMediaBehavior for Behaviour {
    fn fail_local_media(
        &mut self,
        input: sip::FailLocalMedia,
    ) -> Result<sip::FailLocalMediaOutcome, UnmetObligation> {
        // The dialog stays open: media failing is not the dialog closing, and the specification
        // keeps them apart by giving this outcome no subject.
        Ok(sip::FailLocalMediaOutcome::Failed {
            local_media_failed: sip::LocalMediaFailed {
                dialog_id: input.dialog_id,
                cause: input.cause,
            },
        })
    }
}

impl obligations::CloseDialogBehavior for Behaviour {
    fn close_dialog(
        &mut self,
        input: sip::CloseDialog,
    ) -> Result<sip::CloseDialogOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .dialogs
            .iter()
            .position(|d| d.data.dialog_id == input.dialog_id)
        else {
            return Ok(sip::CloseDialogOutcome::WrongState {
                error: sip::SipDialogStateConflict {
                    state: sip::SipDialogState::Closed,
                },
            });
        };
        match store.dialogs.remove(index).refine() {
            sip::AnySipDialog::Open(open) => {
                store.dialogs.insert(index, stamp_dialog(open.close()));
                Ok(sip::CloseDialogOutcome::Closed {
                    sip_dialog_closed: sip::SipDialogClosed {
                        dialog_id: input.dialog_id,
                        session_id: input.session_id,
                        cause: input.cause,
                        reason: input.reason,
                    },
                })
            }
            other => {
                let state = other.state();
                store.dialogs.insert(index, other.snapshot());
                Ok(sip::CloseDialogOutcome::WrongState {
                    error: sip::SipDialogStateConflict { state },
                })
            }
        }
    }
}

impl obligations::RegistrationByIdQuery for Behaviour {
    fn registration_by_id(&self) -> Result<Vec<sip::RegistrationById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .registrations
            .iter()
            .map(|r| sip::RegistrationById {
                registration_id: r.data.registration_id.clone(),
                aor: r.data.aor.clone(),
                transport: r.data.transport,
                state: r.state,
            })
            .collect())
    }
}

impl obligations::SipDialogByIdQuery for Behaviour {
    fn sip_dialog_by_id(&self) -> Result<Vec<sip::SipDialogById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .dialogs
            .iter()
            .map(|d| sip::SipDialogById {
                dialog_id: d.data.dialog_id.clone(),
                session_id: d.data.session_id.clone(),
                registration_id: d.data.registration_id.clone(),
                remote_uri: d.data.remote_uri.clone(),
                media_security: d.data.media_security,
                local_sdp: d.data.local_sdp.clone(),
                remote_sdp: d.data.remote_sdp.clone(),
                state: d.state,
            })
            .collect())
    }
}
