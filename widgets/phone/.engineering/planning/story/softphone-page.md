---
format: aep.planning-md/1
id: story:softphone-page
kind: story
status: draft
title: The softphone page
summary: Registration indicator, dial field, incoming prompt, in-call controls.
relations:
- decomposes: epic:browser-softphone
- depends_on: story:call-lifecycle
- depends_on: story:browser-media-adapter
- depends_on: story:layer-bridges
revision: 4
---
# The softphone page

## Outcome

A surface a person can use: a keypad, a tile per call carrying a contact's name, and controls to
answer, hang up and mute.

## Acceptance

The generated web surface drives a call before any page is hand-written.
`ess generate synthesize --path ess --target web --out <dir>` emits it — 14 artifacts including
`index.html`, `bridge.js`, `catalog.json` and the `softphone-web` WASM crate — and its command list,
forms, event log and state panel are built from the model. Driving a command with nothing installed
returns the typed refusal naming what is owed; that is the correct empty state, not a failure.

## What was built

`ess/domains/presentation.yaml`. Two entities and nothing else:

- `Keypad` — a partially typed address. Not a `softphone.control.RemoteAddress`: it becomes one at
  the moment somebody dials, and until then it is text on a screen.
- `CallTile` — `call_id` (bridge 4, not optional: a tile with no call is not a tile) and an optional
  `contact_id` (bridge 8). The tile is the thing that wants a name, which is why name resolution is
  here and not in the control surface.

`DismissCall` dismisses the tile. It does not end the call — that is `softphone.control.HangUp`, and
keeping them apart is what stops a screen change from dropping a conversation. `PressKey` is not how
digits reach a live call either; that is `softphone.control.SendDigits`, a different surface and a
different grant.

`Human` is the only actor. That is the other half of the actor-neutrality statement:
`softphone.control` grants `Human` and `Agent` the same commands, and this domain grants an agent
nothing. An agent drives the phone; it does not drive a screen.

## Browser policy

Registration may start on load; **media must not.** Placing or answering a call runs from an explicit
user gesture in a secure context, because microphone permission and audio autoplay require both
(`~/babelforce/projects/ai-agent-platform/docs/designs/browser-voice.md` §3, §7).

## Where the state lives, and what is not yet known

Every view query in synthesis is an obligation, and the reason it gives is "how the projection is
kept current is a storage decision". So the specification chooses no storage and the generated page
implements none.

Whether the page keeps anything across a reload is **still unanswered, and currently unanswerable**:
the emitted web tree does not compile at ESS 0.13.1. See
`upstream-blocker:ess-web-target-redeliver` — `crates/softphone-web/src/lib.rs:377` calls
`self.redeliver(&event)`, which the rust target it depends on does not generate. The rust target
from the same specification builds clean, so this is the emitter and not the model.

Until that clears, this story has no way to produce a browser observation of anything.
