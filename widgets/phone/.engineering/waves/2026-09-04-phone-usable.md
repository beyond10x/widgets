# Wave — make the phone usable, turn 1

Coordinator: this session. Skill `adp:wave` **0.6.1**. `aep --version` → `protocol 0.54.0`.
Base branch `main` at `e6720f9`. Integration branch `wave/phone-usable`.

## Selection

`aep artifact waves --kind story --status draft --format json` returned **4 waves, 13 collisions, 0
unassessed, 0 cycles** after this session wrote typed scope entries for all six draft stories from
six `adp:story-scoper` runs. Its packing was:

| verb's wave | stories |
|---|---|
| 1 | `story:phone-server` |
| 2 | `story:devcenter-widget`, `story:signalling-over-wss` |
| 3 | `story:browser-media-adapter`, `story:call-lifecycle` |
| 4 | `story:softphone-page` |

**This turn runs `story:phone-server` and `story:call-lifecycle` together**, which is a different
packing from the verb's and rests on the verb's own collision list: it records no collision between
those two, so the store says their surfaces are disjoint. The verb returns *a* partition, not the
only one; where it and a reading disagree the verb decides, and here it does not disagree — it
packed differently.

`story:devcenter-widget` is **out of every wave in this repository**: its primary surface is
`beyond10x/devcenter/frontend`, and a wave runs in one repository's worktrees.

## Collisions the verb excluded, verbatim

```
browser-media-adapter ↔ devcenter-widget   Taskfile.yml                    inferred
browser-media-adapter ↔ devcenter-widget   crates/softphone-shell          inferred
browser-media-adapter ↔ phone-server       Cargo.toml                      inferred
browser-media-adapter ↔ phone-server       Taskfile.yml                    inferred
browser-media-adapter ↔ softphone-page     Taskfile.yml                    inferred
browser-media-adapter ↔ softphone-page     docs                            inferred
call-lifecycle        ↔ softphone-page     scenarios                       inferred
devcenter-widget      ↔ phone-server       Taskfile.yml                    inferred
devcenter-widget      ↔ softphone-page     Taskfile.yml                    inferred
devcenter-widget      ↔ softphone-page     player/skin.js                  inferred
phone-server          ↔ signalling-over-wss  crates/phone-server           cited
phone-server          ↔ softphone-page     Taskfile.yml                    cited
signalling-over-wss   ↔ softphone-page     pages/phone                     inferred
```

`Taskfile.yml` is in five of the thirteen. It is the file every unit wants a task in, and it is why
the remaining stories run in sequence rather than at once.

**One limitation of the comparison, recorded rather than worked around:** the store compares path
strings, so `scenarios` and `scenarios/registration-recovers.yaml` are not seen as the same
surface. The pairs this turn dispatches were also read by hand.

## Units

| | unit 1 | unit 2 |
|---|---|---|
| story | `story:phone-server` | `story:call-lifecycle` |
| scope | **cited** (`crates/phone-server`, `Cargo.toml`, `Taskfile.yml`) | **cited** (`scenarios`, `tests/suite.mjs`) |
| branch | `impl/phone-server` | `impl/call-lifecycle` |
| forked at | `eb40ddc` | `eb40ddc` |
| head | **`5115fa3`** | **`d9668a2`**, merged as `d36634f` |
| worktree | `~/.local/state/worktree/trees/b10x/widgets/wave-phone-server` | `~/.local/state/worktree/trees/b10x/widgets/wave-call-lifecycle` |
| build dir | `~/.cache/widgets-wave/phone-server-target` | `~/.cache/widgets-wave/call-lifecycle-target` |
| scratch | `~/.cache/widgets-wave/phone-server-scratch` | `~/.cache/widgets-wave/call-lifecycle-scratch` |
| brief | `~/.cache/claude-tmp/briefs/unit-1-phone-server.md` | `~/.cache/claude-tmp/briefs/unit-2-call-lifecycle.md` |
| stage | pass 1 red, **correction round 1 running** | **merged** into `wave/phone-usable` at `d36634f` |

