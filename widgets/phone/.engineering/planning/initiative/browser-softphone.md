---
format: aep.planning-md/1
id: initiative:browser-softphone
kind: initiative
status: draft
title: Browser softphone 0.1
summary: A standalone browser softphone on the codewandler/sipx browser SDK, in the shape todo established.
revision: 1
---
# Browser softphone 0.1

## Outcome

Deliver a standalone browser softphone that registers a SIP account and carries two-way audio, built
on the `codewandler/sipx` browser SDK, in the shape `todo` established for a reference service:
one ESS domain, one deployment unit, its own planning store.

## Constraints

sipx is a user agent, not a media engine. Its README states it is "not a proxy, registrar, PBX,
browser media engine, or video stack", and its M15 milestone assigns `RTCPeerConnection`, ICE,
DTLS-SRTP, capture and render to the browser. This project therefore owns the media plane in
JavaScript and takes signalling and dialog state from the sipx WebAssembly kernel.

Audio only. Video and data channels are out of scope, as they are upstream.

## Cited from

- sipx `v1.0.0-rc.23`, rev `004ac534b8b222060ad2d2308763efe6e1dedc10`, pinned at
  `connectors/crates/driver-sip/Cargo.toml:21-25`, checked out at
  `~/.cargo/git/checkouts/sipx-65343d777b6000c5/004ac53`.
- `README.md` "Does it fit?" — the user-agent boundary and the `browser-audio` composition profile.
- `docs/roadmap.md` M15 "Browser-embeddable audio" — the six stories and their order.
- `todo/service.yaml` and `todo/.engineering/planning/` — the reference-service shape being copied.
