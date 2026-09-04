//! `softphone.local` — the loopback binding, the phone with no network at all.

use softphone_types::local::{self, obligations};
use softphone_types::obligation::UnmetObligation;

use crate::Behaviour;

fn stamp<S: local::loopback_device_state::Marker>(
    device: local::LoopbackDevice<S>,
) -> local::LoopbackDeviceSnapshot {
    local::LoopbackDeviceSnapshot {
        state: S::STATE,
        data: device.into_data(),
    }
}

impl obligations::AttachLoopbackBehavior for Behaviour {
    fn attach_loopback(
        &mut self,
        input: local::AttachLoopback,
    ) -> Result<local::AttachLoopbackOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let device_id = local::LoopbackDeviceId(store.mint());
        store
            .devices
            .push(stamp(local::LoopbackDevice::new(local::LoopbackDeviceData {
                device_id: device_id.clone(),
                session_id: input.session_id.clone(),
                capture: input.capture.clone(),
                render: input.render.clone(),
            })));
        Ok(local::AttachLoopbackOutcome::Attached {
            loopback_attached: local::LoopbackAttached {
                device_id,
                session_id: input.session_id,
                capture: input.capture,
                render: input.render,
            },
        })
    }
}

impl obligations::DetachLoopbackBehavior for Behaviour {
    fn detach_loopback(
        &mut self,
        input: local::DetachLoopback,
    ) -> Result<local::DetachLoopbackOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .devices
            .iter()
            .position(|d| d.data.device_id == input.device_id)
        else {
            return Ok(local::DetachLoopbackOutcome::WrongState {
                error: local::LoopbackDeviceStateConflict {
                    state: local::LoopbackDeviceState::Detached,
                },
            });
        };
        match store.devices.remove(index).refine() {
            local::AnyLoopbackDevice::Attached(attached) => {
                store.devices.insert(index, stamp(attached.detach()));
                Ok(local::DetachLoopbackOutcome::Detached {
                    loopback_detached: local::LoopbackDetached {
                        device_id: input.device_id,
                        session_id: input.session_id,
                        reason: input.reason,
                    },
                })
            }
            other => {
                let state = other.state();
                store.devices.insert(index, other.snapshot());
                Ok(local::DetachLoopbackOutcome::WrongState {
                    error: local::LoopbackDeviceStateConflict { state },
                })
            }
        }
    }
}

impl obligations::LoopbackDeviceByIdQuery for Behaviour {
    fn loopback_device_by_id(&self) -> Result<Vec<local::LoopbackDeviceById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .devices
            .iter()
            .map(|d| local::LoopbackDeviceById {
                device_id: d.data.device_id.clone(),
                session_id: d.data.session_id.clone(),
                capture: d.data.capture.clone(),
                render: d.data.render.clone(),
                state: d.state,
            })
            .collect())
    }
}
