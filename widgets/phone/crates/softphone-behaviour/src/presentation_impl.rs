//! `softphone.presentation` — what is on screen, which is not the call.

use softphone_types::obligation::UnmetObligation;
use softphone_types::presentation::{self, obligations};

use crate::Behaviour;

fn stamp_console<S: presentation::console_state::Marker>(
    console: presentation::Console<S>,
) -> presentation::ConsoleSnapshot {
    presentation::ConsoleSnapshot {
        state: S::STATE,
        data: console.into_data(),
    }
}

fn stamp_tile<S: presentation::call_tile_state::Marker>(
    tile: presentation::CallTile<S>,
) -> presentation::CallTileSnapshot {
    presentation::CallTileSnapshot {
        state: S::STATE,
        data: tile.into_data(),
    }
}

fn no_such_console() -> presentation::ConsoleStateConflict {
    // `Console` has no terminal state — a screen is never done — so the refusal for a console
    // nobody opened names `Idle`, the state it would have rested in.
    presentation::ConsoleStateConflict {
        state: presentation::ConsoleState::Idle,
    }
}

impl obligations::OpenConsoleBehavior for Behaviour {
    fn open_console(
        &mut self,
        _input: presentation::OpenConsole,
    ) -> Result<presentation::OpenConsoleOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let console_id = presentation::ConsoleId(store.mint());
        store
            .consoles
            .push(stamp_console(presentation::Console::new(
                presentation::ConsoleData {
                    console_id: console_id.clone(),
                    active_tile: None,
                },
            )));
        Ok(presentation::OpenConsoleOutcome::Opened {
            console_opened: presentation::ConsoleOpened { console_id },
        })
    }
}

