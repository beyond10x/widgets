---
format: aep.planning-md/1
id: story:call-history
kind: story
status: implemented
title: Record the calls that finished
summary: 'softphone.history: one durable record per call, in the neutral termination vocabulary.'
relations:
- decomposes: epic:browser-softphone
- depends_on: story:media-session-domain
revision: 5
---
# Record the calls that finished

## Outcome

`softphone.history` holds one durable record per finished call, in the neutral termination
vocabulary, optionally attributed to a contact.

## Acceptance

`ess validate` accepts the domain. `CallRecord.termination` is `softphone.control.EndCause` — six
words, none of them SIP's. A record survives its call: the relation to `softphone.control.Call` is
`references`, so nothing in the model lets a call take its record with it. There is no bulk-erase
command. `ended_at >= started_at` is a checked invariant.

**A missed call is derivable**, which it was not when this story shipped: `answered_at` is `Optional`
and `status` carries the SIP response where the far end sent one, so a call nobody took, a call the
user declined and a 486 from the far end are three different rows. Before, all three recorded
identically and this story's acceptance did not notice.

Three bindings reach `RecordCall` — from `CallHungUp`, `CallRejected` and `CallFailed` — so a
finished call arrives in the log without anybody asking. Nothing reached it at all before.

`termination` was `softphone.media.TerminationReason` and is not any more: that was the wrong layer.
A declined call has an end cause and never had a media session.

## What was built

`ess/domains/history.yaml`. `CallRecord` — `record_id`, an optional `call_id`, an optional
`contact_id`, `direction`, `remote`, `started_at`, `ended_at`, `termination`. Three commands:
`RecordCall`, `AttributeRecord`, `DeleteRecord`. Two views: `CallRecordById`, `RecentCalls`.

Bridges 6 and 7 are its two `relations:` entries, both `references`/one, both `Optional` — a record
outlives the call it came from, and a call to a number nobody saved has no contact.

## Why this is a domain and not a view over ended calls

A view would have been cheaper. Three things ruled it out.

- **`owns` forbids the shape a view implies.** A live call cannot own its history entry, because
  `owns` means the far side cannot survive its owner.
- **The vocabulary.** Recording `RemoteHangup` rather than `BYE` is what lets the next binding write
  into the same log unchanged.
- **Retention is not call control.** Answering it inside `softphone.control` would give the surface
  an agent drives a second job.

`remote` is copied into the record rather than referenced, so a record of a call to a contact that
has since been deleted still says what was dialled.

## Not built

`ClearHistory`. Erasing an unbounded set on one press is a separate decision and does not arrive by
implication from *keep a call log*.
