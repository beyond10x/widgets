---
format: aep.planning-md/1
id: review-result:adversary-call-lifecycle-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 — story:call-lifecycle: a mutant that deletes the entity passes every step'
relations:
- reviews: story:call-lifecycle
revision: 1
---
# Adversary pass 1 — `story:call-lifecycle`

Worktree `~/.local/state/worktree/trees/b10x/widgets/wave-call-lifecycle`, branch
`impl/call-lifecycle`, base `eb40ddc`, uncommitted. Agent `adp:adversary`.

```
verdict: NEEDS-CHANGE
cases: executed 257→257, red 2
origin: introduced 2 / pre-existing 6 / undecided 0
```

## The two cases it added, red on first execution

| file | asserts |
|---|---|
| `scenarios/a-refusal-leaves-the-call-where-it-was.yaml` | after the refused `Answer`, `CallById` holds exactly one row and it is `Ended` |
| `scenarios/a-refusal-leaves-the-bridge-where-it-was.yaml` | after the refused `ConfirmBridge`, `BridgeById` holds exactly one row and it is `Closed` |

Both compile (`16 authored scenario(s) from 16 file(s), 0 refusal(s)`) and both fail as
`step N (query_view): a step kind this runner does not know`.

## The blocker, in the shape it was measured

A mutant whose refusals **drop the entity** instead of restoring it — one `store.*.insert` line
removed from the `ConfirmBridge` arm of `bridge_impl.rs` and from `accept()` in `control_impl.rs` —
**passes all 14 scenarios and all 257 steps, exit 0.** Direct observation of the `views` the runner
discards: after the first refusal `CallById.rows=[]` and `BridgeById.rows=[]`, while every later
refusal still answers the identical `{"state":"Ended"}`.

Two mechanisms make it invisible. `expect_error` observes only the response, and `no_such_call()`
answers an instance the store never held with `state: Ended` — byte-identical to the genuine
terminal refusal.

## Attacked and could not break

- a refusal claiming the wrong state — `expect_error`'s field assertion catches it:
  `'state': expected "Ringing", got "Ended"`;
- the states the errors name are the states the instances are actually in, read from the `views`;
- a capture bound off a refusal that published nothing — the runner catches it;
- `last` staleness — no authored ordering leaves it pointing at a previous response;
- refusals that append — no `WrongState` arm in any of the nine `*_impl.rs` calls `store.ended`,
  `store.timing` or `store.tick`.

## Findings

```findings
- file: widgets/phone/tests/suite.mjs
  line: 104
  category: mutant
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: a build whose refusals delete the entity instead of restoring it passes all 14 scenarios and all 257 steps, because expect_error observes only the response and no_such_call/no_such_bridge answer a missing instance with the same terminal-state conflict a real one gets.
- file: widgets/phone/tests/suite.mjs
  line: 145
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: the runner throws on query_view and expect_view, the only step kinds that can state a claim about the store, even though every command response it already holds carries every view's rows under `views`.
- file: widgets/phone/tests/suite.mjs
  line: 117
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: all 14 expect_no_event steps and the published-non-empty guard are unfalsifiable, because the generated wire writes published as an empty literal on 22 of 22 wrong-state arms and any other outcome is caught first by expect_outcome.
- file: widgets/phone/crates/softphone-behaviour/src/control_impl.rs
  line: 19
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: pre-existing
  message: no_such_call returns state Ended for an instance the store never held, so the state the refusal names does not identify the state the instance is in and the unit's 12 assertions cannot tell the two cases apart.
- file: widgets/phone/crates/softphone-behaviour/src/bridge_impl.rs
  line: 168
  category: contract-drift
  severity: warning
  verdict: INFEASIBLE
  origin: pre-existing
  message: BridgeById and CallById declare a filter on their declared parameter and the query obligation returns every row, so a counts claim on a params-carrying view is not a claim about that instance.
- file: widgets/phone/scenarios
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: 12 of the 26 declared wrong-state branches are taken and 14 are not, and the coverage check that would report it was written by the implementor and left unlanded in the shared scratch root as prove-every-refusal.patch.
- file: widgets/phone/tests/suite.mjs
  line: 157
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: a failing scenario contributes zero steps to the executed total, so the step count the acceptance clause compares against 181 falls silently when a scenario breaks.
- file: widgets/phone/.engineering/planning/story/call-lifecycle.md
  category: acceptance
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: the acceptance clause requiring an empty grep for sip in the control domain cannot be satisfied without renaming the load-bearing EndCause variant Sip, which matches 10 lines today.
```
