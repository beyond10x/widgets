<!--
generated from softphone v1
model digest 522f372d9f45cae05577d8379ac02cac26b458beceb66dea739471ba4f91e59e
contract digest a716fbbbe26b7fdd24cf69b5441a8a7a57b304f349ef2e63e6705b0c329ffd20
do not edit: regenerate with `ess generate`
-->

# Interactions

A binding is the only way an event in one context causes a command in another. Each one states how many times the command may run and what happens when it does not, because a binding that can fail quietly is the difference between specifying a system and specifying a demo.

[Back to the index](index.md).

## `activate-session-with-bridge`

A bridge that came up activates the session it carries.

`softphone.bridge.BridgeConfirmed` causes [`softphone.media.ActivateSession`](domains/softphone-media.md#activatesession).

```mermaid
flowchart LR
    event["softphone.bridge.BridgeConfirmed"]
    command["softphone.media.ActivateSession"]
    event -->|"activate-session-with-bridge"| command
    outcome0["activated"]
    command --> outcome0
    emit0_0["softphone.media.SessionActivated"]
    outcome0 --> emit0_0
    outcome1["wrong-state"]
    command --> outcome1
    error1["softphone.media.MediaSessionStateConflict"]
    outcome1 --> error1
    error1 --> failure["retried by the transport"]
```

Delivered **at least once**, so `softphone.media.ActivateSession` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `session_id` (`softphone.media.MediaSessionId`) ← the event's `session_id` (`softphone.media.MediaSessionId`).

## `end-session-with-bridge`

Closing the bridge terminates the session it carried.

`softphone.bridge.BridgeClosed` causes [`softphone.media.TerminateSession`](domains/softphone-media.md#terminatesession).

```mermaid
flowchart LR
    event["softphone.bridge.BridgeClosed"]
    command["softphone.media.TerminateSession"]
    event -->|"end-session-with-bridge"| command
    outcome0["terminated"]
    command --> outcome0
    emit0_0["softphone.media.SessionTerminated"]
    outcome0 --> emit0_0
    outcome1["wrong-state"]
    command --> outcome1
    error1["softphone.media.MediaSessionStateConflict"]
    outcome1 --> error1
    error1 --> failure["retried by the transport"]
```

Delivered **at least once**, so `softphone.media.TerminateSession` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `session_id` (`softphone.media.MediaSessionId`) ← the event's `session_id` (`softphone.media.MediaSessionId`).
- `reason` (`softphone.media.TerminationReason`) ← the event's `reason` (`softphone.media.TerminationReason`).

## `end-session-with-dialog`

Closing the dialog terminates the session it carried.

`softphone.sip.SipDialogClosed` causes [`softphone.media.TerminateSession`](domains/softphone-media.md#terminatesession).

```mermaid
flowchart LR
    event["softphone.sip.SipDialogClosed"]
    command["softphone.media.TerminateSession"]
    event -->|"end-session-with-dialog"| command
    outcome0["terminated"]
    command --> outcome0
    emit0_0["softphone.media.SessionTerminated"]
    outcome0 --> emit0_0
    outcome1["wrong-state"]
    command --> outcome1
    error1["softphone.media.MediaSessionStateConflict"]
    outcome1 --> error1
    error1 --> failure["retried by the transport"]
```

Delivered **at least once**, so `softphone.media.TerminateSession` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `session_id` (`softphone.media.MediaSessionId`) ← the event's `session_id` (`softphone.media.MediaSessionId`).
- `reason` (`softphone.media.TerminationReason`) ← the event's `reason` (`softphone.media.TerminationReason`).

## `end-session-with-failed-bridge`

A bridge that failed terminates the session it carried.

`softphone.bridge.BridgeFailed` causes [`softphone.media.TerminateSession`](domains/softphone-media.md#terminatesession).

```mermaid
flowchart LR
    event["softphone.bridge.BridgeFailed"]
    command["softphone.media.TerminateSession"]
    event -->|"end-session-with-failed-bridge"| command
    outcome0["terminated"]
    command --> outcome0
    emit0_0["softphone.media.SessionTerminated"]
    outcome0 --> emit0_0
    outcome1["wrong-state"]
    command --> outcome1
    error1["softphone.media.MediaSessionStateConflict"]
    outcome1 --> error1
    error1 --> failure["retried by the transport"]
```

Delivered **at least once**, so `softphone.media.TerminateSession` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `session_id` (`softphone.media.MediaSessionId`) ← the event's `session_id` (`softphone.media.MediaSessionId`).
- `reason` (`softphone.media.TerminationReason`) ← the event's `reason` (`softphone.media.TerminationReason`).

## `end-session-with-loopback`

Detaching the loopback terminates the session it carried.

`softphone.local.LoopbackDetached` causes [`softphone.media.TerminateSession`](domains/softphone-media.md#terminatesession).

```mermaid
flowchart LR
    event["softphone.local.LoopbackDetached"]
    command["softphone.media.TerminateSession"]
    event -->|"end-session-with-loopback"| command
    outcome0["terminated"]
    command --> outcome0
    emit0_0["softphone.media.SessionTerminated"]
    outcome0 --> emit0_0
    outcome1["wrong-state"]
    command --> outcome1
    error1["softphone.media.MediaSessionStateConflict"]
    outcome1 --> error1
    error1 --> failure["retried by the transport"]
```

Delivered **at least once**, so `softphone.media.TerminateSession` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `session_id` (`softphone.media.MediaSessionId`) ← the event's `session_id` (`softphone.media.MediaSessionId`).
- `reason` (`softphone.media.TerminationReason`) ← the event's `reason` (`softphone.media.TerminationReason`).

## `record-failed-call`

A call the network ended reaches the log.

`softphone.control.CallFailed` causes [`softphone.history.RecordCall`](domains/softphone-history.md#recordcall).

```mermaid
flowchart LR
    event["softphone.control.CallFailed"]
    command["softphone.history.RecordCall"]
    event -->|"record-failed-call"| command
    outcome0["recorded"]
    command --> outcome0
    emit0_0["softphone.history.CallRecorded"]
    outcome0 --> emit0_0
```

Delivered **at least once**, so `softphone.history.RecordCall` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `call_id` (`softphone.control.CallId`) ← the event's `call_id` (`softphone.control.CallId`).

## `record-hung-up-call`

A call the user ended reaches the log.

`softphone.control.CallHungUp` causes [`softphone.history.RecordCall`](domains/softphone-history.md#recordcall).

```mermaid
flowchart LR
    event["softphone.control.CallHungUp"]
    command["softphone.history.RecordCall"]
    event -->|"record-hung-up-call"| command
    outcome0["recorded"]
    command --> outcome0
    emit0_0["softphone.history.CallRecorded"]
    outcome0 --> emit0_0
```

Delivered **at least once**, so `softphone.history.RecordCall` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `call_id` (`softphone.control.CallId`) ← the event's `call_id` (`softphone.control.CallId`).

## `record-rejected-call`

A call the user declined reaches the log.

`softphone.control.CallRejected` causes [`softphone.history.RecordCall`](domains/softphone-history.md#recordcall).

```mermaid
flowchart LR
    event["softphone.control.CallRejected"]
    command["softphone.history.RecordCall"]
    event -->|"record-rejected-call"| command
    outcome0["recorded"]
    command --> outcome0
    emit0_0["softphone.history.CallRecorded"]
    outcome0 --> emit0_0
```

Delivered **at least once**, so `softphone.history.RecordCall` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `call_id` (`softphone.control.CallId`) ← the event's `call_id` (`softphone.control.CallId`).

## `show-incoming-call`

An arriving call puts itself on screen.

`softphone.control.CallOffered` causes [`softphone.presentation.ShowCall`](domains/softphone-presentation.md#showcall).

```mermaid
flowchart LR
    event["softphone.control.CallOffered"]
    command["softphone.presentation.ShowCall"]
    event -->|"show-incoming-call"| command
    outcome0["shown"]
    command --> outcome0
    emit0_0["softphone.presentation.CallShown"]
    outcome0 --> emit0_0
```

Delivered **at least once**, so `softphone.presentation.ShowCall` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `call_id` (`softphone.control.CallId`) ← the event's `call_id` (`softphone.control.CallId`).

## `show-outbound-call`

A call being placed puts itself on screen.

`softphone.control.CallDialled` causes [`softphone.presentation.ShowCall`](domains/softphone-presentation.md#showcall).

```mermaid
flowchart LR
    event["softphone.control.CallDialled"]
    command["softphone.presentation.ShowCall"]
    event -->|"show-outbound-call"| command
    outcome0["shown"]
    command --> outcome0
    emit0_0["softphone.presentation.CallShown"]
    outcome0 --> emit0_0
```

Delivered **at least once**, so `softphone.presentation.ShowCall` must be idempotent: the same event arriving twice must not do the work twice. "Exactly once" is what everyone believes they have until a retry proves otherwise, which is why this is written down rather than assumed.

When it fails it is **retried**, on whatever schedule the transport provides. Nothing here says how many times, so nothing here says when it stops. A retry publishes nothing of its own, because it is already observable: it is another invocation of the command.

It fills the command's input like this:

- `call_id` (`softphone.control.CallId`) ← the event's `call_id` (`softphone.control.CallId`).

## Events nothing reacts to

Legal, and worth seeing. An event with no reader inside the system is either a deliberate boundary — something outside consumes it — or a binding somebody forgot, and only a person can tell which.

- `softphone.bridge.BridgeConnecting`
- `softphone.control.CallAnswered`
- `softphone.control.CallConfirmed`
- `softphone.control.CallEstablished`
- `softphone.control.CallRinging`
- `softphone.control.DigitsSent`
- `softphone.control.EndpointConfigured`
- `softphone.control.HoldChanged`
- `softphone.control.MediaAttached`
- `softphone.control.MuteChanged`
- `softphone.directory.ContactAdded`
- `softphone.directory.ContactAddressAdded`
- `softphone.directory.ContactAddressRemoved`
- `softphone.directory.ContactDeleted`
- `softphone.directory.ContactRenamed`
- `softphone.history.CallRecorded`
- `softphone.history.RecordAttributed`
- `softphone.history.RecordDeleted`
- `softphone.local.LoopbackAttached`
- `softphone.media.OutputInterrupted`
- `softphone.media.SessionActivated`
- `softphone.media.SessionOpened`
- `softphone.media.SessionTerminated`
- `softphone.media.SignalReceived`
- `softphone.media.SignalSent`
- `softphone.presence.PeerGone`
- `softphone.presence.PeerPresent`
- `softphone.presence.PresenceAnnounced`
- `softphone.presence.PresenceConfirmed`
- `softphone.presence.PresenceFailed`
- `softphone.presence.PresenceWithdrawn`
- `softphone.presentation.CallDismissed`
- `softphone.presentation.CallShown`
- `softphone.presentation.ConsoleDialing`
- `softphone.presentation.ConsoleEngaged`
- `softphone.presentation.ConsoleIncoming`
- `softphone.presentation.ConsoleOpened`
- `softphone.presentation.ConsoleReleased`
- `softphone.presentation.EntryCleared`
- `softphone.presentation.KeyPressed`
- `softphone.presentation.KeypadOpened`
- `softphone.presentation.TileAttributed`
- `softphone.sip.LocalMediaFailed`
- `softphone.sip.LocalMediaOffered`
- `softphone.sip.LocalMediaRequested`
- `softphone.sip.RegistrationConfirmed`
- `softphone.sip.RegistrationEnded`
- `softphone.sip.RegistrationFailed`
- `softphone.sip.RegistrationRefreshed`
- `softphone.sip.RegistrationRequested`
- `softphone.sip.RegistrationRetried`
- `softphone.sip.RemoteMediaApplied`
- `softphone.sip.SipDialogOpened`


---

Generated from softphone v1 · model digest `522f372d9f45cae05577d8379ac02cac26b458beceb66dea739471ba4f91e59e` · contract digest `a716fbbbe26b7fdd24cf69b5441a8a7a57b304f349ef2e63e6705b0c329ffd20`. Do not edit this file; change the specification and regenerate it with `ess generate`.