The trees are the operator's managed ones — `worktree create --purpose … --base wave/phone-usable
--id wave-<unit>`, ids `wave-phone-server` and `wave-call-lifecycle` — not raw `git worktree`, which
this machine's standing rule reserves. Cleanup is `worktree finish` and a reviewed
`worktree gc --apply --id <id>`, never a force or a manual delete.

**Implementors do not commit.** The brief forbids it: this repository accepts only `b10x-bot[bot]`
authorship through the Atlas wrapper, so each unit's work is committed here, from its worktree,
after its diff has been read. The wave page's `head` column stays at the fork point until that
happens.

`subagent_type` for every dispatch: **`adp:implementor`**, then **`adp:adversary`** per unit.

## Pre-flight

| check | number |
|---|---|
| working tree | clean, on `main` at `e6720f9` |
| `git worktree list` | one entry, the main checkout |
| free disk | **32 G** (97% used). It was 47 G eight minutes earlier; the drop is not this session's — `~/.cache/dl-blocking` is 11 G and `~/.cache/wave-categorize` 3.5 G |
| one measured build | `cargo build --release --target wasm32-unknown-unknown` into an empty target directory: **4 s**, **11 M**, with `sccache` warm (4.5 G cache, 6584 hits) |
| compiler cache | `sccache` is installed and `RUSTC_WRAPPER` was **unset** — both briefs set it |
| model budget | not stated by the operator; the skill's default is 4 and this wave uses **2** |
| `AGENTS.md` | read: `ess` binary is the authority, `docs/` is generated, planning through `aep artifact` only, commits through the Atlas bot wrapper |

## Shared surfaces the coordinator keeps

`docs/`, `pages/phone/`, `.engineering/planning/`, and the regeneration of both documentation trees.
Neither unit commits a generated file; `task docs` and `task pages` run here, at integration.

## Commits this wave makes

Two unit commits, one merge per unit into `wave/phone-usable`, the opening and closing store
commits, and the merge of `wave/phone-usable` into `main` once the whole gate is green. **No push,
no tag, no release.**

## Unit 2, as it stands

Implementor green. 181 → **257** steps, 11 → **14** scenarios, `task scenarios` 0 refusals. Three
scenario documents and 37 lines in `tests/suite.mjs`, which learned `expect_error` and
`expect_no_event`. The authored surface can express a refusal — `error: {name, fields}` plus
`no_events:` — so the brief's "if it cannot, that is the finding" did not fire.

Three of its own assertions were driven red by mutation and reverted. The fourth — that a refusal
published nothing — could not be driven red, because the generated wire writes `"published":[]`
unconditionally on every `WrongState` arm. It is a guard, not a measured check, and the report says
so rather than implying otherwise.

Two things it handed back, both filed rather than absorbed:

- **`story:prove-every-refusal-branch`** — the model declares 26 `wrong-state` branches and this
  unit takes 12. The implementor wrote the check that turns the remaining 14 into a gate, ran it,
  confirmed it names exactly those 14, and took it back out because a gate that fails on work
  nobody was assigned breaks the integration branch. The patch is in that story's body, because the
  worktree it was written in does not survive this wave.
- **`story:gate-formatting-and-lints`** — `cargo fmt --check` and `cargo clippy … -D warnings` have
  never run in this application and are both red at `eb40ddc`, in `crates/softphone-behaviour`:
  7 unformatted files and 26 lints. `origin: pre-existing`, from `story:fill-the-obligations`.
  `task check` does not run either, so the integration gate stays green; it is filed, not fixed,
  because the surface belongs to no unit of this wave.

**A stale published tree to regenerate at integration:** `pages/phone/player/suite.json` holds 11
scenarios and there are 14, so `task pages-drift` will refuse until `task pages` has run here.

## Unit 2, adversary pass 1

`review-result:adversary-call-lifecycle-pass-1`. **NEEDS-CHANGE**, 2 red cases, 8 findings —
2 `introduced`, 6 `pre-existing`, 0 `undecided`.

The blocker is worth stating in full because it is the kind a passing suite hides: **a mutant whose
refusals delete the entity instead of restoring it passes all 14 scenarios and all 257 steps.**
`expect_error` observes only the response, and `no_such_call()` answers an instance the store never
held with the same terminal state a real one gets. The unit proved that a refusal answers correctly
and nothing about the state it claims to leave alone — which is the half its own story asks for.

Routed: four findings back to the same implementor, which still holds its context — teach the runner
`query_view`/`expect_view` (the response already carries `views` and the runner discards them), make
the adversary's two cases green, make `expect_no_event` either bite or say it is a guard, and count
steps for a scenario that fails. Brief: `~/.cache/claude-tmp/briefs/unit-2-correction.md`.

Filed rather than absorbed: `story:refusals-cannot-say-not-found` (the not-found state and the
unfiltered parameterised queries — both `INFEASIBLE` inside this unit) and, already filed,
`story:prove-every-refusal-branch`. The story's own grep clause is confirmed unsatisfiable and its
acceptance is the coordinator's to restate.

Attack budget: **1 of 2 spent.**

## Unit 1, as it stands

Implementor green and committed as `5115fa3` — committed early, and out of order with the adversary,
because free disk fell from 47 G to 16 G under the wave and 5,139 lines of uncommitted work is the
thing worth securing first. 15 cases, 12 red on the first run.

| leg | proven |
|---|---|
| SIP | fully — one real outbound call over loopback UDP against `sipx_call`'s own `answer` role: 180, 200, ACK, BYE, plus the no-answer timeout class and one RFC 4733 keypress read at the far end |
| bridge | fully, with audio — half a second recorded at the far end three hops from the source, plus pass-through against transcoding, and mute staying local to one leg |
| browser | **SDP contract only** — ten weakened offers each refused with its own stated boundary and none answered; the DTLS and ICE half needs a peer presenting the certificate the fingerprint named, which is `story:browser-media-adapter`'s |

**Two facts it measured that this coordinator had wrong.**

`sipx =1.1.0` **is not published**. `cargo fetch` refuses it; 1.0.1 is the highest published version
and carries every entry point the story cites. So no git-rev pin: the manifest says `=1.0.1` from
crates.io and records the measurement rather than the assumption.

**The server→browser table in unit 1's brief was wrong.** I wrote that a far leg ending arrives as
`softphone.control.HangUp`; `HangUp` is granted to `Human` and `Agent` only, and
`softphone.control.Kernel.may` is `AttachMedia, ConfirmAnswer, FailCall, MediaConnected, OfferCall,
RingCall`. A far leg going away leaves as `FailCall` with one of the six `EndCause` classes. The
unit did not implement my table; it implemented the specification and wrote the test that keeps it
that way — `tests/wire.rs` reads `ess compile --format json` and fails if the server sends a command
no kernel actor may.

**One patch it handed back, and it is unit 2's surface:** `tests/suite.mjs:16` hard-codes
`./target/…`, so `task check` and `task test` exit 201 with ENOENT in any tree that redirects
`CARGO_TARGET_DIR` — which every agent in this wave was told to do. Preserved at
`~/.cache/claude-tmp/needs-coordinator--suite-mjs-honours-CARGO_TARGET_DIR.patch` and going to unit 2.

## Disk

| when | free |
|---|---|
| pre-flight | 47 G |
| after unit 2's implementor | 32 G |
| after unit 2's adversary | 23 G → 18 G |
| after unit 1's implementor | **16 G, 99% used** |

**Not the wave.** Its whole footprint is `phone-server-target` 2.6 G, `call-lifecycle-target` 121 M
and 13 M of scratch. `~/.local/state/worktree` holds **228 G** — other sessions' managed trees — and
`worktree gc --dry-run` reports all but one of them `retained: no-remote-recovery-proof`, which is
the tool refusing to delete unpushed work. So there is nothing here to reclaim and nothing of mine
to reclaim that is not still in use.

## Unit 2, correction round 1 — green

**304 of 304 steps, 16 scenarios, 0 failures, and the mutant goes red in three of them.** All four
routed items landed. The runner learned `query_view` and `expect_view`, which is what turns an
assertion about a response into an assertion about the store; the response already carried every
view's rows and the runner was throwing them away.

Two judgement calls I accept as made:

- **`at`, `ranked` and `satisfies` are refused by name rather than implemented.** The catalogue does
  not publish `order_by` — checked against `softphone.history.RecentCalls`, which declares one in
  `history.yaml:230` and has no such key in `catalog.json` — so a comparator written now would be
  untested and would mis-assert in silence. The implementor also corrected its own earlier
  inference here, from "no view declares an order" to "the catalogue does not publish the key".
- **`query_view` applies a parameterised view's filter itself**, requiring each bound parameter to
  appear as `x == param.x`, stripping those clauses and refusing when a parameter remains. It is not
  a fix for the wire defect and does not claim to be; it is what stops the assertion being about the
  whole store while wearing the view's name. Verified against the specification's one compound
  filter rather than only the easy ones.

## Two process defects this wave found in itself

**A worktree cannot read a store artifact written after it was forked.** Unit 2's implementor could
not open `review-result:adversary-call-lifecycle-pass-1`: its tree has no `review-result/` directory,
because the artifact was created in the main checkout after `eb40ddc`. It worked from the correction
brief instead and said so. **The brief must carry the review's content, not a pointer to it** —
pass 2's dispatch does that.

**A unit's brief can be wrong about the specification, and the specification wins.** Unit 1's brief
had `HangUp` as a kernel-reported fact; it is not. The unit implemented the model and wrote the test
that keeps it that way. Both units caught something in their brief this wave, which is the argument
for writing briefs to a file: a wrong line in a file can be quoted back.

## Filed from unit 2's correction, not absorbed

`story:a-delete-either-cascades-or-it-does-not` — `scenarios/phonebook-contact-and-address.yaml:1-3`
says "nothing here cascades. The addresses survive their contact"; `directory_impl.rs:157-163`
cascades on purpose. Measured: after `DeleteContact`, `ContactAddresses` holds no rows. The scenario
has no `assert:` block, which is why both statements passed the gate. One of the two comments is
false and which one is `story:phonebook`'s call.

Attack budget on unit 2: **2 of 2 spent** once pass 2 returns. After it, the correction is mine to
read, and the unit merges or goes to a person — no third pass.

## The hard-coded module path, and where the class stops

Unit 1 found that `tests/suite.mjs` read `./target/…` and so failed in every tree that redirects
`CARGO_TARGET_DIR`; unit 2 fixed it, in `6c0a0e4`. Two corrections to what I had written:

- **The patch no longer applied** — unit 2's own round-2 change had landed inside its context hunk —
  so unit 2 wrote the equivalent and kept unit 1's comment verbatim. Both facts are in its report.
- **Node exits 1 on that ENOENT.** The 201 I recorded was `task`'s own mapping of the failure, not
  the runner's code.

Unit 2 also enumerated the class rather than declaring it closed: `suite.mjs` resolves four paths the
build system could move, and the wasm module was the only one reachable by an environment variable.
The other three — `bridge.js`, `suite.json`, `catalog.json` — move only if somebody runs
`task <target> OUT=x`, and nothing in the repository does. **Coordinator's decision: leave them.** A
hard-coded default with no caller that moves it is not a live defect, and closing it needs
`Taskfile.yml` to pass `OUT` through as an environment variable, which buys a coupling for a case
that does not occur. Recorded here so the next reader does not re-derive it.

It also removed the `target/wasm32-unknown-unknown` symlink it had created in round 1 — the
workaround for the bug just fixed, and one that would have let the suite read a module out of a
cache directory while a reader believed they were reading `./target`.

## Unit 1, adversary pass 1 — two blockers

`review-result:adversary-phone-server-pass-1`. 6 red cases, 8 findings, **all `introduced`** and
checked rather than assumed: `eb40ddc` contains no `crates/phone-server` at all.

**The bridge does not resample, and the unit's audio proof ran on a codec pair the server never
negotiates.** 500 ms spoken on an Opus browser leg arrives at the G.711 far end as 23,040 samples —
2,880 ms. `sipx-media`'s transcoding leg moves buffers with no rate conversion, the unit's own
`browser_profile.rs:149` proves the browser offer negotiates Opus, and `sip.rs:193` places G.711.
The three green bridge cases used a PCMU pair. This is the class of defect a passing suite is worst
at: the test was right and the pair was wrong.

**Coordinator's decision, sent as correction round 1: no resampler.** Answer PCMU on the browser leg
if the profile permits the answer to select it while the offer still carries Opus, and refuse an
Opus acceptance fail-closed if it does not. Both legs then sit at 8 kHz and the bridge is a
pass-through. `story:opus-needs-rate-conversion` carries the real fix and records that the better
version of it belongs upstream in sipx, where both rates are known.

Second blocker: `set_held` and `set_browser_muted` swapped one `AtomicBool`, so unholding cleared a
mute the page never lifted. The specification declares `muted` and `held` as two independent fields
and sipx's own `Call::mute` doc heads a section "Mute is not hold".

Also routed back: the crate doc named `softphone.control.HangUp` among what the server sends — the
same wrong claim `tests/wire.rs` caught in my brief, now in the prose `cargo doc` publishes — a
hard-coded SDP session id shared by every bridge against RFC 4566 §5.2, a doc claim about binding
order its only caller contradicts, and `FailCall{cause: Media}` for three different failures. The
adversary also closed the enumeration hole the implementor had named and could not close.

## Unit 2, adversary pass 2 — the budget is spent

`review-result:adversary-call-lifecycle-pass-2`. 9 findings — 8 `introduced`, 1 `pre-existing`.
`aep artifact findings story:call-lifecycle` reads **carried 0, new 9, resolved 8**: nothing from
pass 1 came back, and pass 2 found nine things pass 1 did not. Diverging, not regressing.

Three blockers. A true claim about a struct-valued field **cannot** match, because `matchesRow`
compares with `JSON.stringify` while the compiled literal is key-sorted and the wire writes
declaration order — and ESS's own reference runner compares structurally, so the specification
accepts the row the runner rejects. Then two mutants that survive all 16 scenarios: deleting the
refusal-restore lines in `close()` and `terminate()`, and a `ConfirmBridge` refusal that restores
the bridge *and* writes `remote_sdp`, which is the exact invariant one of those scenarios says it
guards.

The adversary's five new scenarios answer the two mutants, and one of them executes a branch of the
runner that had **zero** coverage — the correction's claim to have verified the compound filter was
a read, not a run.

Correction round 2 is dispatched and it is the last: two attacks are the budget, so after it **the
diff is mine to read** — no assertion dropped, nothing re-pinned by relaxing it — and then the unit
merges or goes to a person.

## Unit 2 — merged

Correction round 2 green: **21 scenarios, 411 of 411 steps, 0 failures, and all three mutants red**
(pass 1's at 4 red, M2 at 2, M3 at 2). Committed `d9668a2`, merged `d36634f`.

**I read the diff myself, which is what the budget being spent means.** Both things it asks for
hold: no assertion was dropped, and nothing was re-pinned by relaxing it. Every scenario change is a
strengthening — `contains: {state: Closed}` became five projected fields plus an `excludes` on the
answer the refused command carried; the second call in the ended-call scenario is driven to `Active`
so that "holds exactly one row" has another row to leave out and "`ActiveCalls` excludes it" has a
live row to keep; and a struct-valued event payload is asserted where nothing asserted one before.
In the runner, `held.rows ?? []` became a refusal naming the unmet capability, which removes a
silent-pass path rather than adding one.

**Two corrections it made to me, both measured.** My brief said the adversary's scenario covered
deleting the unbound-parameter guard; it does not, and cannot: `ess conform author` requires every
declared parameter to be bound, so no document can under-bind one. Rather than leave a `throw`
reading as a check for a third pass to find, it wrote a table into `rowsFor`'s doc comment saying
which of the function's eight refusals a scenario can reach — two checked by scenarios, one only by
a mutated query, five unreachable from any document.

**And two unsound measurements of its own, reported rather than buried.** It had piped
`ess conform author` through `>/dev/null` and read the runner's green over a suite that had silently
dropped the mutated document — `20 authored scenario(s) from 21 file(s), 1 refusal(s)`. Not a hole
in the gate: `ess conform author` exits 1 on a refusal, so `task scenarios` would have caught it. It
was the ad-hoc loop that hid it.

Cost: 4 agent runs, **1,138,341** sub-agent tokens, 235 tool uses, 66 minutes of wall time across
the four.

## Unit 3, added mid-wave at the operator's request — the page's offer

The operator asked for the remaining issue to be fixed, on **Fable**. The remaining issue is the one
unit 1's refusal creates: a browser offers Opus first, the server answers and then declines, so no
real call bridges. The answerer cannot reorder formats — `validate_answer` refuses an answer that
does — so the fix is the offer, and the offer is the page's.

| | unit 3 |
|---|---|
| story | `story:browser-media-adapter`, in part — the offer and the one function under it |
| scope | `web/` — cited by the measurement, recorded on that story as `inferred` because the story still names no file |
| branch | `impl/pcmu-first-offer`, forked from `wave/phone-usable` at `d36634f` |
| worktree | `~/.local/state/worktree/trees/b10x/widgets/wave-pcmu-offer`, id `wave-pcmu-offer` |
| scratch | `~/.cache/widgets-wave/pcmu-offer-scratch` |
| build dir | none — nothing here compiles Rust |
| brief | `~/.cache/claude-tmp/briefs/unit-3-pcmu-offer.md` |
| agent | `adp:implementor` on **Fable** |
| stage | implementor dispatched |

**No collision.** `web/` is a new directory no other unit names; unit 1 holds
`crates/phone-server`, `Cargo.toml`, `Cargo.lock` and `Taskfile.yml`, and unit 3 is forbidden the
Taskfile by its brief — the task it would have added comes back in its report and I land it.

Three files: a pure `pcmuFirst(sdp)` that reorders one `m=audio` format list and nothing else, the
small amount of `RTCPeerConnection` around it, and cases for the pure half — which is the whole
reason it is separated, since nothing under node can hold a peer connection. The offer is
non-trickle: the server answers one offer once.
