---
format: aep.planning-md/1
id: upstream-blocker:ess-web-target-redeliver
kind: upstream-blocker
status: open
title: ESS 0.13.1 web synthesis emits a call to a method its rust target does not generate
relations:
- blocks: story:softphone-page
withholds: test_result
revision: 2
---
# The generated web target does not compile at ESS 0.13.1

## What was observed

`ess generate synthesize --path ess --target web` emits a tree that fails to build:

```
crates/softphone-web/src/lib.rs:377:14
    self.redeliver(&event)?;
         ^^^^^^^^^ method not found in `&mut System<CallHistoryBehaviors, ..., ...>`
error: could not compile `softphone-web` (lib) due to 1 previous error
```

`redeliver` appears **nowhere** in the rust target the web crate depends on — `grep -rn redeliver`
over all 30 emitted Rust artifacts returns nothing. The web emitter emits a call to a method the
rust emitter does not generate.

## It is not this specification

The rust target from the same specification builds clean: `cargo build --release` →
`Finished release profile [optimized] target(s) in 1.02s`, all nine crates. Both targets report the
same 231 capabilities — 168 generated, 52 obligations, 11 refused — and web synthesis is
byte-identical across two runs. The specification validates: `softphone v1 — 9 file(s), valid`.

So the defect is in `ess generate synthesize --target web` at `0.13.1`
(`git+https://github.com/beyond10x/ess?tag=0.13.1#d1a66772a91b5411d942d7a45bbf08dfc5de4651`), in the
`replay` function of the emitted bridge.

## What it blocks

`story:softphone-page`'s acceptance is *the generated web surface drives a call before any page is
hand-written*. That surface cannot be loaded, so no browser observation of this specification can be
produced at all — which also leaves the open question of whether the page persists anything across
a reload unanswerable.

## What would clear it

A fix upstream in `beyond10x/ess`: either the web emitter stops calling `redeliver`, or the rust
emitter generates it. One line of the emitted bridge names the seam.

Filed as an `upstream-blocker` rather than a `decision-blocker` because there is nothing here to
decide. `aep artifact kinds` does not list this kind by name — the `<type>-blocker` family is open
and `aep artifact lifecycle upstream-blocker` answers `open -> cleared`.

## What is not blocked

Everything specification-side. All eight bridges are in the compiled IR, the five refusal codes were
observed, and the rust target builds — so the layering is verified without the page.

## Version, corrected

The version this record was filed against is wrong twice over, and the defect outlived both.

Filed as ESS `0.13.1`. The installed binary was `0.13.5` at the time of the reviews, and was
reinstalled to **`0.14.0`** (`git+https://github.com/beyond10x/ess?tag=0.14.0#a25090533d2dcaf8e3de6736e7cced236571b563`)
partway through the work. The `redeliver` defect is present at 0.14.0: `redeliver` appears 12 times
in the web output and 0 times in the rust output from the same specification, and the rust target
builds clean.

The model source is byte-identical across 0.13.1 and 0.13.5 (`diff -rq` over
`crates/specify/ess-domain/src` and `docs/`), so nothing read from either tree is invalidated —
only the version number in this record was.