impl obligations::EnterDialingBehavior for Behaviour {
    fn enter_dialing(
        &mut self,
        input: presentation::EnterDialing,
    ) -> Result<presentation::EnterDialingOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .consoles
            .iter()
            .position(|c| c.data.console_id == input.console_id)
        else {
            return Ok(presentation::EnterDialingOutcome::WrongState {
                error: no_such_console(),
            });
        };
        match store.consoles.remove(index).refine() {
            presentation::AnyConsole::Idle(idle) => {
                let mut moved = stamp_console(idle.dial());
                moved.data.active_tile = Some(input.tile_id.clone());
                store.consoles.insert(index, moved);
                Ok(presentation::EnterDialingOutcome::Dialing {
                    console_dialing: presentation::ConsoleDialing {
                        console_id: input.console_id,
                        tile_id: input.tile_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.consoles.insert(index, other.snapshot());
                Ok(presentation::EnterDialingOutcome::WrongState {
                    error: presentation::ConsoleStateConflict { state },
                })
            }
        }
    }
}

impl obligations::EnterIncomingBehavior for Behaviour {
    fn enter_incoming(
        &mut self,
        input: presentation::EnterIncoming,
    ) -> Result<presentation::EnterIncomingOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .consoles
            .iter()
            .position(|c| c.data.console_id == input.console_id)
        else {
            return Ok(presentation::EnterIncomingOutcome::WrongState {
                error: no_such_console(),
            });
        };
        match store.consoles.remove(index).refine() {
            presentation::AnyConsole::Idle(idle) => {
                let mut moved = stamp_console(idle.offer());
                moved.data.active_tile = Some(input.tile_id.clone());
                store.consoles.insert(index, moved);
                Ok(presentation::EnterIncomingOutcome::Incoming {
                    console_incoming: presentation::ConsoleIncoming {
                        console_id: input.console_id,
                        tile_id: input.tile_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.consoles.insert(index, other.snapshot());
                Ok(presentation::EnterIncomingOutcome::WrongState {
                    error: presentation::ConsoleStateConflict { state },
                })
            }
        }
    }
}

impl obligations::EnterCallBehavior for Behaviour {
    fn enter_call(
        &mut self,
        input: presentation::EnterCall,
    ) -> Result<presentation::EnterCallOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .consoles
            .iter()
            .position(|c| c.data.console_id == input.console_id)
        else {
            return Ok(presentation::EnterCallOutcome::WrongState {
                error: no_such_console(),
            });
        };
        match store.consoles.remove(index).refine() {
            presentation::AnyConsole::Dialing(dialing) => {
                store.consoles.insert(index, stamp_console(dialing.engage()));
            }
            presentation::AnyConsole::Incoming(incoming) => {
                store
                    .consoles
                    .insert(index, stamp_console(incoming.engage()));
            }
            other => {
                let state = other.state();
                store.consoles.insert(index, other.snapshot());
                return Ok(presentation::EnterCallOutcome::WrongState {
                    error: presentation::ConsoleStateConflict { state },
                });
            }
        }
        Ok(presentation::EnterCallOutcome::Engaged {
            console_engaged: presentation::ConsoleEngaged {
                console_id: input.console_id,
            },
        })
    }
}

impl obligations::LeaveCallBehavior for Behaviour {
    fn leave_call(
        &mut self,
        input: presentation::LeaveCall,
    ) -> Result<presentation::LeaveCallOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .consoles
            .iter()
            .position(|c| c.data.console_id == input.console_id)
        else {
            return Ok(presentation::LeaveCallOutcome::WrongState {
                error: no_such_console(),
            });
        };
        let mut moved = match store.consoles.remove(index).refine() {
            presentation::AnyConsole::Dialing(dialing) => stamp_console(dialing.release()),
            presentation::AnyConsole::Incoming(incoming) => stamp_console(incoming.release()),
            presentation::AnyConsole::InCall(in_call) => stamp_console(in_call.release()),
            other => {
                let state = other.state();
                store.consoles.insert(index, other.snapshot());
                return Ok(presentation::LeaveCallOutcome::WrongState {
                    error: presentation::ConsoleStateConflict { state },
                });
            }
        };
        // Back to `Idle` is back to no tile in front: leaving a call and still pointing at its
        // tile would be a screen claiming a call that is gone.
        moved.data.active_tile = None;
        store.consoles.insert(index, moved);
        Ok(presentation::LeaveCallOutcome::Released {
            console_released: presentation::ConsoleReleased {
                console_id: input.console_id,
            },
        })
    }
}

impl obligations::OpenKeypadBehavior for Behaviour {
    fn open_keypad(
        &mut self,
        _input: presentation::OpenKeypad,
    ) -> Result<presentation::OpenKeypadOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let keypad_id = presentation::KeypadId(store.mint());
        store.keypads.push(stamp_keypad(presentation::Keypad::new(
            presentation::KeypadData {
                keypad_id: keypad_id.clone(),
                entry: String::new(),
            },
        )));
        Ok(presentation::OpenKeypadOutcome::Opened {
            keypad_opened: presentation::KeypadOpened { keypad_id },
        })
    }
}

fn stamp_keypad<S: presentation::keypad_state::Marker>(
    keypad: presentation::Keypad<S>,
) -> presentation::KeypadSnapshot {
    presentation::KeypadSnapshot {
        state: S::STATE,
        data: keypad.into_data(),
    }
}

impl obligations::PressKeyBehavior for Behaviour {
    fn press_key(
        &mut self,
        input: presentation::PressKey,
    ) -> Result<presentation::PressKeyOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        // `updates` with no `sets:`, and the summary says why: what a keypress does to a
        // half-typed number is behaviour. It appends.
        if let Some(keypad) = store
            .keypads
            .iter_mut()
            .find(|k| k.data.keypad_id == input.keypad_id)
        {
            keypad.data.entry.push_str(&input.key);
        }
        Ok(presentation::PressKeyOutcome::Pressed {
            key_pressed: presentation::KeyPressed {
                keypad_id: input.keypad_id,
                key: input.key,
            },
        })
    }
}

