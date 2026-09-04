//! `softphone.bridge` — the carrier this page opens.

use softphone_types::bridge::{self, obligations};
use softphone_types::obligation::UnmetObligation;

use crate::Behaviour;

fn stamp<S: bridge::bridge_session_state::Marker>(
    session: bridge::BridgeSession<S>,
) -> bridge::BridgeSessionSnapshot {
    bridge::BridgeSessionSnapshot {
        state: S::STATE,
        data: session.into_data(),
    }
}

fn no_such_bridge() -> bridge::BridgeStateConflict {
    bridge::BridgeStateConflict {
        state: bridge::BridgeSessionState::Closed,
    }
}

impl obligations::ConnectBridgeBehavior for Behaviour {
    fn connect_bridge(
        &mut self,
        input: bridge::ConnectBridge,
    ) -> Result<bridge::ConnectBridgeOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let bridge_id = bridge::BridgeId(store.mint());
        store.bridges.push(stamp(bridge::BridgeSession::new(
            bridge::BridgeSessionData {
                bridge_id: bridge_id.clone(),
                session_id: input.session_id.clone(),
                endpoint: input.endpoint.clone(),
                local_sdp: input.offer.clone(),
                // One offer, one answer: nothing has answered yet, and the entity's invariant is
                // what says no media crosses before it does.
                remote_sdp: None,
                cause: None,
            },
        )));
        Ok(bridge::ConnectBridgeOutcome::Connecting {
            bridge_connecting: bridge::BridgeConnecting {
                bridge_id,
                session_id: input.session_id,
                endpoint: input.endpoint,
                offer: input.offer,
            },
        })
    }
}

impl obligations::ConfirmBridgeBehavior for Behaviour {
    fn confirm_bridge(
        &mut self,
        input: bridge::ConfirmBridge,
    ) -> Result<bridge::ConfirmBridgeOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .bridges
            .iter()
            .position(|b| b.data.bridge_id == input.bridge_id)
        else {
            return Ok(bridge::ConfirmBridgeOutcome::WrongState {
                error: no_such_bridge(),
            });
        };
        match store.bridges.remove(index).refine() {
            bridge::AnyBridgeSession::Connecting(connecting) => {
                let mut moved = stamp(connecting.establish());
                // Written with the move, not after it: `Live` implies an answer arrived.
                moved.data.remote_sdp = Some(input.answer.clone());
                store.bridges.insert(index, moved);
                Ok(bridge::ConfirmBridgeOutcome::Confirmed {
                    bridge_confirmed: bridge::BridgeConfirmed {
                        bridge_id: input.bridge_id,
                        session_id: input.session_id,
                        answer: input.answer,
                    },
                })
            }
            other => {
                let state = other.state();
                store.bridges.insert(index, other.snapshot());
                Ok(bridge::ConfirmBridgeOutcome::WrongState {
                    error: bridge::BridgeStateConflict { state },
                })
            }
        }
    }
}

/// `close`, which two commands take: the page taking the bridge down, and the kernel reporting it
/// gone. The move is one; what differs is the `cause` each records.
fn close(
    store: &mut crate::Store,
    bridge_id: &bridge::BridgeId,
    cause: bridge::BridgeCause,
) -> Result<(), Option<bridge::BridgeSessionState>> {
    let Some(index) = store
        .bridges
        .iter()
        .position(|b| &b.data.bridge_id == bridge_id)
    else {
        return Err(None);
    };
    let mut moved = match store.bridges.remove(index).refine() {
        bridge::AnyBridgeSession::Connecting(connecting) => stamp(connecting.close()),
        bridge::AnyBridgeSession::Live(live) => stamp(live.close()),
        other => {
            let state = other.state();
            store.bridges.insert(index, other.snapshot());
            return Err(Some(state));
        }
    };
    moved.data.cause = Some(cause);
    store.bridges.insert(index, moved);
    Ok(())
}

impl obligations::FailBridgeBehavior for Behaviour {
    fn fail_bridge(
        &mut self,
        input: bridge::FailBridge,
    ) -> Result<bridge::FailBridgeOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        match close(&mut store, &input.bridge_id, input.cause) {
            Ok(()) => Ok(bridge::FailBridgeOutcome::Failed {
                bridge_failed: bridge::BridgeFailed {
                    bridge_id: input.bridge_id,
                    session_id: input.session_id,
                    cause: input.cause,
                    reason: input.reason,
                },
            }),
            Err(state) => Ok(bridge::FailBridgeOutcome::WrongState {
                error: state.map_or_else(no_such_bridge, |state| bridge::BridgeStateConflict {
                    state,
                }),
            }),
        }
    }
}

impl obligations::CloseBridgeBehavior for Behaviour {
    fn close_bridge(
        &mut self,
        input: bridge::CloseBridge,
    ) -> Result<bridge::CloseBridgeOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        match close(&mut store, &input.bridge_id, input.cause) {
            Ok(()) => Ok(bridge::CloseBridgeOutcome::Closed {
                bridge_closed: bridge::BridgeClosed {
                    bridge_id: input.bridge_id,
                    session_id: input.session_id,
                    cause: input.cause,
                    reason: input.reason,
                },
            }),
            Err(state) => Ok(bridge::CloseBridgeOutcome::WrongState {
                error: state.map_or_else(no_such_bridge, |state| bridge::BridgeStateConflict {
                    state,
                }),
            }),
        }
    }
}

impl obligations::BridgeByIdQuery for Behaviour {
    fn bridge_by_id(&self) -> Result<Vec<bridge::BridgeById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .bridges
            .iter()
            .map(|b| bridge::BridgeById {
                bridge_id: b.data.bridge_id.clone(),
                session_id: b.data.session_id.clone(),
                endpoint: b.data.endpoint.clone(),
                local_sdp: b.data.local_sdp.clone(),
                remote_sdp: b.data.remote_sdp.clone(),
                cause: b.data.cause,
                state: b.state,
            })
            .collect())
    }
}

impl obligations::LiveBridgesQuery for Behaviour {
    fn live_bridges(&self) -> Result<Vec<bridge::LiveBridges>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .bridges
            .iter()
            .filter(|b| b.state == bridge::BridgeSessionState::Live)
            .map(|b| bridge::LiveBridges {
                bridge_id: b.data.bridge_id.clone(),
                session_id: b.data.session_id.clone(),
                endpoint: b.data.endpoint.clone(),
                local_sdp: b.data.local_sdp.clone(),
                remote_sdp: b.data.remote_sdp.clone(),
                cause: b.data.cause,
                state: b.state,
            })
            .collect())
    }
}
