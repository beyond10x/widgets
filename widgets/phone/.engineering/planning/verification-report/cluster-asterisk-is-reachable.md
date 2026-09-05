---
format: aep.planning-md/1
id: verification-report:cluster-asterisk-is-reachable
kind: verification-report
status: draft
title: The dev cluster's Asterisk is reachable, admits our calls, and runs its dialplan on them
relations:
- verifies: story:phone-server
- informed_by: story:devcenter-widget
revision: 1
---
# The dev cluster's Asterisk is reachable, admits our calls, and runs its dialplan on them

Measured 2026-09-05 from a workstation. Every line below is a command's own output, not an
inference from a manifest.

## What the far end is

| fact | how it was read |
|---|---|
| a pod, running 51 days | `kubectl get pods -A` |
| a **headless** service — `clusterIP: None`, so DNS answers pod addresses and there is no stable virtual IP | `kubectl get svc -o yaml` |
| SIP on **5062**, both UDP and TCP; AMI on 5038 | same |
| the channel driver is **chan_sip**, not pjsip: `udpbindaddr=0.0.0.0:5062` | `sip.conf:3` in the pod |
| it offers **alaw and slin only** — `disallow=all`, `allow=alaw`, `allow=slin` | `sip.conf:35-37` |
| RTP 10000–60000 | `rtp.conf` |
| `nat=force_rport,comedia`, and `localnet` covers `172.16.0.0/12` | `sip.conf:18,23` |

## How a workstation reaches it

The **pod** network is routed over WireGuard and the **service** network is not:

```console
$ ip route get <pod>            # dev wg0 src 172.20.0.3
$ ip route get <service-ip>     # via <default gateway> dev enp4s0
```

So `--sip-proxy` takes the pod address, and the SDP advertises `172.20.0.3` — the tunnel's own
source address, which is inside the `localnet` above, so Asterisk treats us as local and rewrites
nothing. `Taskfile.yml`'s `call-cluster` resolves both at run time rather than carrying either.

## What it does with our calls

Four `sipx dial` attempts, guest — no registration, no credential:

| dialled | answered |
|---|---|
| `5551234` | `404 Not Found` |
| `1001` | `404 Not Found` |
| `echo` | `404 Not Found` |
| `s` | **`603 Declined`** |

Three responses and two distinct ones is the finding. A SIP response at all means the INVITE
crossed the tunnel and a transaction completed. `allowguest=yes` (`sip.conf:12`) means it was
admitted rather than challenged — a rejected source would be `403`. And `s` answering `603` where
every number answers `404` is Asterisk **executing its dialplan on our call**: `[default]` has
`exten => s,1,NoOp()` then `Hangup` for "a SIP OPTIONS without a phone number", and `603` is what
that branch produces.

**Reach, admission and dialplan execution are established. Audio is not.**

## Why no number here is audible

`[macro-echotest]` answers only when the dialled extension equals `${ENV(VOIPE2E_NUMBER)}`
(`extensions.inbound.conf:17-19`), and the pod's environment does not set it — `env` in the
container lists `TRUNK_ENDPOINT` and no `VOIPE2E_NUMBER`. So the echo test is off, and its `Answer()`
and `Echo()` are unreachable. Every other number goes to `app-inbound-setup`, which is an AGI call
to the platform's handler (`extensions.app.conf:73`) and answers with audio only for a number that
handler has an application for.

Two ways to an audible cluster call, neither of them this repository's to take: set
`VOIPE2E_NUMBER` on the Asterisk deployment — it is ArgoCD-managed, so through that repository and
not by hand — or dial a number already wired to an application.

## What this does not claim

No audio has been carried to or from the cluster. The bridge, the browser leg and the forwarding are
proven against `sipx answer` on one machine and against this crate's own tests; the cluster leg is
proven as far as signalling and no further.
