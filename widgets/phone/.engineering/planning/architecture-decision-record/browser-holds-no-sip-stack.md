---
format: aep.planning-md/1
id: architecture-decision-record:browser-holds-no-sip-stack
kind: architecture-decision-record
status: accepted
title: The browser holds no SIP stack; a server does, and the page bridges to it
relations:
- decides: epic:browser-softphone
revision: 2
---
# The browser holds no SIP stack; a server does, and the page bridges to it

## Status

Accepted for this project. It replaces the arrangement `story:signalling-over-wss` assumed, and
`topology.yaml` is where the change is visible.

## Context

Until now `sip-binding` declared `browser: secure-websocket` and `browser: rtc-peer-connection`, so
the specification said this page registers over SIP and negotiates its own dialogs. Two facts made
that the wrong model.

A browser owns `RTCPeerConnection`, ICE, DTLS-SRTP, capture and render, and it owns no SIP stack.
The one that would have served — the `sipx.browser.v1` kernel in `crates/sipx-wasm` — is real but
unpackaged: `A-17` (generate and package the browser SDK) and `M-52` (browser-native WebRTC audio)
are both `status: ready`, not `done`.

Second, the operator's requirement is that an engineer reach a telephone from a page in Devcenter.
That is a deployment with a server in it either way, and a server that already terminates SIP has
no use for a second SIP stack in the browser.

## Decision

The page opens **one media bridge** to a server it names: a control channel and one WebRTC audio
leg. `softphone.bridge` is that half of the arrangement, and `softphone.media.BindingKind` gains
`Bridge` beside `Sip` and `Loopback`.

The server — `widgets/phone/crates/phone-server` — holds the SIP leg. `softphone.sip` therefore
stays in this specification and stops being a browser workload: its `requires:` entries are now
`network: sip-signalling` and `network: rtp-media`.

**The browser holds the phone's state; the server is a device.** `phone-server` runs no ESS system.
It reports facts about legs, and each fact arrives as a command the specification already grants to
a kernel actor — `RingCall`, `ConfirmAnswer`, `MediaConnected`, `FailCall`, `ConfirmBridge`,
`FailBridge`. There is one authority over a call's state and no event log to reconcile across a
process boundary.

## Consequences

- `softphone.bridge` declares no SIP construct and its own `SessionDescription`, rather than reusing
  `softphone.sip`'s. Two carriers of one neutral session must not depend on each other, and a shared
  type between them would be that dependency.
- `activate-session-with-bridge` is a binding the `bindings:` comment had rejected for
  `control.CallAnswered`, on the ground that the event carries no session id. `BridgeConfirmed`
  carries one, so the reaction is legal here and the session no longer needs a person to activate it.
- `story:signalling-over-wss` ("Register over secure WebSocket") describes work this decision moves
  off the browser. It is not deleted: registration is still specified, and it is now the server's.
- Nothing from any other organisation's code is used, and no gRPC appears. The control channel
  carries the ESS command and event wire the specification already determines.

## Alternatives considered

**SIP over secure WebSocket from the page**, with `sipx-wasm` or a JavaScript SIP library. It needs
no server of ours and the specification already modelled it. Rejected: the kernel's packaging and
media adapter are unshipped, and a third-party SIP library in the page is a second protocol stack
this project would then own.

**A vendor control API with a hosted media path.** Rejected by the operator: the phone is to be
replicated rather than assembled from somebody else's service.
