---
format: aep.planning-md/1
id: story:signalling-over-wss
kind: story
status: draft
title: Register the SIP leg, on the server that holds it
summary: 'Registration moved off the browser: phone-server drives REGISTER to a terminal state.'
relations:
- decomposes: epic:browser-softphone
- depends_on: story:media-session-domain
- informed_by: architecture-decision-record:browser-holds-no-sip-stack
scope:
- confidence: cited
  path: crates/phone-server
- confidence: inferred
  path: crates/softphone-behaviour/src/sip_impl.rs
- confidence: cited
  path: docs/domains/softphone-sip.md
- confidence: cited
  path: ess/domains/sip.yaml
- confidence: inferred
  path: pages/phone
- confidence: inferred
  path: scenarios/registration-recovers.yaml
revision: 11
---
# Register the SIP leg, on the server that holds it

## What changed about this story

It was "Register over secure WebSocket": the page would load the sipx WebAssembly session kernel,
open WSS to a SIP provider and drive REGISTER itself.
`architecture-decision-record:browser-holds-no-sip-stack` moved that off the browser. Registration
is still specified — `softphone.sip` is unchanged — and it is now `phone-server`'s, which
`ess/topology.yaml` states: `sip-binding` requires `network: sip-signalling` and `network: rtp-media`
rather than two browser facilities.

The kernel this story was built on is not the reason it moved, but it is why it could not have
shipped as written: `sipx` `A-17` (package the browser SDK) and `M-52` (browser-native WebRTC audio)
are both `status: ready`.

## Outcome

`phone-server` drives `softphone.sip.Register` to `Registered` or `Failed`, and back out of `Failed`
— that state stopped being terminal for exactly this reason: one transient drop must not kill a
registration permanently.

## Acceptance

Against the SIP service the deployment names, a configured address-of-record and expiry produce a
`softphone.sip.RegistrationConfirmed` fact and `softphone.sip.RegistrationById` reports `Registered`.
A rejected REGISTER produces `RegistrationFailed` and no retry loop — `RetryRegistration` is a
command somebody issues, not a loop the model implies. `RefreshRegistration` extends the lifetime
with no state change.

## Scope

Derived 2026-09-04 by `story-scoper`, confidence **medium** — the `ess/` half is cited to the line
by the story itself; the server half names a crate that exists in no commit, so nothing inside it
is citable. Paths are relative to `widgets/phone`.

| path | mark | why |
|---|---|---|
| `ess/domains/sip.yaml` | cited | the body's scope reads "`softphone.sip` only". `:47-49` is `SignallingTransport`, `:52-54` is `MediaSecurity`, and `:6-8`, `:16-20`, `:51` are the comments justifying each one-member set by a browser limit the ADR retires |
| `crates/phone-server` | cited | "the server-side code that drives them through `sipx-call`" |
| `docs/domains/softphone-sip.md` | cited | generated and committed, and `task drift` inside `task check` fails when it is not what the specification determines |
| `pages/phone` | inferred | `task pages` regenerates the committed published tree and `pages-drift` demands it |
| `crates/softphone-behaviour/src/sip_impl.rs` | inferred | it fills every `softphone.sip` obligation but touches `transport` and `media_security` only as clones, so the two enum edits alone do not reach it |
| `scenarios/registration-recovers.yaml` | inferred | the only `softphone.sip` scenario, and it pins `transport: SecureWebSocket` |

**Two open questions this unit answers, both in `ess/domains/sip.yaml`.** `SignallingTransport` has
one member, `SecureWebSocket`, and a server can use UDP, TCP or TLS — so either the member set grows
or the deployment is stated to be WSS-only. `MediaSecurity` has one member, `DtlsSrtp`; the browser
leg keeps it and the SIP leg to a dev-cluster Asterisk is most likely plain RTP or SDES-keyed SRTP.

**Not a surface:** `ess/topology.yaml` — `sip-binding` already requires `network: sip-signalling`
and `network: rtp-media`; this story reports that rather than editing it.

**Would collide with** `story:phone-server`, unconditionally: `crates/phone-server` is that story's
whole surface and this one lands inside it. The store declares no edge ordering the two.

## What is no longer in scope

The vendored `browser/src/` binding, the `crates/sipx-wasm` module, the hand-written ABI glue, and
the WebSocket-subprotocol token trick a browser needs because it cannot set an `Authorization`
header on an upgrade. A server sets the header.
