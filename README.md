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
specification exists, and the phone behaviours are implemented.** What is here is the model, nine
domains of it, the crates that fill every obligation its plan owes, and the evidence that it holds
together; what is not here is storage or authorisation, which the specification deliberately leaves
to a host.

### Nine domains, one per concern

| Domain | Responsible for |
|---|---|
| `softphone.media` | the media session, and no protocol. Modelled construct-for-type from the protocol-neutral Rust port in [`beyond10x/connectors`](https://github.com/beyond10x/connectors) |
| `softphone.control` | the surface. A human and an agent are granted the same seven commands, byte for byte |
| `softphone.history` | one durable record per finished call |
| `softphone.directory` | contacts and the addresses they are reachable at |
| `softphone.presentation` | the keypad, one tile per call, and which mode the screen is in |
| `softphone.sip` | the SIP binding, including the offer/answer exchange |
| `softphone.local` | a local-audio binding |
| `softphone.bridge` | the media bridge to the server that holds the SIP leg: where it points, the offer it sent, the answer it got and why it closed |
| `softphone.presence` | this phone's standing with that server, and the roster of other phones it has been told about |

The seventh is the point of the first. With one binding, a neutral layer and its only carrier are
indistinguishable and "protocol-agnostic" is an assertion; `softphone.local` is what makes the claim
checkable, and the check is mechanical — search the compiled `domains."softphone.media"` for
`softphone.sip.*`, `softphone.local.*` or `softphone.bridge.*` and it returns nothing.

### What joins them

**Nine `relations:` entries**, eight crossing a domain boundary. A broken one is refused by
`ess validate` under five rules — an unknown target, a missing carrier, a mistyped carrier, a second
owner, a twice-carried field — and all five were exercised by introducing the defect and recording
the code.

**Ten `bindings:`**, each one event causing one command. They are why an arriving call puts itself
on screen and why a finished call reaches the log without anybody asking for either.

### Counts

`ess validate` reports `softphone v1 — 12 file(s), valid`. 63 commands, 63 events, 21 views, 15
entities, 9 workloads. `ess generate synthesize --target rust` reports 384 capabilities: 276
generated, 84 obligations, 24 refused — and the emitted crates build.

**And the 84 are filled.** `crates/softphone-behaviour` implements every `*Behavior` and `*Query`
trait the plan owes; `crates/softphone-shell` installs it into the generated web bridge through that
bridge's own `install` seam, so the emitted page, wire and catalogue are untouched and the page says
"a realization is installed". `task test` replays the authored scenarios through the same three
WebAssembly exports a browser uses, which is the difference between a specification that is coherent
and one that executes. The 23 is the count `ess conform author` reports today; `task test` was not
run when this paragraph was last edited, and the most recent recorded green run is 21 scenarios and
411 of 411 steps (`.engineering/waves/2026-09-04-phone-usable.md:318`).

## What the specification deliberately does not decide

**Storage.** Every view query is an obligation whose stated reason is "how the projection is kept
current is a storage decision", so where the phonebook and the call log live is a host's answer.

**Authorisation.** All fifteen actor grants are refused by synthesis: a grant is checked against a
caller identity, which types do not carry, and enforcement belongs to the layer that knows who is
calling. The `Human`/`Agent` parity is a fact a reader and a checker can read, and a host obligation.

Both are on the [obligations page](widgets/phone/docs/obligations.md), with all 84.

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
pages with their lifecycle diagrams, the interaction graph, the topology, the type crossings, all 84
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
