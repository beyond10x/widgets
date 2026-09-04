---
format: aep.planning-md/1
id: story:devcenter-widget
kind: story
status: draft
title: Mount the phone in Devcenter
relations:
- decomposes: epic:browser-softphone
- depends_on: story:phone-server
revision: 1
---
# Mount the phone in Devcenter

## Outcome

An engineer opens Devcenter and the phone is in it, driving the same wasm system and the same
`phone-server`.

## Scope

Devcenter's frontend is Vue 3 + Vite + pnpm 11.25.0 (`beyond10x/devcenter/frontend/package.json`)
and this project's `player/skin.js` is already a Vue component, so the surface is a component and a
wasm asset rather than a rewrite.

## Acceptance

`pnpm --dir frontend review` on `http://127.0.0.1:4173` renders it, and the journey the standalone
page passes — dial, answer, mute, hold, hang up, with audio — passes inside Devcenter.

## Depends on

The M1 shell and the M2 adapter. Nothing about this story is specification work.
