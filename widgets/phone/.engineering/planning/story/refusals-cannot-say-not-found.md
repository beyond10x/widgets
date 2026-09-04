---
format: aep.planning-md/1
id: story:refusals-cannot-say-not-found
kind: story
status: draft
title: A refusal cannot say "no such instance", and two things rest on that
relations:
- decomposes: epic:browser-softphone
- informed_by: review-result:adversary-call-lifecycle-pass-1
scope:
- confidence: cited
  path: crates/softphone-behaviour/src/bridge_impl.rs
- confidence: cited
  path: crates/softphone-behaviour/src/control_impl.rs
- confidence: cited
  path: crates/softphone-behaviour/src/media_impl.rs
- confidence: inferred
  path: ess
revision: 3
---
# A refusal cannot say "no such instance", and two things rest on that

## What was measured

`adp:adversary`, pass 1 on `story:call-lifecycle`, found that a build whose refusals **delete** the
entity instead of restoring it passes all 14 scenarios and all 257 steps. Two facts combine to make
that invisible, and neither is fixable inside that story's surface.

**One — the store's own answer for a missing instance is a lie by necessity.**
`crates/softphone-behaviour/src/control_impl.rs:19` answers a command naming a call this store never
held with `state: Ended`, byte-identical to the genuine terminal refusal. `bridge_impl.rs:17` and
`media_impl.rs:61,99` do the same for their entities. The reason is in the code's own comment: the
model declares **no not-found outcome**, and a lifecycle has no value meaning *absent*, so every
possible answer misstates something. `Ended` was chosen because no declared move starts from a
terminal state, which keeps the refusal coherent with the state it reports.

**Two — a parameterised view's query cannot filter.**
`softphone.control.CallById` and `softphone.bridge.BridgeById` declare
`filter: <id> == param.<id>`, and the generated obligation is `fn call_by_id(&self) -> Vec<CallById>`
— **no parameter**. So `crates/softphone-behaviour/src/control_impl.rs:444` and
`bridge_impl.rs:168` return every row, and `projected()` calls them with nothing. A `counts:` claim
on such a view is therefore not a claim about one instance.

That second one is the web target's declared weakening — "the synthesised system holds no entity
store … so the page shows each declared view's rows" — reaching further than the page: it reaches
any test that wants to assert one instance.

## What would close each

**The not-found case** wants a declared outcome. Either every command that names an instance gains a
`not_found` branch in `ess/`, which is 26 commands and a large diff, or the specification states that
naming a missing instance is out of contract and the store may answer any refusal — in which case
nothing may assert on the reported state, and that is worth writing down where the next reader finds
it.

**The unfiltered query** wants either a parameterised query obligation upstream in ESS, or an
application-side query surface beside the generated one that takes the parameter. The second is a
day's work here; the first is an ESS story that has not been written.

## Acceptance

Whichever is chosen, a test can distinguish *refused because terminal* from *refused because the
instance is gone*, and the mutant the adversary built goes red.

## Scope

Derived by the coordinator from `review-result:adversary-call-lifecycle-pass-1`, confidence
**high** — every line was read from the tree at `eb40ddc`.

- `ess/` — inferred, and the larger of the two options
- `crates/softphone-behaviour/src/control_impl.rs`, `bridge_impl.rs`, `media_impl.rs` — cited, the
  four `no_such_*` helpers and the two unfiltered `*_by_id` queries
- **Would collide with** any unit touching `ess/` or `crates/softphone-behaviour`.
