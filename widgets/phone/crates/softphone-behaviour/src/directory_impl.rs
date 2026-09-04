//! `softphone.directory` — the phonebook.

use softphone_types::directory::{self, obligations};
use softphone_types::obligation::UnmetObligation;

use crate::Behaviour;

fn stamp_contact<S: directory::contact_state::Marker>(
    contact: directory::Contact<S>,
) -> directory::ContactSnapshot {
    directory::ContactSnapshot {
        state: S::STATE,
        data: contact.into_data(),
    }
}

fn stamp_address<S: directory::contact_address_state::Marker>(
    address: directory::ContactAddress<S>,
) -> directory::ContactAddressSnapshot {
    directory::ContactAddressSnapshot {
        state: S::STATE,
        data: address.into_data(),
    }
}

impl obligations::AddContactBehavior for Behaviour {
    fn add_contact(
        &mut self,
        input: directory::AddContact,
    ) -> Result<directory::AddContactOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let contact_id = directory::ContactId(store.mint());
        store
            .contacts
            .push(stamp_contact(directory::Contact::new(
                directory::ContactData {
                    contact_id: contact_id.clone(),
                    display_name: input.display_name.clone(),
                },
            )));
        Ok(directory::AddContactOutcome::Added {
            contact_added: directory::ContactAdded {
                contact_id,
                display_name: input.display_name,
            },
        })
    }
}

impl obligations::RenameContactBehavior for Behaviour {
    fn rename_contact(
        &mut self,
        input: directory::RenameContact,
    ) -> Result<directory::RenameContactOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        if let Some(contact) = store
            .contacts
            .iter_mut()
            .find(|c| c.data.contact_id == input.contact_id)
        {
            contact.data.display_name = input.display_name.clone();
        }
        Ok(directory::RenameContactOutcome::Renamed {
            contact_renamed: directory::ContactRenamed {
                contact_id: input.contact_id,
                display_name: input.display_name,
            },
        })
    }
}

impl obligations::AddAddressBehavior for Behaviour {
    fn add_address(
        &mut self,
        input: directory::AddAddress,
    ) -> Result<directory::AddAddressOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let address_id = directory::ContactAddressId(store.mint());
        store
            .addresses
            .push(stamp_address(directory::ContactAddress::new(
                directory::ContactAddressData {
                    address_id: address_id.clone(),
                    contact_id: input.contact_id.clone(),
                    address: input.address.clone(),
                    label: input.label.clone(),
                },
            )));
        Ok(directory::AddAddressOutcome::Added {
            contact_address_added: directory::ContactAddressAdded {
                address_id,
                contact_id: input.contact_id,
                address: input.address,
                label: input.label,
            },
        })
    }
}

impl obligations::RemoveAddressBehavior for Behaviour {
    fn remove_address(
        &mut self,
        input: directory::RemoveAddress,
    ) -> Result<directory::RemoveAddressOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .addresses
            .iter()
            .position(|a| a.data.address_id == input.address_id)
        else {
            return Ok(directory::RemoveAddressOutcome::WrongState {
                error: directory::ContactAddressStateConflict {
                    state: directory::ContactAddressState::Removed,
                },
            });
        };
        match store.addresses.remove(index).refine() {
            directory::AnyContactAddress::Active(active) => {
                store.addresses.insert(index, stamp_address(active.remove()));
                Ok(directory::RemoveAddressOutcome::Removed {
                    contact_address_removed: directory::ContactAddressRemoved {
                        address_id: input.address_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.addresses.insert(index, other.snapshot());
                Ok(directory::RemoveAddressOutcome::WrongState {
                    error: directory::ContactAddressStateConflict { state },
                })
            }
        }
    }
}

impl obligations::DeleteContactBehavior for Behaviour {
    fn delete_contact(
        &mut self,
        input: directory::DeleteContact,
    ) -> Result<directory::DeleteContactOutcome, UnmetObligation> {
        let mut store = self.store.borrow_mut();
        let Some(index) = store
            .contacts
            .iter()
            .position(|c| c.data.contact_id == input.contact_id)
        else {
            return Ok(directory::DeleteContactOutcome::WrongState {
                error: directory::ContactStateConflict {
                    state: directory::ContactState::Deleted,
                },
            });
        };
        match store.contacts.remove(index).refine() {
            directory::AnyContact::Active(active) => {
                store.contacts.insert(index, stamp_contact(active.delete()));
                // The addresses go with it: `Contact owns ContactAddress`, which is what `owns`
                // means — the far side has no meaning without its owner, so a delete cannot leave
                // it behind. Which of the two a delete does is behaviour, and this is the choice.
                for address in &mut store.addresses {
                    if address.data.contact_id == input.contact_id
                        && address.state == directory::ContactAddressState::Active
                    {
                        *address = directory::ContactAddressSnapshot {
                            state: directory::ContactAddressState::Removed,
                            data: address.data.clone(),
                        };
                    }
                }
                Ok(directory::DeleteContactOutcome::Deleted {
                    contact_deleted: directory::ContactDeleted {
                        contact_id: input.contact_id,
                    },
                })
            }
            other => {
                let state = other.state();
                store.contacts.insert(index, other.snapshot());
                Ok(directory::DeleteContactOutcome::WrongState {
                    error: directory::ContactStateConflict { state },
                })
            }
        }
    }
}

impl obligations::ContactByIdQuery for Behaviour {
    fn contact_by_id(&self) -> Result<Vec<directory::ContactById>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .contacts
            .iter()
            .map(|c| directory::ContactById {
                contact_id: c.data.contact_id.clone(),
                display_name: c.data.display_name.clone(),
                state: c.state,
            })
            .collect())
    }
}

impl obligations::ContactsQuery for Behaviour {
    fn contacts(&self) -> Result<Vec<directory::Contacts>, UnmetObligation> {
        let store = self.store.borrow();
        Ok(store
            .contacts
            .iter()
            .filter(|c| c.state == directory::ContactState::Active)
            .map(|c| directory::Contacts {
                contact_id: c.data.contact_id.clone(),
                display_name: c.data.display_name.clone(),
                state: c.state,
            })
            .collect())
    }
}

impl obligations::ContactAddressesQuery for Behaviour {
    fn contact_addresses(&self) -> Result<Vec<directory::ContactAddresses>, UnmetObligation> {
        let store = self.store.borrow();
        // The view's filter is `contact_id == param.contact_id and state == Active`. The generated
        // query takes no parameter — the target's own weakening — so the half that can be applied
        // here is applied, and selecting the contact is the caller's.
        Ok(store
            .addresses
            .iter()
            .filter(|a| a.state == directory::ContactAddressState::Active)
            .map(|a| directory::ContactAddresses {
                address_id: a.data.address_id.clone(),
                contact_id: a.data.contact_id.clone(),
                address: a.data.address.clone(),
                label: a.data.label.clone(),
                state: a.state,
            })
            .collect())
    }
}
