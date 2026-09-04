---
format: aep.planning-md/1
id: story:phone-server
kind: story
status: draft
title: Bridge one WebRTC leg to one SIP leg
relations:
- decomposes: epic:browser-softphone
- depends_on: story:fill-the-obligations
- implements: architecture-decision-record:browser-holds-no-sip-stack
revision: 1
---
# Bridge one WebRTC leg to one SIP leg

## Outcome

An engineer dials from the page and talks to Asterisk. `widgets/phone/crates/phone-server` is the
only new service, and it holds no phone state.

## Scope

- **browser leg** — `sipx-media`: `bind`, `gather`, `key_with_dtls`, then `start_browser_audio`
  against the offer the page sent (`crates/sipx-media/src/session.rs:1866,2033,1927,2133`). The
  profile is fail-closed and is `docs/specs/webrtc-audio.md` §1: one `audio` section,
  `UDP/TLS/RTP/SAVPF`, rtcp-mux on one ICE component, fingerprint verified before keys.
  Non-trickle — one offer, one answer, candidates gathered before either crosses the wire.
- **SIP leg** — a `sipx-call` outbound call, PCMU.
- **the bridge** — `sipx_media::Bridge`, which forwards audio between two media sessions
  (`crates/sipx-call/src/bridge.rs:1-25`). Both legs are sipx sessions, so no PCM is injected into
  a live call and the read-side seam in `docs/specs/call-audio-seam.md` is not on this path.
- **control** — one WebSocket per phone, JSON, carrying the ESS commands and events the
  specification determines. No gRPC and no second protocol.

`start_browser_audio` is on sipx `main` (`5c302380`); its stories `M-134` and `M-137` are recent. If
the published `=1.1.0` does not carry it, pin sipx by git rev and say so in the manifest.

## Acceptance

Before any cluster: `phone-server` against `sipx answer` on the same machine — a real WebRTC leg
from Brave, a real SIP leg to the CLI, audio both ways. Then leg B is repointed at the dev-cluster
Asterisk and the same call is held.

## Blocked on facts nobody here holds

The SIP coordinates of that Asterisk — host, port, transport, and whether it expects a registration
or trusts an address — what to dial, and where `phone-server` runs for the test.
