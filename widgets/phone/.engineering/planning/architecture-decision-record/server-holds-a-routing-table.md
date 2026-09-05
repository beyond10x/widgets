---
format: aep.planning-md/1
id: architecture-decision-record:server-holds-a-routing-table
kind: architecture-decision-record
status: accepted
title: The server holds a routing table and still holds no phone state
relations:
- decides: epic:browser-softphone
- informed_by: architecture-decision-record:browser-holds-no-sip-stack
revision: 2
---
# The server holds a routing table and still holds no phone state

## Status

Accepted for this project. It refines
`architecture-decision-record:browser-holds-no-sip-stack` rather than replacing it, and narrows one
sentence of it.

## Context

`crates/phone-server/src/main.rs` says, in prose that a reader will find before they find this
record:

> **One phone per connection, and no state outside it.** A connection holds its own bridge and
> nothing else does; when it closes, the bridge is dropped and both legs go with it. That is the
> ADR's arrangement made structural rather than remembered — there is no map here for a second
> phone's call to be looked up in.

Presence and a phone-to-phone call both need exactly that map. For phone A to reach phone B, some
process has to know that B is connected and where to send a frame, and a browser cannot know it: a
page sees one WebSocket and nothing else on the network.

## Decision

**The server holds a routing table. It still holds no phone state.**

The distinction is not a wording trick, and it is what keeps the accepted ADR true:

| the server holds | the browser holds |
|---|---|
| which handles are connected, and the channel each is reachable on | whether a call is ringing, active, held or ended |
| which media sessions it has terminated, for a bridge or a mixer | which call a media session is attached to |
| nothing that survives the connection | the log, the roster, the history, the screen |

Everything in the left column is answerable by looking at the process's own sockets, and none of it
outlives them. Everything in the right column is a claim about a call, and it stays where the
accepted ADR put it: in the page, as one authority, with no second event log to reconcile.

## Consequences

- **Presence is reported, not shared.** The server tells every connected page that a phone arrived
  or left, as `softphone.presence.NotePresent` and `NoteGone` — commands the specification grants a
  kernel actor. Each page projects its own roster from those facts and from nothing it asserted
  itself, so two pages that disagree are two projections and not a split log.
- **A roster is not durable.** A page that reconnects is told the roster again. The server keeps no
  record of a phone that is not currently connected, which is why `softphone.presence` models
  present and gone and nothing else — no last-seen, no availability, no status text.
- **The paragraph in `main.rs` becomes wrong and must be rewritten** in the same change that adds
  the table, with a pointer here. A comment that contradicts the code is worse than no comment.
- **A handle is claimed, not assigned.** Two phones asking for one handle is a conflict the server
  is the only thing that can see, so `AnnouncePresence` asks and `ConfirmPresence` or
  `FailPresence` answers. Until one of them arrives the phone is reachable by nobody, which is what
  the `Announcing` state is for.
- The table is per process. A conference or a peer call that spans two `phone-server` instances is
  out of scope, and nothing here is a step toward it.
