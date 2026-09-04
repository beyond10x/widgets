---
format: aep.planning-md/1
id: upstream-blocker:ess-mapping-does-not-widen-optional
kind: upstream-blocker
status: open
title: A binding mapping widens into Optional at validate but not in the emitted Rust
relations:
- blocks: story:call-history
revision: 1
---
# A binding mapping widens into `Optional` at validate and not in the emitted code

## What was observed

A binding mapped an event field of type `T` into a command input of type `Optional<T>`:

```yaml
    when:  { event: softphone.control.CallHungUp }     # call_id: softphone.control.CallId
    invoke: { command: softphone.history.RecordCall }  # call_id: Optional<softphone.control.CallId>
    mapping:
      call_id: event.call_id
```

`ess validate` accepts it. `ess generate synthesize --target rust` emits code that does not compile:

```
error[E0308]: mismatched types
   --> crates/softphone-system/src/lib.rs:268:18
268 |         call_id: event.call_id.clone(),
    |                  ^^^^^^^^^^^^^^^^^^^^^ expected `Option<CallId>`, found `CallId`
help: try wrapping the expression in `Some`
```

Three occurrences, one per binding with that shape.

## Which half is wrong is not established here

Either the mapping is well-typed and the emitter should wrap it, or it is ill-typed and `validate`
should refuse it. Both are defensible and this project cannot tell which was intended. What is
certain is that the two disagree: a document passes every gate and produces a system that does not
build.

Observed at `ess 0.14.0`, commit `a25090533d2dcaf8e3de6736e7cced236571b563`.

## Worked around

`softphone.history.CallRecord.call_id` is now bare rather than `Optional`. That is a better model
anyway — every record was made from a call — so the workaround cost nothing here. It would cost
something in a model where the absence was real.
