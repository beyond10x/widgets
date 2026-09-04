# Widgets

**▶ [Play the scenarios](https://beyond10x.github.io/widgets/phone/player/)** ·
[documentation](https://beyond10x.github.io/widgets/phone/) ·
[obligations](https://beyond10x.github.io/widgets/phone/plan/obligations.html)

Small, self-contained applications specified end to end before they are built. Each one is a
directory under [`widgets/`](widgets), holds its own [ESS](https://github.com/beyond10x/ess)
specification and its own [AEP](https://github.com/beyond10x/aep) planning store, and is deployable
on its own.

There is one so far.

## `widgets/phone` — a browser softphone

A softphone that registers a SIP account over secure WebSocket and carries audio in a browser page,
built on the [`codewandler/sipx`](https://github.com/codewandler/sipx) browser SDK. **The
specification exists; the implementation does not.** What is here is the model, seven domains of it,
and the evidence that it holds together.

### Seven domains, one per concern

| Domain | Responsible for |
|---|---|
| `softphone.media` | the media session, and no protocol. Modelled construct-for-type from the protocol-neutral Rust port in [`beyond10x/connectors`](https://github.com/beyond10x/connectors) |
| `softphone.control` | the surface. A human and an agent are granted the same seven commands, byte for byte |
| `softphone.history` | one durable record per finished call |
| `softphone.directory` | contacts and the addresses they are reachable at |
| `softphone.presentation` | the keypad, one tile per call, and which mode the screen is in |
| `softphone.sip` | the SIP binding, including the offer/answer exchange |
| `softphone.local` | a local-audio binding |

The seventh is the point of the first. With one binding, a neutral layer and its only carrier are
indistinguishable and "protocol-agnostic" is an assertion; `softphone.local` is what makes the claim
checkable, and the check is mechanical — search the compiled `domains."softphone.media"` for
`softphone.sip.*` or `softphone.local.*` and it returns nothing.

### What joins them

**Eight `relations:` entries**, seven crossing a domain boundary. A broken one is refused by
`ess validate` under five rules — an unknown target, a missing carrier, a mistyped carrier, a second
owner, a twice-carried field — and all five were exercised by introducing the defect and recording
the code.

**Seven `bindings:`**, each one event causing one command. They are why an arriving call puts itself
on screen and why a finished call reaches the log without anybody asking for either.

### Counts

`ess validate` reports `softphone v1 — 10 file(s), valid`. 52 commands, 52 events, 16 views, 12
entities, 7 invariants, 7 workloads. `ess generate synthesize --target rust` reports 307
capabilities: 221 generated, 68 obligations, 18 refused — and the emitted crates build.

## What the specification deliberately does not decide

**Storage.** Every view query is an obligation whose stated reason is "how the projection is kept
current is a storage decision", so where the phonebook and the call log live is a host's answer.

**Authorisation.** All eleven actor grants are refused by synthesis: a grant is checked against a
caller identity, which types do not carry, and enforcement belongs to the layer that knows who is
calling. The `Human`/`Agent` parity is a fact a reader and a checker can read, and a host obligation.

Both are on the [obligations page](widgets/phone/docs/obligations.md), with all 68.

## Reading it

| | |
|---|---|
| [Overview](widgets/phone/docs/index.md) | the system as a graph, its contexts and its components |
| [Interactions](widgets/phone/docs/interactions.md) | the seven reactions, and the events nothing reacts to |
| [Type crossings](widgets/phone/docs/crossings.md) | every place two contexts have to agree about a type |
| [Topology](widgets/phone/docs/topology.md) | what each component needs from a browser to run at all |
| [Obligations](widgets/phone/docs/obligations.md) | what a specification does not decide |

Everything under `widgets/phone/docs/` is generated from the specification and carries a
`do not edit` header. `task docs` regenerates it; `task drift` fails if what is committed is not what
the specification determines.

The same model is published as a browsable site at
**[beyond10x.github.io/widgets/phone/](https://beyond10x.github.io/widgets/phone/)** — the domain
pages with their lifecycle diagrams, the interaction graph, the topology, the type crossings, all 68
obligations, and the [scenario player](https://beyond10x.github.io/widgets/phone/player/). `task
pages` assembles it and `task pages-drift` refuses a stale one.

## Working on it

```console
$ task validate   # ess validate over the specification
$ task check      # validate, drift, compile
$ task site       # render the browsable HTML, obligations page included
```

Requires the [`ess`](https://github.com/beyond10x/ess) CLI. The planning store under
`widgets/phone/.engineering/` is read and written with the [`aep`](https://github.com/beyond10x/aep)
CLI — never by hand.

## Licence

Apache 2.0. See [LICENSE](LICENSE).
