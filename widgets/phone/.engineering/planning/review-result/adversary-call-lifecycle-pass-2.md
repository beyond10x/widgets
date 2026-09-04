---
format: aep.planning-md/1
id: review-result:adversary-call-lifecycle-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 — story:call-lifecycle: two mutants survive, and a true claim cannot match'
relations:
- reviews: story:call-lifecycle
revision: 1
---
# Adversary pass 2 — `story:call-lifecycle`

Worktree `~/.local/state/worktree/trees/b10x/widgets/wave-call-lifecycle`, HEAD `6c0a0e4` — which
moved from `2ff5513` mid-pass, because the coordinator committed the round-1 follow-on while the
adversary was working; the pass says so rather than reporting a base it did not have. Five untracked
scenario files added; no implementation file and no runner line touched. Agent `adp:adversary`.

```
verdict: NEEDS-CHANGE
cases: executed 304→398, red 1
origin: introduced 8 / pre-existing 1 / undecided 0
```

## Three blockers, and two of them are mutants that survive

**A true claim about a struct-valued field cannot match.** `matchesRow` compares
`JSON.stringify(want) !== JSON.stringify(got)`, and the two sides disagree about key order: the
compiled literal in `.build/suite.json` is key-sorted, and the wire writes declaration order
(`softphone-web/src/lib.rs`, the row encoder). ESS's own reference runner compares `Node::Map`
structurally (`ess-conformance/src/runner.rs:1660`), so the specification's semantics accept the row
this runner rejects. Red output, same values, different order:

```
`profile`: expected {"channels":1,"frame_bytes":320,"packet_time_ms":20,"sample_format":"PcmS16Le","sample_rate_hz":8000},
                got {"sample_format":"PcmS16Le","sample_rate_hz":8000,"channels":1,"packet_time_ms":20,"frame_bytes":320}
```

**Mutant M2 — deleting the refusal-restore lines in `bridge_impl.rs:112` and `media_impl.rs:124` —
passes all 16 scenarios, 304 of 304 steps, exit 0.** `refusals-on-a-closed-bridge.yaml` and
`refusals-on-a-repeated-session-move.yaml` each issue exactly that refusal and carry **no `assert:`
block at all**, and `no_such_bridge()` answers a missing bridge with `state: Closed`, so the
following step is answered identically either way.

**Mutant M3 — a `ConfirmBridge` refusal that restores the bridge *and* writes
`remote_sdp = input.answer` — passes all 16.** That is the exact invariant
`refusals-on-a-closed-bridge.yaml:5-7` says the refusal exists to protect, and the only store
assertion on that arm checks `state` and nothing else.

Pass 1's mutant was rebuilt and reproduces the correction's claim exactly: `293 of 304`, 3 failures.

## The five scenarios it added are the fix for the two mutants

Four are green as they stand and each goes red under the mutant it was written for; the fifth is red
only because of the key-order defect above. One of them, the compound-filter case, closes a path the
correction had verified by reading rather than by running: deleting the `param.` guard or collapsing
the remainder to the empty string left the suite green at 304 of 304, because no scenario read
`ContactAddresses` — the only view whose filter has more than one clause.

## Six more

A comment that says three scenarios carry a falsifiable `no_events:` where five assert refusals and
two carry only the unfalsifiable form; an unindented duplicate `assert:` header sitting between two
steps it does not describe; an `excludes` assertion that decides against zero rows, so the claim
beside it is carried entirely by the `counts` above it; a view the wire reports as `unmet` read as an
empty view by `held.rows ?? []`, which no obligation in this widget currently produces
(`INFEASIBLE`, the state was built); and the same key-order comparison in `matches()` for event
payloads, which reproduces at `eb40ddc` and is the one `pre-existing` finding.

## Attacked and could not break

Parameter stripping against every view in the catalogue — no param name is a substring of another
view's clause, an unbound parameter is caught, a param the row does not project is refused. The
claim that a non-parameter remainder is already applied on the way out: true for all six such views,
each named. The `at`/`ranked`/`satisfies` refusal: the catalogue publishes no `order_by`, 0
occurrences. `counts`/`contains`/`excludes` against ESS's reference runner: equivalent, including
`contains` failing on an empty view. Step counting: `293 of 304` under pass 1's mutant reproduces
exactly. Pass 1's two scenarios green and unweakened.

## Findings

```findings
- file: widgets/phone/tests/suite.mjs
  line: 65
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: matchesRow compares rows with JSON.stringify, so a contains claim on a struct-valued field never matches — the compiled literal is key-sorted and the wire writes declaration order, and ESS's own runner compares Node::Map structurally.
- file: crates/softphone-behaviour/src/bridge_impl.rs
  line: 112
  category: mutant
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: deleting the refusal-restore lines in close() and media_impl terminate() leaves all 16 scenarios green at 304 of 304 steps, because refusals-on-a-closed-bridge.yaml and refusals-on-a-repeated-session-move.yaml assert refusals and carry no assert block at all.
- file: crates/softphone-behaviour/src/bridge_impl.rs
  line: 84
  category: mutant
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a ConfirmBridge refusal that restores the bridge and writes remote_sdp — the exact invariant refusals-on-a-closed-bridge.yaml says it guards — passes all 16 scenarios, because the only store assertion on that arm checks state and nothing else.
- file: widgets/phone/tests/suite.mjs
  line: 110
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the compound-filter half of rowsFor has zero executed coverage — its guard can be deleted or its remainder collapsed to the empty string with the suite unchanged at 16 scenarios and 304 of 304 steps, because no scenario reads ContactAddresses.
- file: widgets/phone/tests/suite.mjs
  line: 196
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the comment says three scenarios assert refusals and each puts a falsifiable no_events on a publishing command, but five do and two of them carry only the unfalsifiable form.
- file: widgets/phone/scenarios/refusals-on-an-ended-call.yaml
  line: 128
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: an unindented duplicate of the assert block's header comment sits between the last refusal step and the OfferCall step it does not describe.
- file: widgets/phone/scenarios/refusals-on-an-ended-call.yaml
  line: 152
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the ActiveCalls excludes assertion decides against zero rows, so the second half of the file's closing claim is carried entirely by the counts assertion above it.
- file: widgets/phone/tests/suite.mjs
  line: 95
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a view the wire reports as unmet is read as an empty view by `held.rows ?? []`, so excludes would pass and counts reports a misleading row count; no obligation in this widget is currently unmet, so I built the state.
- file: widgets/phone/tests/suite.mjs
  line: 53
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: matches() carries the same key-order-sensitive comparison for event payloads; the only struct-valued event fields are SessionOpened's profile and participant and no scenario asserts them.
```
