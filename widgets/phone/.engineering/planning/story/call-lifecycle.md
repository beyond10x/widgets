---
format: aep.planning-md/1
id: story:call-lifecycle
kind: story
status: draft
title: Place, answer and end calls
summary: Wire the softphone.control commands to kernel events in both directions.
relations:
- decomposes: epic:browser-softphone
- depends_on: story:signalling-over-wss
- depends_on: story:media-session-domain
revision: 3
---
# Place, answer and end calls

## Outcome

`softphone.control` is the phone's whole surface, and a human and an agent drive it with the same
commands. No protocol type and no contact appears in it.

## Acceptance

`ess validate` accepts the domain. `Human`'s and `Agent`'s `may:` lists are identical — read from
the compiled IR, not the YAML — and the agent is granted nothing in `softphone.presentation`. A
command issued from a state its transition does not allow returns
`softphone.control.CallStateConflict` and appends nothing. Grep the domain for `sip`, `rtvbp`,
`webrtc` and `directory`: all empty.

## What was built

`ess/domains/control.yaml`. `PhoneEndpoint` — the phone itself, with a `default_binding` typed
`softphone.media.BindingKind`. `Call` — `endpoint_id`, `direction`, `remote`, an optional
`session_id`, `muted`; states `Requested → Ringing → Active → Ended`.

Ten commands, and three actors:

| Actor | Grants |
|---|---|
| `Human` | `ConfigureEndpoint`, `Dial`, `Answer`, `Reject`, `HangUp`, `SendDigits`, `SetMuted` |
| `Agent` | the same seven, byte-identical |
| `Kernel` | `OfferCall`, `RingCall`, `ConfirmAnswer` |

`Answer` and `ConfirmAnswer` take the same `activate` transition and are different facts: the first
is the local decision on an inbound call, the second is the far end answering an outbound one. Only
`Kernel` may assert the second, so a page or an agent cannot claim the far end picked up. `Reject`
and `HangUp` stand in the same relation over `end`.

`Call.session_id` is bridge 1 — `references`, not `owns`, because a media session is a thing in its
own right and a call that ends does not take it with it.

## The parity is the specification, and nothing enforces it

Web synthesis refuses actor grants: "a grant is checked against a caller identity, which types do
not carry, and enforcement belongs to the layer that knows who is calling". So `Human == Agent` is a
fact a reader and a checker of the IR can read, and a host obligation. It is not a generated check.

Verified from the IR rather than the source: an earlier check compared two absent keys and reported
a pass. Seven commands each, identical.
