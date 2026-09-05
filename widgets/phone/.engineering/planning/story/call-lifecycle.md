---
format: aep.planning-md/1
id: story:call-lifecycle
kind: story
status: active
title: Place, answer and end calls
summary: Wire the softphone.control commands to kernel events in both directions.
relations:
- decomposes: epic:browser-softphone
- depends_on: story:signalling-over-wss
- depends_on: story:media-session-domain
scope:
- confidence: inferred
  path: ess/domains/control.yaml
- confidence: cited
  path: scenarios
- confidence: cited
  path: tests/suite.mjs
revision: 8
---
# Place, answer and end calls

## Outcome

`softphone.control` is the phone's whole surface, and a human and an agent drive it with the same
commands. No protocol type and no contact appears in it.

## Acceptance

`ess validate` accepts the domain. `Human`'s and `Agent`'s `may:` lists are identical — read from
the compiled IR, not the YAML — and the agent is granted nothing in `softphone.presentation`. A
command issued from a state its transition does not allow returns
`softphone.control.CallStateConflict` and appends nothing. Grep the domain for `sip`, `rtvbp`,
`webrtc` and `directory`: all empty.

## What was built

`ess/domains/control.yaml`. `PhoneEndpoint` — the phone itself, with a `default_binding` typed
`softphone.media.BindingKind`. `Call` — `endpoint_id`, `direction`, `remote`, an optional
`session_id`, `muted`; states `Requested → Ringing → Active → Ended`.

Ten commands, and three actors:

| Actor | Grants |
|---|---|
| `Human` | `ConfigureEndpoint`, `Dial`, `Answer`, `Reject`, `HangUp`, `SendDigits`, `SetMuted` |
| `Agent` | the same seven, byte-identical |
| `Kernel` | `OfferCall`, `RingCall`, `ConfirmAnswer` |

`Answer` and `ConfirmAnswer` take the same `activate` transition and are different facts: the first
is the local decision on an inbound call, the second is the far end answering an outbound one. Only
`Kernel` may assert the second, so a page or an agent cannot claim the far end picked up. `Reject`
and `HangUp` stand in the same relation over `end`.

`Call.session_id` is bridge 1 — `references`, not `owns`, because a media session is a thing in its
own right and a call that ends does not take it with it.

## The parity is the specification, and nothing enforces it

Web synthesis refuses actor grants: "a grant is checked against a caller identity, which types do
not carry, and enforcement belongs to the layer that knows who is calling". So `Human == Agent` is a
fact a reader and a checker of the IR can read, and a host obligation. It is not a generated check.

Verified from the IR rather than the source: an earlier check compared two absent keys and reported
a pass. Seven commands each, identical.

## Scope

Derived 2026-09-04 by `story-scoper`, confidence **high** — every clause was checked against the
compiled IR, the compiled suite and the implementation. Paths are relative to `widgets/phone`.

**Three of the four acceptance clauses are already satisfied**, by work recorded under
`story:fill-the-obligations`, and the evidence is in the tree rather than asserted:

- `ess validate --path ess` reports `softphone v1 — 11 file(s), valid` — cited
- `Human` and `Agent` carry the same eight commands in the compiled IR, and
  `softphone.presentation` declares no `Agent` actor at all — cited
- every wrong-state branch is implemented and appends nothing:
  `crates/softphone-behaviour/src/control_impl.rs:117,133,180,200,215,231,298,322`, and the wire
  emits `"published":[]` beside the refusal — cited
- all 14 `softphone.control` commands are exercised by the authored scenarios: 181 steps, 0
  failures — cited

**What remains is the clause nothing executes.**

| path | mark | why |
|---|---|---|
| `scenarios` | cited | the compiled suite contains **zero** wrong-state expectations, so `CallStateConflict` is implemented and unproven |
| `tests/suite.mjs` | cited | `:107` throws on any step kind outside the four it knows, and an authored `error:` claim compiles to an `ExpectError` step it does not |
| `ess/domains/control.yaml` | inferred | only on one of two resolutions of the grep clause: `sip` matches 10 lines today, including the `EndCause` variant `Sip`, and `directory` matches one comment. Either the spec drops the words or the acceptance is restated |

**The body is stale against the domain it describes** — four states where there are five, ten
commands where there are 14, seven grants where there are eight, three kernel grants where there
are six, and `Answer` takes `accept` rather than `activate`.

**Would collide with** any unit touching `scenarios/` or `tests/suite.mjs` — the executable
acceptance surface of the whole application, not this story's alone — and, on the spec resolution,
`ess/domains/control.yaml`.
