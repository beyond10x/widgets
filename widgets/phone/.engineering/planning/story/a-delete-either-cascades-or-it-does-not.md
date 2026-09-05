---
format: aep.planning-md/1
id: story:a-delete-either-cascades-or-it-does-not
kind: story
status: draft
title: A delete either cascades or it does not, and this repository says both
relations:
- decomposes: epic:browser-softphone
- informed_by: story:call-lifecycle
scope:
- confidence: cited
  path: crates/softphone-behaviour/src/directory_impl.rs
- confidence: cited
  path: scenarios/phonebook-contact-and-address.yaml
revision: 2
---
# A delete either cascades or it does not, and this repository says both

## What was measured

`story:call-lifecycle`'s implementor found it while probing a compound view filter, and measured it
directly (`probe-contact-addresses.mjs`): after `softphone.directory.DeleteContact`, the view
`softphone.directory.ContactAddresses` holds `{"rows":[]}`; before it, one row.

Two statements in this repository contradict each other, and both are load-bearing prose somebody
wrote on purpose:

**`scenarios/phonebook-contact-and-address.yaml:1-3`** — "nothing here cascades. The addresses
survive their contact, pointing at a `Deleted` one."

**`crates/softphone-behaviour/src/directory_impl.rs:157-163`** — moves every owned address out of
`Active` when the contact is deleted, with the comment "`Contact owns ContactAddress` … a delete
cannot leave it behind. Which of the two a delete does is behaviour, and this is the choice."

The scenario has **no `assert:` block**, which is why the contradiction survived: nothing in the
suite reads the addresses after the delete, so both statements passed the gate.

## Which is likely right, marked as a reading rather than a decision

`ess/domains/directory.yaml` declares `Contact owns ContactAddress`, and `owns` in `ess/1` means the
far side has no meaning without its owner. On that reading the implementation is right and the
scenario's opening comment is stale — it predates the relation being written as `owns`. **This is an
inference from the relation's semantics, not something anybody recorded**, which is exactly why it
needs a person: `ess validate` refuses a second owner and says nothing about what a delete does,
and the domain file itself notes that what the delete *does* is a command outcome rather than a
field of the relation.

## Acceptance

One of the two statements is corrected, and the scenario gains the `assert:` block that pins the
answer — after `DeleteContact`, `ContactAddresses` either holds the row or does not, asserted.
Whichever way it goes, the other statement is rewritten rather than left standing.

## Scope

Derived by the coordinator from `story:call-lifecycle`'s correction report, confidence **high** —
both statements were read and the behaviour was measured.

- `scenarios/phonebook-contact-and-address.yaml` — cited
- `crates/softphone-behaviour/src/directory_impl.rs` — cited, only if the implementation is the half
  that changes
- **Would collide with** any unit touching `scenarios/` or `crates/softphone-behaviour`.
