---
format: aep.planning-md/1
id: decision-blocker:media-adapter-owner
kind: decision-blocker
status: cleared
title: Nobody has decided whether the browser media adapter lands here or upstream in sipx
relations:
- blocks: story:browser-media-adapter
revision: 3
---
# Who owns the browser media adapter

## The question

`story:browser-media-adapter` implements what upstream calls `M-52`. It can live in this project, or
be contributed to `codewandler/sipx` and consumed from there.

## Why it is not mine to decide

`docs/roadmap.md` states M15 "is tracked but is **not** part of the selected M13 wave", so upstream
has no date. Writing it here is faster and is reversible for this prototype; writing it upstream is
slower, needs the sipx contribution gates in `AGENTS.md`, and is the only version that ends with an
installable `@sipx/browser` for every other consumer.

Nothing in the repository settles which, and the answer changes where the tests live and which
licence applies.

## What clears it

Answered by the operator on 2026-09-04: the adapter lands in this project.
`architecture-decision-record:media-adapter-lands-here` holds the decision and its consequences.
