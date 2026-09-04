<!--
  generated from softphone v1
  do not edit: regenerate with `ess verify conform web`
-->
# softphone v1 — scenarios

11 scenario(s), compiled from the documents an author wrote. Serve this directory and open `index.html`; a browser will not instantiate a module from a `file://` URL.

```console
$ python3 -m http.server
$ open http://localhost:8000/index.html
```

## What it shows

**Flow.** One lane per actor a scenario names, one row per act, time downward. The current act is lit. A third lane holds what a binding causes — drawn dashed, because the model declares it and the scenario asserts nothing about it.

**State, views and the UI are three different things**, and the tabs keep them apart. State is the truth now. A view is a projection with a filter, parameters and a consistency, so it selects, it needs an argument, and an `eventual` one is allowed to be behind. The UI is whatever a `skin.js` beside this file renders, and there is none unless somebody wrote one.

## What it does not claim

It replays. A scenario declares which outcome each command took and the player applies the effect the model attaches to that outcome. No obligation is filled and nothing here decides anything, so a green walk says the specification is coherent — not that any implementation works.