impl obligations::ClearEntryBehavior for Behaviour {
    fn clear_entry(
        &mut self,
        input: presentation::ClearEntry,
    ) -> Result<presentation::ClearEntryOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        if let Some(keypad) = store
            .keypads
            .iter_mut()
            .find(|k| k.data.keypad_id == input.keypad_id)
        {
            keypad.data.entry.clear();
        }
        Ok(presentation::ClearEntryOutcome::Cleared {
            entry_cleared: presentation::EntryCleared {
                keypad_id: input.keypad_id,
            },
        })
    }
}

impl obligations::ShowCallBehavior for Behaviour {
    fn show_call(
        &mut self,
        input: presentation::ShowCall,
    ) -> Result<presentation::ShowCallOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let tile_id = presentation::CallTileId(store.mint());
        store.tiles.push(stamp_tile(presentation::CallTile::new(
            presentation::CallTileData {
                tile_id: tile_id.clone(),
                call_id: input.call_id.clone(),
                // Nothing is resolved here. A tile knows which call it is for; whose number it is
                // arrives later, through `AttributeTile`.
                contact_id: None,
            },
        )));
        Ok(presentation::ShowCallOutcome::Shown {
            call_shown: presentation::CallShown {
                tile_id,
                call_id: input.call_id,
            },
        })
    }
}

impl obligations::AttributeTileBehavior for Behaviour {
    fn attribute_tile(
        &mut self,
        input: presentation::AttributeTile,
    ) -> Result<presentation::AttributeTileOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        if let Some(tile) = store
            .tiles
            .iter_mut()
            .find(|t| t.data.tile_id == input.tile_id)
        {
            tile.data.contact_id = input.contact_id.clone();
        }
        Ok(presentation::AttributeTileOutcome::Attributed {
            tile_attributed: presentation::TileAttributed {
                tile_id: input.tile_id,
                contact_id: input.contact_id,
            },
        })
    }
}

impl obligations::DismissCallBehavior for Behaviour {
    fn dismiss_call(
        &mut self,
        input: presentation::DismissCall,
    ) -> Result<presentation::DismissCallOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .tiles
            .iter()
            .position(|t| t.data.tile_id == input.tile_id)
        else {
            return Ok(presentation::DismissCallOutcome::WrongState {
                error: presentation::CallTileStateConflict {
                    state: presentation::CallTileState::Dismissed,
                },
            });
        };
        match store.tiles.remove(index).refine() {
            presentation::AnyCallTile::Shown(shown) => {
                store.tiles.insert(index, stamp_tile(shown.dismiss()));
                Ok(presentation::DismissCallOutcome::Dismissed {
                    call_dismissed: presentation::CallDismissed {
                        tile_id: input.tile_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.tiles.insert(index, other.snapshot());
                Ok(presentation::DismissCallOutcome::WrongState {
                    error: presentation::CallTileStateConflict { state },
                })
            }
        }
    }
}

impl obligations::ConsoleByIdQuery for Behaviour {
    fn console_by_id(&self) -> Result<Vec<presentation::ConsoleById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .consoles
            .iter()
            .map(|c| presentation::ConsoleById {
                console_id: c.data.console_id.clone(),
                active_tile: c.data.active_tile.clone(),
                state: c.state,
            })
            .collect())
    }
}

impl obligations::KeypadByIdQuery for Behaviour {
    fn keypad_by_id(&self) -> Result<Vec<presentation::KeypadById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .keypads
            .iter()
            .map(|k| presentation::KeypadById {
                keypad_id: k.data.keypad_id.clone(),
                entry: k.data.entry.clone(),
                state: k.state,
            })
            .collect())
    }
}

impl obligations::CallTilesQuery for Behaviour {
    fn call_tiles(&self) -> Result<Vec<presentation::CallTiles>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .tiles
            .iter()
            .filter(|t| t.state == presentation::CallTileState::Shown)
            .map(|t| presentation::CallTiles {
                tile_id: t.data.tile_id.clone(),
                call_id: t.data.call_id.clone(),
                contact_id: t.data.contact_id.clone(),
                state: t.state,
            })
            .collect())
    }
}
