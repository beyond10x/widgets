# The softphone specification

Seven domains in one ESS system, `softphone` v1. A browser softphone: it registers a SIP account
over secure WebSocket, places and answers audio calls, keeps a log of them and a phonebook, and
shows all of it in a page.

## The layers, and why there are seven of them

| Domain | What it is responsible for |
|---|---|
| `softphone.media` | the media session, and **no protocol**. Modelled construct-for-type from `connectors/crates/domain/src/voice.rs`, the neutral port that already existed in Rust. |
| `softphone.control` | the surface. A human and an agent are granted the same seven commands, byte for byte, and that identity is the specification of actor-neutrality. |
| `softphone.history` | one durable record per finished call, in the neutral termination vocabulary rather than in SIP causes. |
| `softphone.directory` | contacts and the addresses they are reachable at. `softphone.control` never names a contact. |
| `softphone.presentation` | the keypad, one tile per call, and which mode the screen is in. |
| `softphone.sip` | the SIP binding — the only domain that names SIP, SDP, an address-of-record or a WebSocket. |
| `softphone.local` | a local-audio binding. It exists so that the media layer's neutrality has evidence behind it: with one binding, the neutral layer and its only carrier are indistinguishable. |

The layers are joined by **eight `relations:` entries**, seven of which cross a domain boundary, and
by **seven `bindings:`**, which are what make one layer react to another. `ess validate` refuses a
broken relation — five rules — and the *Interactions* page draws every binding.

## Reading it

- **Topology** — what each component needs from a browser to run at all.
- **Interactions** — the seven reactions, and the events nothing reacts to. An event with no reader
  is either a boundary or a binding somebody forgot, and only a person can tell which.
- **Type crossings** — every place two contexts have to agree about a type.
- **Obligations** — what a specification does *not* decide. Every command behaviour and every view
  query is an obligation a host has to fill; the page lists all of them with their contracts.
- **Refusals** — what a browser could not carry across the plan.

## What it does not decide

Storage. Every view query is an obligation whose stated reason is "how the projection is kept
current is a storage decision", so where the phonebook and the log live is the host's answer and not
this document's.

Actor grants. Synthesis refuses all eleven of them: "a grant is checked against a caller identity,
which types do not carry, and enforcement belongs to the layer that knows who is calling."
