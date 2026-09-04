---
format: aep.planning-md/1
id: architecture-decision-record:media-adapter-lands-here
kind: architecture-decision-record
status: accepted
title: The browser media adapter lands in this project
summary: Write the audio-only RTCPeerConnection adapter here against the sipx spec, not upstream.
relations:
- decides: story:browser-media-adapter
revision: 2
---
# The browser media adapter lands in this project

## Status

Accepted 2026-09-04. Decided by the operator, in session, in answer to
`decision-blocker:media-adapter-owner`.

## Context

`story:browser-media-adapter` implements what `codewandler/sipx` calls `M-52` — the audio-only
`RTCPeerConnection` adapter. At the pinned revision `004ac534b8b222060ad2d2308763efe6e1dedc10`
(`v1.0.0-rc.23`) that story has not shipped, and `docs/roadmap.md` states its milestone M15 "is
tracked but is **not** part of the selected M13 wave", so upstream has no date for it.

Two homes were available: this project, or a contribution to `codewandler/sipx` consumed back as
`@sipx/browser` (upstream story `A-17`, also unshipped).

## Decision

The adapter lands here.

## Consequences

- The adapter's code and its tests live in this repository, under this repository's licence.
- It is written against `docs/specs/browser-sdk.md` §5.4, §6.2, §8.2 and §8.5, which are normative,
  rather than against a package API that does not exist yet.
- No upstream contribution gate, review or release is on this project's path.
- `@sipx/browser` remains unavailable to other consumers; this decision does not change that, and it
  is the cost of the faster path.
- If `M-52` later ships upstream, this adapter becomes the thing to retire, not the thing to keep.
  Whoever does that reads this record first.

## Cited from

- `docs/roadmap.md` M15 — the story order and the unscheduled state.
- `docs/specs/browser-sdk.md:854` — `M-52` owns the media adapter surfaces.
- `docs/specs/browser-sdk.md:855` — `A-17` owns the package and lifecycle layer.
- `docs/specs/browser-signalling.md:215` — "Anything to do with media. `RTCPeerConnection` is
  `M-52`'s."
