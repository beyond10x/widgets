---
format: aep.planning-md/1
id: story:phone-server
kind: story
status: active
title: Bridge one WebRTC leg to one SIP leg
relations:
- decomposes: epic:browser-softphone
- depends_on: story:fill-the-obligations
- implements: architecture-decision-record:browser-holds-no-sip-stack
scope:
- confidence: inferred
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/phone-server
revision: 7
---
# Bridge one WebRTC leg to one SIP leg

## Outcome

An engineer dials from the page and talks to Asterisk. `widgets/phone/crates/phone-server` is the
only new service, and it holds no phone state.

## Scope

Derived 2026-09-04 by `story-scoper`, confidence **high**. Paths are relative to `widgets/phone`.
Each line is **cited** (read from the story or the tree) or **inferred**.

| path | mark | why |
|---|---|---|
| `crates/phone-server` | cited | the story names it as "the only new service"; `grep -rn phone-server` over the tree matches nothing, so the whole crate is new |
| `Cargo.toml` | cited | `members` is an explicit two-entry list, so a new crate under `crates/` must be added to it; the sipx pins land in `[workspace.dependencies]` |
| `Taskfile.yml` | cited | `impl` is `cargo build --release --target wasm32-unknown-unknown` with no `-p`, so a native member joins that build and needs its own task |
| `Cargo.lock` | inferred | mechanical consequence of new dependencies |

**Verified upstream, at the lines the story cites** — `sipx_media`'s `bind`, `gather`,
`key_with_dtls` and `start_browser_audio` at `crates/sipx-media/src/session.rs:1866,2033,1927,2133`
of `/home/timo/projects/sipx` on `main` `5c302380`, and `sipx_media::Bridge` reached through
`sipx-call`'s `CallBridge` at `crates/sipx-call/src/bridge.rs:1-25`.

**Not touched, and each for a reason read rather than assumed.** `ess/` — the ADR states
`phone-server` runs no ESS system and `softphone.bridge` is already in the tree.
`crates/softphone-behaviour` — all six `softphone.bridge` obligations and all fourteen
`softphone.sip` ones are implemented, and the server reports facts as commands those behaviours
already accept. `crates/softphone-shell`, `tests/suite.mjs`, `scenarios/`, `docs/` — the suite
drives wasm with no network, and `docs/` is generated from an `ess/` that does not change.

**Would collide with** any unit touching `Cargo.toml`, `Cargo.lock` or `Taskfile.yml`, and any unit
landing inside `crates/phone-server` — which is the whole surface of `story:signalling-over-wss`.

**The hole.** No file in this application opens a `WebSocket` or an `RTCPeerConnection`, and the
story names none, so the page-side glue that produces the offer is unplaced. `story:browser-media-adapter`
is where it belongs.

## Acceptance

Before any cluster: `phone-server` against `sipx answer` on the same machine — a real WebRTC leg
from Brave, a real SIP leg to the CLI, audio both ways. Then leg B is repointed at the dev-cluster
Asterisk and the same call is held.

## Blocked on facts nobody here holds

The SIP coordinates of that Asterisk — host, port, transport, and whether it expects a registration
or trusts an address — what to dial, and where `phone-server` runs for the test.
