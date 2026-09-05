---
format: aep.planning-md/1
id: story:devcenter-widget
kind: story
status: draft
title: Mount the phone in Devcenter
relations:
- decomposes: epic:browser-softphone
- depends_on: story:phone-server
scope:
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: beyond10x/devcenter/crates/devcenter-http/src/lib.rs
- confidence: inferred
  path: beyond10x/devcenter/frontend/package.json
- confidence: inferred
  path: beyond10x/devcenter/frontend/src/app/navigation.ts
- confidence: inferred
  path: beyond10x/devcenter/frontend/src/router/index.ts
- confidence: inferred
  path: crates/softphone-shell
- confidence: inferred
  path: package.json
- confidence: cited
  path: player/skin.js
revision: 5
---
# Mount the phone in Devcenter

## Outcome

An engineer opens Devcenter and the phone is in it, driving the same wasm system and the same
`phone-server`.

## Scope

Derived 2026-09-04 by `story-scoper`, confidence **medium** — high that the work is
two-repository and that Devcenter's `frontend/` is the primary surface, low on the exact Devcenter
paths, because no phone feature exists there and no artifact in either store names one.

**This story straddles two repositories, so a wave cannot carry it end to end.** Paths are
qualified by the repository they belong to.

**In `beyond10x/widgets`** — `widgets/phone/player/skin.js` (cited): the component the story names,
and not importable as it stands, because it imports Vue from a path that exists only inside the
emitted player and takes its whole state from `inject('player')`, the scenario player's provider.
Then, inferred: `widgets/phone/Taskfile.yml` (asset assembly lives there),
`widgets/phone/crates/softphone-shell` (packaging only), and a new `widgets/phone/package.json`
**only if** the component enters Devcenter as a git-pinned pnpm dependency, which is the precedent
`@b10x/agentide-ui` and `@b10x/service-console-vue` set.

**In `beyond10x/devcenter`** — `frontend/package.json` and `frontend/e2e/devcenter.spec.ts` (cited);
then, inferred, `frontend/pnpm-lock.yaml`, a new `frontend/src/features/phone/`,
`frontend/src/router/index.ts`, `frontend/src/app/navigation.ts`, `frontend/src/app/AppShell.vue`,
`frontend/src/app/search.ts`, `frontend/public/`, `frontend/vite.config.ts`,
`frontend/review/plugin.ts`, `frontend/review-e2e/`, and
`crates/devcenter-http/src/lib.rs` for the CSP.

**The CSP is the finding.** The served policy carries `script-src 'self' 'wasm-unsafe-eval'`, so the
module instantiates, and `connect-src 'self'`, which blocks the per-phone WebSocket to
`phone-server`. Either that string changes — and `frontend/e2e/devcenter.spec.ts:1231` asserts it
verbatim, so the test moves with it — or `phone-server` is proxied onto Devcenter's own origin.
Which of the two is undecided in either store.

**Would collide with**, in Devcenter, any unit touching the frontend's shared registration files —
the router, the navigation map, the manifest or the lockfile — or the CSP and its assertion. In
widgets, any unit touching `widgets/phone/player/skin.js` or `widgets/phone/Taskfile.yml`.

## Acceptance

`pnpm --dir frontend review` on `http://127.0.0.1:4173` renders it, and the journey the standalone
page passes — dial, answer, mute, hold, hang up, with audio — passes inside Devcenter.

## Depends on

The M1 shell and the M2 adapter. Nothing about this story is specification work.
