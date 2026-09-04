---
format: aep.planning-md/1
id: upstream-blocker:ess-system-level-type-breaks-synthesis
kind: upstream-blocker
status: open
title: A system-level type validates and then breaks all three ESS synthesis targets
relations:
- blocks: story:media-session-domain
revision: 1
---
# A system-level type validates and breaks every synthesis target

## What was observed

`system.yaml` may declare a `types:` block of its own — a type belonging to no domain, which
`crates/specify/ess-domain/src/system.rs` checks only for being inside the system's namespace.
`ess validate` and `ess compile` both accept one. All three synthesis targets then fail:

```
$ ess generate synthesize --path ess --target rust --out <dir>
`softphone.BindingKind` is not a declaration this layout knows: it was derived from a different IR
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

`--target go` and `--target web` fail identically. The four documentation and contract kinds are
unaffected: `--kind docs` (11 artifacts), `--kind schema` (146), `--kind openapi` (7),
`--kind asyncapi` (7) all succeed with the same document.

## Isolated

A copy of the specification with the one type moved back into a domain — nothing else changed —
synthesises: `231 capabilities: 168 generated, 52 obligation(s), 11 refused`, 30 artifacts. The
system-level declaration is the sole cause.

Observed at `ess 0.14.0` (`git+https://github.com/beyond10x/ess?tag=0.14.0#a25090533d2dcaf8e3de6736e7cced236571b563`).

## What it cost here

`softphone.media.BindingKind` enumerates the two bindings and lives in the one domain whose whole
claim is that it names none. System level is the right home for it and the reason the construct
exists. It is declared in `domains/media.yaml` instead, with the refusal quoted beside it.

## What would clear it

The synthesis layout resolving a type that belongs to no domain. Nothing in this project can.
