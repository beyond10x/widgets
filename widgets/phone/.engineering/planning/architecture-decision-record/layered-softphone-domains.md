---
format: aep.planning-md/1
id: architecture-decision-record:layered-softphone-domains
kind: architecture-decision-record
status: accepted
title: The softphone is specified as seven independent domains in one ESS system
summary: One system, one domain per concern, bridged by relations ess validate can refuse.
relations:
- decides: epic:browser-softphone
revision: 2
---
# The softphone is specified as seven independent domains in one ESS system

## Status

Accepted 2026-09-04. Supersedes nothing recorded; `ess/domains/session.yaml`, the single conflated
domain it replaces, was removed rather than archived because it was never released.

## Context

The first specification put a media session, a call, a registration and a SIP dialog in one domain,
`softphone.session`. Three concepts that change for different reasons, at different rates, driven by
different actors, in one file. Two more arrived afterwards: a call log and a phonebook.

## Decision

Seven domains in one system, `softphone` v1, one per concern:

| Domain | Concern |
|---|---|
| `softphone.media` | the protocol-neutral media session |
| `softphone.control` | the surface a human or an agent drives, identically |
| `softphone.history` | what happened |
| `softphone.directory` | who can be called |
| `softphone.presentation` | what is on screen |
| `softphone.sip` | the SIP binding |
| `softphone.local` | the local-audio binding |

**One system, not seven.** Cross-domain `relations:` are checked by `ess validate` — five refusal
rules, all five exercised and recorded in `story:layer-bridges`. Three separate ESS systems bridged
by an `ess-composition/1` document would be more independent and less checkable: `ess validate`
cannot see across a composition boundary, so a broken link would surface at composition time
instead of at validate. Independence that costs the checker is not the independence this was for.

**Two bindings, not one.** With a single binding the neutral layer and its only carrier are
indistinguishable, and "protocol-agnostic" is an assertion. `softphone.local` is cheap on purpose:
local audio, no network, no registration. Its existence is the evidence.

**Browser only.** One deployment unit, every component `reached_by: in_process`, no
`ess-composition/1` document, no service, no event log.

## Consequences

- 231 capabilities in web synthesis: 168 generated, 52 obligations, 11 refused. Two runs produce
  byte-identical trees.
- **Actor grants are not enforced by anything generated.** Synthesis refuses them: "a grant is
  checked against a caller identity, which types do not carry, and enforcement belongs to the layer
  that knows who is calling". The `Human`/`Agent` parity is a specification fact and a host
  obligation, not a generated check.
- **Storage is not specified and not generated.** Every view query is an obligation whose stated
  reason is "how the projection is kept current is a storage decision". The phonebook and the
  history are therefore host obligations against browser storage. That is one story, not a reason to
  add a service — adding one would reverse the browser-only decision above.
- Two things ESS cannot express are recorded in the domains that would have carried them: "exactly
  one of these two children exists", and a command in one domain causing a command in another.
- Adding an eighth domain for a second real protocol — `softphone.rtvbp` — touches no existing
  domain. That is the property the split was bought for, and it is the next milestone.

## Cited from

- `docs/design/ess-entity-relations-design-v0.1.md` §2, §3 — the relation shape and the five rules.
- `crates/specify/ess-domain/src/system.rs:481-495` — one type registry for the whole system.
- `crates/specify/ess-domain/src/entity.rs:821-822` — an invariant reads only its own entity.
- `crates/specify/ess-domain/src/component.rs:921` — a component cannot accept another's commands.
- `connectors/crates/domain/src/voice.rs` — the neutral port `softphone.media` was read from.
