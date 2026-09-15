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
run `task docs`; never edit the output.

`b10x.docs.yaml` **is in this tree.** It was hand-written on 2026-09-15 (commit `4d1040c`) for the
catalog adoption, deliberately and as an exception, and its own lines 2-3 say so. It is
**Atlas-owned from adoption on**: once this repository carries a row in `atlas/docs/catalog.md`,
`atlas docs reconcile` regenerates the file and the delivery and routing fields stop being this
repository's to choose. Until then, change only the source and presentation fields the manifest's
header assigns to this repository, and do not restructure it — the generator's shape
(`schema: b10x-docs/v4`) is what reconcile will rewrite against.

`.github/workflows/b10x-docs-pages.yml` is **Atlas-owned and still not in this tree**; it is
created by `atlas docs reconcile` at adoption. Never hand-write it — an authored copy puts this
repository out of step with the delivery plan that routes it. (`.github/workflows/pages.yml` is a
different, repository-owned workflow and is not the Atlas one.)

## Planning

`widgets/*/.engineering/planning/` is written only through `aep artifact`. A status changes only
through `aep artifact move`; a refusal names every status legal from where the artifact stands, and
that refusal is the answer rather than an obstacle.

## Commits

`b10x-bot[bot]`, through the Atlas `scripts/as-bot.sh` wrapper. Never raw `git commit` or
`git push`, and never `--no-verify`. Semantic subjects, a body with detail, no attribution trailers.

<!-- b10x-release-operations:start -->
## Release completion

An ordinary release completes after this repository's exact tag, required source checks,
published release and required artifacts are verified. A pushed tag with unfinished checks or
uploads is queued; report it as released only after those requirements succeed.

Atlas reconciliation and public documentation publication run asynchronously. Do not wait for
Atlas or Website, update Website source locks or bootstrap snapshots, promote consumer pins,
release shared docs tooling, or redeploy documentation façades as part of an ordinary source
release. Report documentation as pending unless its publication was actually verified. A background
documentation failure does not invalidate a successful source release.

Keep this repository's provenance, correctness, security, compatibility and artifact verification
requirements. Shared rendering, routing or delivery-control changes still require their relevant
integration gates. A release request does not authorize deployment or downstream releases.
Repositories without a release unit retain their existing publication policy. This completion
boundary supersedes older instructions that attach synchronous documentation ceremony to each
source release.
<!-- b10x-release-operations:end -->
