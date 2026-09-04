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
| head | — | — |
| worktree | `../widgets-wave-phone-server` | `../widgets-wave-call-lifecycle` |
| build dir | `~/.cache/widgets-wave/phone-server-target` | `~/.cache/widgets-wave/call-lifecycle-target` |
| scratch | `~/.cache/widgets-wave/phone-server-scratch` | `~/.cache/widgets-wave/call-lifecycle-scratch` |
| stage | planned | planned |

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
