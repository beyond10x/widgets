# Widgets

Each directory under `widgets/` is one application, specified before it is built. It owns its ESS
specification and its AEP planning store, and neither is edited by hand.

## Serves

The objectives named in Atlas's ROADMAP.md that this repository moves:

- **O2 — decisions as data, with evidence.** Typed specifications, cross-domain relations and
  executable scenarios make application behavior and its remaining obligations checkable.
- **O4 — products built from the platform.** Small, independently deployable applications combine
  reusable capabilities; the browser phone joins protocol-neutral media with SIP and local
  bindings and gives human and agent callers the same control surface.

The current application and its evidence are described in [README.md](README.md).

## Before changing a specification

Read the `ess-schema` skill, and treat the installed `ess` binary as the authority — not a checkout
that happens to be in the workspace. `/home/timo/beyond10x/ess` has been a different, older tree
than the installed CLI more than once, and a reader who trusted it concluded that entity `relations:`
did not exist. Establish the version and its source commit from `~/.cargo/.crates2.json`, then read
the matching checkout under `~/.cargo/git/checkouts/`.

Run `ess validate --path <spec>` after **each** edit, not once at the end. `task check` is the gate:
validate, documentation drift, compile.

## What `ess validate` does not check

It checks that references resolve and that types match. It does not check that a declared field can
ever be written, that an event has a reader, or that a comment is true. Every one of those produced
a real defect here, so assert against the compiled IR rather than the source:

```console
$ ess compile --path <spec> --format json
```

Useful assertions, all of which hold today and none of which `validate` would have caught:

- no `creates`/`updates` outcome without a `sets:` block, or a stated reason for having none
- no two views byte-identical once `name` and `naming` are removed
- `bindings` non-empty — an event with no reader is a boundary or a forgotten binding
- the `Human` and `Agent` grants in `softphone.control` identical

## Documentation

`docs/` is generated and carries a `do not edit` header on every file. Change the specification and
run `task docs`; never edit the output. `b10x.docs.yaml` and
`.github/workflows/b10x-docs-pages.yml` are **Atlas-owned** — they are produced by
`atlas docs reconcile`, and editing them here puts this repository out of step with the delivery
plan that routes it.

## Planning

`widgets/*/.engineering/planning/` is written only through `aep artifact`. A status changes only
through `aep artifact move`; a refusal names every status legal from where the artifact stands, and
that refusal is the answer rather than an obstacle.

## Commits

`b10x-bot[bot]`, through the Atlas `scripts/as-bot.sh` wrapper. Never raw `git commit` or
`git push`, and never `--no-verify`. Semantic subjects, a body with detail, no attribution trailers.
