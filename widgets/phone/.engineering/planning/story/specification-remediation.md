---
format: aep.planning-md/1
id: story:specification-remediation
kind: story
status: implemented
title: Make the specification say only true things, and use the constructs it has
summary: sets, bindings, invariants, view params and filters, refs, topology; and four false claims corrected.
relations:
- decomposes: epic:browser-softphone
- depends_on: story:layer-bridges
revision: 4
---
# Make the specification say only true things, and use the constructs it has

## Outcome

The seven domains use the ESS constructs they were missing, and every claim written in them is
checkable. Two independent reviews — one on the phone as a product, one on the ESS usage — returned
21 and 16 findings against a specification that validated, compiled and synthesised
byte-deterministically. Neither disputed the layering; both found the same root cause from opposite
sides.

**`ess validate` checks that references resolve and that types match. It does not check that a
declared field can ever be written, that an event has a reader, or that a comment is true.**

## Acceptance

All assertions read from the compiled IR, not the source:

| Claim | Before | After |
|---|---|---|
| `creates`/`updates` outcomes writing no field | 33 | **0** |
| outcomes carrying `sets:` | 0 | 16 |
| `bindings:` — events with a reader | 0 | 7 |
| byte-identical view pairs | 4 | **0** |
| views with `params`, `filter` or `order_by` | 0 | 16 of 16 have at least one |
| `invariants:` | 0 | 3 on entities, 4 on a type |
| components under "components that run nowhere" | 7 | **0** |
| `refs:` carrying provenance a projection reads | 0 | 13 |
| `Human` / `Agent` grants identical | yes | yes, 7 commands each |
| generated Rust builds | clean | clean, `Finished release profile in 1.21s` |

`ess validate --path ess` → `softphone v1 — 10 file(s), valid`. Two `synthesize` runs to different
directories diff clean.

## What was wrong, and is not now

- **`Call.session_id` had no writer.** Bridge 1 was a relation nothing could establish, so no call
  could be joined to its media. `AttachMedia` writes it.
- **Nothing on the network could end a call.** `Kernel` held only `OfferCall`, `RingCall` and
  `ConfirmAnswer`, so a cancelled inbound call rang until a person hung up a call already gone.
  `FailCall` carries one of six `EndCause` words.
- **`Active` was asserted a step too early.** `Answer` moved straight to `Active`, where the contract
  requires the `RTCPeerConnection` be connected first and a failure be "presented as `failed`, not
  as connected". `Answering` is that step, and only `MediaConnected` leaves it.
- **`Call.direction` was undetermined.** An implementation recording every call `Inbound` conformed.
  `Dial` and `OfferCall` now set it as an enum literal.
- **The offer/answer exchange had no home.** Four commands in `softphone.sip` and two descriptions on
  the dialog.
- **`Registration.Failed` was terminal.** One transient WSS drop killed the registration for good.
  `RetryRegistration` is the way back, and `expires` exists at all now.
- **A ringing call could not show itself.** `softphone.presentation` grants only `Human` and
  `ShowCall` was in that grant alone. `show-incoming-call` is the binding.
- **Four claims in the YAML were false**, one of them the acceptance statement of a story already at
  `implemented`. All four are corrected, and the media-neutrality claim is now an IR query rather
  than a `grep` that returned five comment lines.

## Three ESS defects found doing it

Each is filed, worked around, and the workaround is written where somebody will read it.

| Blocker | What it is |
|---|---|
| `upstream-blocker:ess-system-level-type-breaks-synthesis` | a system-level `types:` entry validates and panics all three synthesis targets |
| `upstream-blocker:ess-mapping-does-not-widen-optional` | a binding mapping `T` into `Optional<T>` validates and emits Rust that does not compile |
| `upstream-blocker:ess-web-target-redeliver` | the web target calls a `redeliver` method the rust target does not generate; still present at 0.14.0 |

## Two limits that are not defects

- **An invariant reads only its own entity.** So "exactly one of these two children exists" is not
  expressible; causation holds it — one `creates` outcome per binding.
- **A binding maps only from its event's fields or a literal.** No lookup, no clock. That is why
  `RecordCall` takes a call id and reads the rest, why `ShowCall` lost its `contact_id` input in
  favour of `AttributeTile`, and why no binding can move `Console`: no control event carries a
  console id.
