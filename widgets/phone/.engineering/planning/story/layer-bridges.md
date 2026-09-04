---
format: aep.planning-md/1
id: story:layer-bridges
kind: story
status: implemented
title: Bridge the layers with checked relations
summary: Eight relations across seven domains, with the five refusal codes as evidence.
relations:
- decomposes: epic:browser-softphone
- depends_on: story:media-session-domain
- depends_on: story:signalling-over-wss
- depends_on: story:call-lifecycle
- depends_on: story:call-history
- depends_on: story:phonebook
revision: 5
---
# Bridge the layers with checked relations

## Outcome

The eight links between the seven domains are `relations:` entries, so `ess validate` refuses a
broken one. A layering described in prose is a layering nobody can check.

## Acceptance

All eight appear in the compiled IR under `entities.*.relations`, and each of the five refusal rules
fires when its defect is introduced. Both were run; the results are below.

## The eight

| # | Source | Kind / card. | Target | via |
|---|---|---|---|---|
| 1 | `control.Call` | references / one | `media.MediaSession` | `session_id` |
| 2 | `sip.SipDialog` | references / one | `media.MediaSession` | `session_id` |
| 3 | `local.LoopbackDevice` | references / one | `media.MediaSession` | `session_id` |
| 4 | `presentation.CallTile` | references / one | `control.Call` | `call_id` |
| 5 | `directory.Contact` | owns / many | `directory.ContactAddress` | `contact_id` |
| 6 | `history.CallRecord` | references / one | `control.Call` | `call_id` |
| 7 | `history.CallRecord` | references / one | `directory.Contact` | `contact_id` |
| 8 | `presentation.CallTile` | references / one | `directory.Contact` | `contact_id` |

Seven of the eight cross a domain boundary. That works because `build_registry` indexes every
domain's types into one registry and resolves each reference against it
(`crates/specify/ess-domain/src/system.rs:481-495`); nothing scopes resolution to a domain.

**Bridges 2 and 3 point the other way from how this story shipped.** They were
`media.MediaSession owns sip.SipDialog` and `owns local.LoopbackDevice`, which put the two binding
domains inside the one domain whose claim is that it names neither.
`architecture-decision-record:media-session-owns-nothing` records the flip and what `owns` bought
that `references` does not — and the two bindings `end-session-with-dialog` and
`end-session-with-loopback` are what hold it now.

Bridge 6's carrier is bare rather than `Optional`, because the three bindings that invoke
`RecordCall` map a bare `CallId` out of their events and ESS does not widen it:
`upstream-blocker:ess-mapping-does-not-widen-optional`.

## Evidence that the bridges are checked and not asserted

Five defects, one at a time, each in a throwaway copy of `ess/` under `$TMPDIR` — never the
committed tree. Every run exited 1.

| Defect | Code returned |
|---|---|
| misspell a relation `target` | `undeclared_reference` |
| `via` names a field that does not exist | `missing_declaration` |
| retype a `via` field to `String` | `type_mismatch` |
| a second `owns` targeting `SipDialog` | `conflicting_declaration` (with `type_mismatch`) |
| two relations carried by one field | `duplicate_declaration` (with `type_mismatch`) |

The five expected codes are the five rules in the relations design document §3. The two secondary
`type_mismatch` results are consequences of the same edit, not surprises: a stolen `owns` and a
re-pointed `via` both leave the carrier holding the wrong identity type.

## What no relation can hold

Two limits, both recorded in the domains rather than worked around.

- **"Exactly one of these two children exists" is not expressible.** Bridges 2 and 3 are both
  `owns`/one. Causation holds the correspondence instead — one `creates` outcome per binding.
- **A command in one domain causing a command in another is not modelled.** Nothing declares that a
  keypress in `softphone.presentation` eventually issues `softphone.control.Dial`. The relation is
  checked; the causation is not, and a component accepting a command another component owns the
  domain of is refused outright (`crates/specify/ess-domain/src/component.rs:921`).
