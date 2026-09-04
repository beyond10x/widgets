//! `softphone.history` — the log of calls that finished.

use softphone_types::history::{self, obligations};
use softphone_types::obligation::UnmetObligation;
use softphone_types::{control, primitives};

use crate::Behaviour;

fn stamp<S: history::call_record_state::Marker>(
    record: history::CallRecord<S>,
) -> history::CallRecordSnapshot {
    history::CallRecordSnapshot {
        state: S::STATE,
        data: record.into_data(),
    }
}

impl obligations::RecordCallBehavior for Behaviour {
    /// The one behaviour here that decides more than it is told.
    ///
    /// `RecordCall` carries a call id and the outcome sets that one field, so everything else on
    /// the record — direction, address, when it started, when it was answered, when and how it
    /// ended — comes from what this crate kept while the call was live (`Store::timings`,
    /// `Store::endings`). That is the shape of the obligation: the specification says a record
    /// exists and what it holds, and where the values come from is storage.
    ///
    /// A call this store never saw is the one case with no honest answer, because the outcome
    /// declares no refusal. It is recorded with an empty address and `Local`, which is visible as
    /// wrong rather than plausible.
    fn record_call(
        &mut self,
        input: history::RecordCall,
    ) -> Result<history::RecordCallOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let record_id = history::CallRecordId(store.mint());
        let call = store
            .calls
            .iter()
            .find(|c| c.data.call_id == input.call_id)
            .map(|c| (c.data.direction.clone(), c.data.remote.clone()));
        let timing = store
            .timings
            .iter()
            .find(|t| t.call == input.call_id)
            .cloned();
        let ending = store
            .endings
            .iter()
            .find(|e| e.call == input.call_id)
            .cloned();
        let now = store.tick();
        let (direction, remote) = call.unwrap_or((
            control::CallDirection::Outbound,
            control::RemoteAddress(String::new()),
        ));
        store
            .records
            .push(stamp(history::CallRecord::new(history::CallRecordData {
                record_id: record_id.clone(),
                call_id: input.call_id.clone(),
                // Attribution is a separate command: who a number belongs to can change after the
                // call, so nothing is resolved here.
                contact_id: None,
                direction,
                remote,
                started_at: timing
                    .as_ref()
                    .map_or_else(|| now.clone(), |t| t.started_at.clone()),
                ended_at: ending
                    .as_ref()
                    .map_or_else(|| now.clone(), |e| e.ended_at.clone()),
                answered_at: timing.and_then(|t| t.answered_at),
                termination: ending
                    .as_ref()
                    .map_or(control::EndCause::Local, |e| e.cause.clone()),
                status: ending.and_then(|e| e.status),
            })));
        Ok(history::RecordCallOutcome::Recorded {
            call_recorded: history::CallRecorded {
                record_id,
                call_id: input.call_id,
            },
        })
    }
}

impl obligations::AttributeRecordBehavior for Behaviour {
    fn attribute_record(
        &mut self,
        input: history::AttributeRecord,
    ) -> Result<history::AttributeRecordOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        if let Some(record) = store
            .records
            .iter_mut()
            .find(|r| r.data.record_id == input.record_id)
        {
            record.data.contact_id = input.contact_id.clone();
        }
        Ok(history::AttributeRecordOutcome::Attributed {
            record_attributed: history::RecordAttributed {
                record_id: input.record_id,
                contact_id: input.contact_id,
            },
        })
    }
}

impl obligations::DeleteRecordBehavior for Behaviour {
    fn delete_record(
        &mut self,
        input: history::DeleteRecord,
    ) -> Result<history::DeleteRecordOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .records
            .iter()
            .position(|r| r.data.record_id == input.record_id)
        else {
            return Ok(history::DeleteRecordOutcome::WrongState {
                error: history::CallRecordStateConflict {
                    state: history::CallRecordState::Deleted,
                },
            });
        };
        match store.records.remove(index).refine() {
            history::AnyCallRecord::Recorded(recorded) => {
                store.records.insert(index, stamp(recorded.delete()));
                Ok(history::DeleteRecordOutcome::Deleted {
                    record_deleted: history::RecordDeleted {
                        record_id: input.record_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.records.insert(index, other.snapshot());
                Ok(history::DeleteRecordOutcome::WrongState {
                    error: history::CallRecordStateConflict { state },
                })
            }
        }
    }
}

/// Every field of a record, in the order the row declares them.
fn fields(
    snapshot: &history::CallRecordSnapshot,
) -> (
    history::CallRecordId,
    control::CallId,
    Option<softphone_types::directory::ContactId>,
    control::CallDirection,
    control::RemoteAddress,
    primitives::Timestamp,
    primitives::Timestamp,
    Option<primitives::Timestamp>,
    control::EndCause,
    Option<control::RejectStatus>,
    history::CallRecordState,
) {
    (
        snapshot.data.record_id.clone(),
        snapshot.data.call_id.clone(),
        snapshot.data.contact_id.clone(),
        snapshot.data.direction.clone(),
        snapshot.data.remote.clone(),
        snapshot.data.started_at.clone(),
        snapshot.data.ended_at.clone(),
        snapshot.data.answered_at.clone(),
        snapshot.data.termination.clone(),
        snapshot.data.status.clone(),
        snapshot.state,
    )
}

impl obligations::CallRecordByIdQuery for Behaviour {
    fn call_record_by_id(&self) -> Result<Vec<history::CallRecordById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .records
            .iter()
            .map(|r| {
                let f = fields(r);
                history::CallRecordById {
                    record_id: f.0,
                    call_id: f.1,
                    contact_id: f.2,
                    direction: f.3,
                    remote: f.4,
                    started_at: f.5,
                    ended_at: f.6,
                    answered_at: f.7,
                    termination: f.8,
                    status: f.9,
                    state: f.10,
                }
            })
            .collect())
    }
}

impl obligations::RecentCallsQuery for Behaviour {
    fn recent_calls(&self) -> Result<Vec<history::RecentCalls>, UnmetObligation> {
        let store = self.store.borrow();
        let mut rows: Vec<history::RecentCalls> = store
            .records
            .iter()
            .filter(|r| r.state == history::CallRecordState::Recorded)
            .map(|r| {
                let f = fields(r);
                history::RecentCalls {
                    record_id: f.0,
                    call_id: f.1,
                    contact_id: f.2,
                    direction: f.3,
                    remote: f.4,
                    started_at: f.5,
                    ended_at: f.6,
                    answered_at: f.7,
                    termination: f.8,
                    status: f.9,
                    state: f.10,
                }
            })
            .collect();
        // `order_by: ended_at desc`, which is part of the view rather than a presentation choice.
        rows.sort_by(|a, b| b.ended_at.cmp(&a.ended_at));
        Ok(rows)
    }
}
