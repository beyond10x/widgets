---
format: aep.planning-md/1
id: upstream-blocker:ess-bridge-refuses-a-compiled-module
kind: upstream-blocker
status: open
title: The generated bridge.js refuses the compiled Module its own doc offers
relations:
- informed_by: story:fill-the-obligations
revision: 1
---
# `bridge.js` refuses the compiled `Module` its own documentation offers

## What happens

`crates/verify/…`/`ess-synth`'s web target emits `bridge.js`, whose `open()` doc says: "`source` is
anything `WebAssembly.instantiate` accepts: the bytes of a `.wasm`, or a compiled `Module`."

The implementation is `const { instance } = await WebAssembly.instantiate(source, {})`. That
destructuring is correct for a buffer, where the call resolves to `{ module, instance }`. For a
compiled `Module` the same call resolves to an **`Instance`**, so `instance` is `undefined` and the
next line fails with `Cannot read properties of undefined (reading 'exports')`.

## Where it was hit

`widgets/phone/tests/suite.mjs` compiles the module once and instantiates it per scenario, because a
scenario starts from nothing and a shared store would make the running order part of the result.
Eleven scenarios failed identically before the runner was changed to hand over the bytes each time.

## What it costs

Recompiling an 845 KB module once per scenario. Nothing incorrect, and it does not grow with the
specification — but the documented API is unusable as documented.

## The fix, upstream

Either accept both in `open()` —

```js
const result = await WebAssembly.instantiate(source, {});
const instance = result instanceof WebAssembly.Instance ? result : result.instance;
```

— or delete the sentence offering a `Module`.
