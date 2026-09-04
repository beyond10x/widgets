---
format: aep.planning-md/1
id: upstream-blocker:ess-conform-author-does-not-check-a-union-member
kind: upstream-blocker
status: open
title: ess conform author admits any object for a union input; the wire then refuses it
relations:
- informed_by: story:fill-the-obligations
revision: 1
---
# `ess conform author` does not check a union member, and the web wire then refuses it

## What happens

`softphone.media.ChannelSignal` is `kind: union`, `tag: kind`, with one variant `dtmf` carrying a
`DigitString`. A scenario wrote its input as the documentation and the domain comment spell it:

```yaml
signal: { kind: dtmf, dtmf: '5' }
```

`ess conform author` compiled it with **0 refusals**. The emitted module then refused the step at run
time:

```json
{"kind":"undecodable","at":"input.signal.value","expected":"a value","found":"nothing"}
```

The web wire decodes a union as `{ "kind": <variant>, "value": <payload> }`. Rewriting the scenario
to `{ kind: dtmf, value: '5' }` also compiles with 0 refusals, so **the author accepts either** — it
checks nothing inside the literal — and only one of the two runs.

## Why it matters

`ess conform author` is the step whose value is that "a name the model does not declare is refused
now rather than at the first run". For every other construct it holds. Here a scenario that names
the payload key wrongly is admitted, and the defect surfaces only once something executes the suite
— which, until this project filled its obligations, nothing did.

## The fix, upstream

Resolve a union literal against the declared variants in the authored-scenario compiler: refuse a
tag no variant declares, and refuse a payload key that is not the wire's. Either spelling could be
the accepted one; what cannot stand is both compiling and one failing.

## Worked around here

`scenarios/media-session-lifecycle.yaml` uses the spelling the wire decodes, with a comment saying
why the readable one is not there.
