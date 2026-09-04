---
format: aep.planning-md/1
id: architecture-decision-record:media-session-owns-nothing
kind: architecture-decision-record
status: accepted
title: The media session owns nothing
summary: Bridges 2 and 3 move to the binding side as references; two bindings hold what owns claimed.
relations:
- decides: story:layer-bridges
- supersedes: architecture-decision-record:layered-softphone-domains
revision: 2
---
# The media session owns nothing

## Status

Accepted 2026-09-04. Amends `architecture-decision-record:layered-softphone-domains`, which recorded
bridges 2 and 3 as `owns` relations on `softphone.media.MediaSession`.

## Context

`softphone.media` claimed, in its own header comment and in the acceptance statement of
`story:media-session-domain`, that nothing in it names a binding. It named two:
`BindingKind`'s variants are `Sip` and `Loopback`, and the two `owns` relations targeted
`softphone.sip.SipDialog` and `softphone.local.LoopbackDevice`. An independent review found the
generated documentation contradicting itself inside one file — `domains/softphone-media.md` printed
"no binding type appears here" and, eighty lines later, "It owns at most one
`softphone.sip.SipDialog`".

## Decision

The two relations move to the binding side and become `references`:

| | before | after |
|---|---|---|
| bridge 2 | `media.MediaSession owns sip.SipDialog` | `sip.SipDialog references media.MediaSession` |
| bridge 3 | `media.MediaSession owns local.LoopbackDevice` | `local.LoopbackDevice references media.MediaSession` |

The carrier field was already on the far side with the correct type, so this is a one-way edit and
`ess validate` accepts both spellings.

`BindingKind` stays in `softphone.media`. System level is its right home and the reason a
system-level `types:` block exists, and it is not usable:
`upstream-blocker:ess-system-level-type-breaks-synthesis`.

## What this costs

`owns` says the far side cannot outlive its owner. That is the whole content of the kind, per the
relations design §5, and `references` does not say it. So the guarantee that a SIP dialog cannot
survive the session it carries is no longer a rule.

**Two bindings hold it instead**, and being bindings they are visible in the interaction graph rather
than implied by a relation kind:

- `end-session-with-dialog` — `sip.SipDialogClosed` causes `media.TerminateSession`
- `end-session-with-loopback` — `local.LoopbackDetached` causes `media.TerminateSession`

That is weaker in one specific way worth stating: a binding is `at_least_once` with `on_failure:
retry`, so it can be late, where a rule could not be. Nothing implemented a cascade under `owns`
either, so no behaviour was lost — but a reader who trusted the word `owns` was trusting something
the model was not going to enforce.

## Consequence

The claim is now mechanical rather than asserted: `ess compile --format json`, then search
`domains."softphone.media"` for `softphone.sip.*` or `softphone.local.*`, returns nothing. That check
is the acceptance statement of `story:media-session-domain`, replacing a `grep` over the source that
returned five comment lines.
