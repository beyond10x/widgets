<!--
generated from softphone v1
model digest 522f372d9f45cae05577d8379ac02cac26b458beceb66dea739471ba4f91e59e
contract digest a716fbbbe26b7fdd24cf69b5441a8a7a57b304f349ef2e63e6705b0c329ffd20
do not edit: regenerate with `ess generate`
-->

# softphone v1

A softphone in four layers: a protocol-neutral media session, a control surface a human or an agent drives identically, what it recorded and who it can call, and a presentation over both. Two bindings carry the media session, and neither appears in it.

## The system as a graph

```mermaid
flowchart TB
    subgraph who["who may ask"]
        who0["softphone.bridge.BridgeHost"]
        who1["softphone.bridge.Kernel"]
        who2["softphone.control.Agent"]
        who3["softphone.control.Human"]
        who4["softphone.control.Kernel"]
        who5["softphone.directory.Human"]
        who6["softphone.history.HistoryHost"]
        who7["softphone.history.Human"]
        who8["softphone.local.LocalHost"]
        who9["softphone.media.SessionHost"]
        who10["softphone.presence.Kernel"]
        who11["softphone.presence.PresenceHost"]
        who12["softphone.presentation.Human"]
        who13["softphone.sip.Kernel"]
        who14["softphone.sip.SipHost"]
    end
    subgraph unit0["bridge-binding"]
        cmd0["softphone.bridge.CloseBridge"]
        cmd1["softphone.bridge.ConfirmBridge"]
        cmd2["softphone.bridge.ConnectBridge"]
        cmd3["softphone.bridge.FailBridge"]
        evt0["softphone.bridge.BridgeClosed"]
        evt1["softphone.bridge.BridgeConfirmed"]
        evt2["softphone.bridge.BridgeConnecting"]
        evt3["softphone.bridge.BridgeFailed"]
    end
    subgraph unit1["call-history"]
        cmd23["softphone.history.AttributeRecord"]
        cmd24["softphone.history.DeleteRecord"]
        cmd25["softphone.history.RecordCall"]
        evt23["softphone.history.CallRecorded"]
        evt24["softphone.history.RecordAttributed"]
        evt25["softphone.history.RecordDeleted"]
    end
    subgraph unit2["local-binding"]
        cmd26["softphone.local.AttachLoopback"]
        cmd27["softphone.local.DetachLoopback"]
        evt26["softphone.local.LoopbackAttached"]
        evt27["softphone.local.LoopbackDetached"]
    end
    subgraph unit3["media-session"]
        cmd28["softphone.media.ActivateSession"]
        cmd29["softphone.media.InterruptOutput"]
        cmd30["softphone.media.OpenSession"]
        cmd31["softphone.media.ReceiveSignal"]
        cmd32["softphone.media.SendSignal"]
        cmd33["softphone.media.TerminateSession"]
        evt28["softphone.media.OutputInterrupted"]
        evt29["softphone.media.SessionActivated"]
        evt30["softphone.media.SessionOpened"]
        evt31["softphone.media.SessionTerminated"]
        evt32["softphone.media.SignalReceived"]
        evt33["softphone.media.SignalSent"]
    end
    subgraph unit4["phone-console"]
        cmd40["softphone.presentation.AttributeTile"]
        cmd41["softphone.presentation.ClearEntry"]
        cmd42["softphone.presentation.DismissCall"]
        cmd48["softphone.presentation.OpenKeypad"]
        cmd49["softphone.presentation.PressKey"]
        cmd50["softphone.presentation.ShowCall"]
        evt40["softphone.presentation.CallDismissed"]
        evt41["softphone.presentation.CallShown"]
        evt47["softphone.presentation.EntryCleared"]
        evt48["softphone.presentation.KeyPressed"]
        evt49["softphone.presentation.KeypadOpened"]
        evt50["softphone.presentation.TileAttributed"]
    end
    subgraph unit5["phone-control"]
        cmd4["softphone.control.Answer"]
        cmd5["softphone.control.AttachMedia"]
        cmd6["softphone.control.ConfigureEndpoint"]
        cmd7["softphone.control.ConfirmAnswer"]
        cmd8["softphone.control.Dial"]
        cmd9["softphone.control.FailCall"]
        cmd10["softphone.control.HangUp"]
        cmd11["softphone.control.MediaConnected"]
        cmd12["softphone.control.OfferCall"]
        cmd13["softphone.control.Reject"]
        cmd14["softphone.control.RingCall"]
        cmd15["softphone.control.SendDigits"]
        cmd16["softphone.control.SetHeld"]
        cmd17["softphone.control.SetMuted"]
        evt4["softphone.control.CallAnswered"]
        evt5["softphone.control.CallConfirmed"]
        evt6["softphone.control.CallDialled"]
        evt7["softphone.control.CallEstablished"]
        evt8["softphone.control.CallFailed"]
        evt9["softphone.control.CallHungUp"]
        evt10["softphone.control.CallOffered"]
        evt11["softphone.control.CallRejected"]
        evt12["softphone.control.CallRinging"]
        evt13["softphone.control.DigitsSent"]
        evt14["softphone.control.EndpointConfigured"]
        evt15["softphone.control.HoldChanged"]
        evt16["softphone.control.MediaAttached"]
        evt17["softphone.control.MuteChanged"]
    end
    subgraph unit6["phone-directory"]
        cmd18["softphone.directory.AddAddress"]
        cmd19["softphone.directory.AddContact"]
        cmd20["softphone.directory.DeleteContact"]
        cmd21["softphone.directory.RemoveAddress"]
        cmd22["softphone.directory.RenameContact"]
        evt18["softphone.directory.ContactAdded"]
        evt19["softphone.directory.ContactAddressAdded"]
        evt20["softphone.directory.ContactAddressRemoved"]
        evt21["softphone.directory.ContactDeleted"]
        evt22["softphone.directory.ContactRenamed"]
    end
    subgraph unit7["phone-presence"]
        cmd34["softphone.presence.AnnouncePresence"]
        cmd35["softphone.presence.ConfirmPresence"]
        cmd36["softphone.presence.FailPresence"]
        cmd37["softphone.presence.NoteGone"]
        cmd38["softphone.presence.NotePresent"]
        cmd39["softphone.presence.WithdrawPresence"]
        evt34["softphone.presence.PeerGone"]
        evt35["softphone.presence.PeerPresent"]
        evt36["softphone.presence.PresenceAnnounced"]
        evt37["softphone.presence.PresenceConfirmed"]
        evt38["softphone.presence.PresenceFailed"]
        evt39["softphone.presence.PresenceWithdrawn"]
    end
    subgraph unit8["sip-binding"]
        cmd51["softphone.sip.ApplyRemoteMedia"]
        cmd52["softphone.sip.CloseDialog"]
        cmd53["softphone.sip.ConfirmRegistration"]
        cmd54["softphone.sip.FailLocalMedia"]
        cmd55["softphone.sip.FailRegistration"]
        cmd56["softphone.sip.OfferLocalMedia"]
        cmd57["softphone.sip.OpenDialog"]
        cmd58["softphone.sip.RefreshRegistration"]
        cmd59["softphone.sip.Register"]
        cmd60["softphone.sip.RequestLocalMedia"]
        cmd61["softphone.sip.RetryRegistration"]
        cmd62["softphone.sip.Unregister"]
        evt51["softphone.sip.LocalMediaFailed"]
        evt52["softphone.sip.LocalMediaOffered"]
        evt53["softphone.sip.LocalMediaRequested"]
        evt54["softphone.sip.RegistrationConfirmed"]
        evt55["softphone.sip.RegistrationEnded"]
        evt56["softphone.sip.RegistrationFailed"]
        evt57["softphone.sip.RegistrationRefreshed"]
        evt58["softphone.sip.RegistrationRequested"]
        evt59["softphone.sip.RegistrationRetried"]
        evt60["softphone.sip.RemoteMediaApplied"]
        evt61["softphone.sip.SipDialogClosed"]
        evt62["softphone.sip.SipDialogOpened"]
    end
    subgraph unowned["owned by no component"]
        cmd43["softphone.presentation.EnterCall"]
        cmd44["softphone.presentation.EnterDialing"]
        cmd45["softphone.presentation.EnterIncoming"]
        cmd46["softphone.presentation.LeaveCall"]
        cmd47["softphone.presentation.OpenConsole"]
        evt42["softphone.presentation.ConsoleDialing"]
        evt43["softphone.presentation.ConsoleEngaged"]
        evt44["softphone.presentation.ConsoleIncoming"]
        evt45["softphone.presentation.ConsoleOpened"]
        evt46["softphone.presentation.ConsoleReleased"]
    end
    who0 -->|"may invoke"| cmd0
    who0 -->|"may invoke"| cmd2
    who1 -->|"may invoke"| cmd1
    who1 -->|"may invoke"| cmd3
    who2 -->|"may invoke"| cmd4
    who2 -->|"may invoke"| cmd6
    who2 -->|"may invoke"| cmd8
    who2 -->|"may invoke"| cmd10
    who2 -->|"may invoke"| cmd13
    who2 -->|"may invoke"| cmd15
    who2 -->|"may invoke"| cmd16
    who2 -->|"may invoke"| cmd17
    who3 -->|"may invoke"| cmd4
    who3 -->|"may invoke"| cmd6
    who3 -->|"may invoke"| cmd8
    who3 -->|"may invoke"| cmd10
    who3 -->|"may invoke"| cmd13
    who3 -->|"may invoke"| cmd15
    who3 -->|"may invoke"| cmd16
    who3 -->|"may invoke"| cmd17
    who4 -->|"may invoke"| cmd5
    who4 -->|"may invoke"| cmd7
    who4 -->|"may invoke"| cmd9
    who4 -->|"may invoke"| cmd11
    who4 -->|"may invoke"| cmd12
    who4 -->|"may invoke"| cmd14
    who5 -->|"may invoke"| cmd18
    who5 -->|"may invoke"| cmd19
    who5 -->|"may invoke"| cmd20
    who5 -->|"may invoke"| cmd21
    who5 -->|"may invoke"| cmd22
    who6 -->|"may invoke"| cmd25
    who7 -->|"may invoke"| cmd23
    who7 -->|"may invoke"| cmd24
    who8 -->|"may invoke"| cmd26
    who8 -->|"may invoke"| cmd27
    who9 -->|"may invoke"| cmd28
    who9 -->|"may invoke"| cmd29
    who9 -->|"may invoke"| cmd30
    who9 -->|"may invoke"| cmd31
    who9 -->|"may invoke"| cmd32
    who9 -->|"may invoke"| cmd33
    who10 -->|"may invoke"| cmd35
    who10 -->|"may invoke"| cmd36
    who10 -->|"may invoke"| cmd37
    who10 -->|"may invoke"| cmd38
    who11 -->|"may invoke"| cmd34
    who11 -->|"may invoke"| cmd39
    who12 -->|"may invoke"| cmd40
    who12 -->|"may invoke"| cmd41
    who12 -->|"may invoke"| cmd42
    who12 -->|"may invoke"| cmd43
    who12 -->|"may invoke"| cmd44
    who12 -->|"may invoke"| cmd45
    who12 -->|"may invoke"| cmd46
    who12 -->|"may invoke"| cmd47
    who12 -->|"may invoke"| cmd48
    who12 -->|"may invoke"| cmd49
    who12 -->|"may invoke"| cmd50
    who13 -->|"may invoke"| cmd51
    who13 -->|"may invoke"| cmd52
    who13 -->|"may invoke"| cmd53
    who13 -->|"may invoke"| cmd55
    who13 -->|"may invoke"| cmd60
    who14 -->|"may invoke"| cmd52
    who14 -->|"may invoke"| cmd54
    who14 -->|"may invoke"| cmd56
    who14 -->|"may invoke"| cmd57
    who14 -->|"may invoke"| cmd58
    who14 -->|"may invoke"| cmd59
    who14 -->|"may invoke"| cmd61
    who14 -->|"may invoke"| cmd62
    cmd0 -->|"closed"| evt0
    cmd1 -->|"confirmed"| evt1
    cmd2 -->|"connecting"| evt2
    cmd3 -->|"failed"| evt3
    cmd4 -->|"answered"| evt4
    cmd5 -->|"attached"| evt16
    cmd6 -->|"configured"| evt14
    cmd7 -->|"confirmed"| evt5
    cmd8 -->|"dialled"| evt6
    cmd9 -->|"failed"| evt8
    cmd10 -->|"hung-up"| evt9
    cmd11 -->|"established"| evt7
    cmd12 -->|"offered"| evt10
    cmd13 -->|"rejected"| evt11
    cmd14 -->|"ringing"| evt12
    cmd15 -->|"sent"| evt13
    cmd16 -->|"changed"| evt15
    cmd17 -->|"changed"| evt17
    cmd18 -->|"added"| evt19
    cmd19 -->|"added"| evt18
    cmd20 -->|"deleted"| evt21
    cmd21 -->|"removed"| evt20
    cmd22 -->|"renamed"| evt22
    cmd23 -->|"attributed"| evt24
    cmd24 -->|"deleted"| evt25
    cmd25 -->|"recorded"| evt23
    cmd26 -->|"attached"| evt26
    cmd27 -->|"detached"| evt27
    cmd28 -->|"activated"| evt29
    cmd29 -->|"interrupted"| evt28
    cmd30 -->|"opened"| evt30
    cmd31 -->|"received"| evt32
    cmd32 -->|"sent"| evt33
    cmd33 -->|"terminated"| evt31
    cmd34 -->|"announcing"| evt36
    cmd35 -->|"confirmed"| evt37
    cmd36 -->|"failed"| evt38
    cmd37 -->|"noted"| evt34
    cmd38 -->|"noted"| evt35
    cmd39 -->|"withdrawn"| evt39
    cmd40 -->|"attributed"| evt50
    cmd41 -->|"cleared"| evt47
    cmd42 -->|"dismissed"| evt40
    cmd43 -->|"engaged"| evt43
    cmd44 -->|"dialing"| evt42
    cmd45 -->|"incoming"| evt44
    cmd46 -->|"released"| evt46
    cmd47 -->|"opened"| evt45
    cmd48 -->|"opened"| evt49
    cmd49 -->|"pressed"| evt48
    cmd50 -->|"shown"| evt41
    cmd51 -->|"applied"| evt60
    cmd52 -->|"closed"| evt61
    cmd53 -->|"confirmed"| evt54
    cmd54 -->|"failed"| evt51
    cmd55 -->|"failed"| evt56
    cmd56 -->|"offered"| evt52
    cmd57 -->|"opened"| evt62
    cmd58 -->|"refreshed"| evt57
    cmd59 -->|"requested"| evt58
    cmd60 -->|"requested"| evt53
    cmd61 -->|"retried"| evt59
    cmd62 -->|"ended"| evt55
    evt1 -.->|"activate-session-with-bridge"| cmd28
    evt0 -.->|"end-session-with-bridge"| cmd33
    evt61 -.->|"end-session-with-dialog"| cmd33
    evt3 -.->|"end-session-with-failed-bridge"| cmd33
    evt27 -.->|"end-session-with-loopback"| cmd33
    evt8 -.->|"record-failed-call"| cmd25
    evt9 -.->|"record-hung-up-call"| cmd25
    evt11 -.->|"record-rejected-call"| cmd25
    evt10 -.->|"show-incoming-call"| cmd50
    evt6 -.->|"show-outbound-call"| cmd50
```

A command is accepted by the component that owns its context, emits the events one of its outcomes declares, and a dashed edge is a binding carrying an event into the next command. Design §9 begins one step earlier, at the actor who invokes the first command, and so does this graph: a solid edge out of an actor is a grant, and an actor drawn with no edge at all may invoke nothing — which is something the model says, not an arrow somebody forgot.

## Bounded contexts

- **[Media bridge](domains/softphone-bridge.md)** (`softphone.bridge`) — One media bridge from this phone to the server that holds its SIP leg: where it points, the offer it sent, the answer it got and why it closed. No SIP construct appears here. Five types, one entity, two views, four commands, four events, one error and two actors.
- **[Phone control](domains/softphone-control.md)** (`softphone.control`) — Calls this phone is placing or receiving, and the commands a human or an agent issues identically to drive them. No protocol type and no contact appears here. Nine types, two entities, three views, 14 commands, 14 events, one error and three actors.
- **[Phonebook](domains/softphone-directory.md)** (`softphone.directory`) — Contacts and the addresses they can be reached at. Owned by the person using the phone; the control surface never names a contact. Five types, two entities, three views, five commands, five events, two errors and one actor.
- **[Call history](domains/softphone-history.md)** (`softphone.history`) — One durable record per finished call, in the neutral termination vocabulary, optionally attributed to a contact. Deleted one record at a time; there is no bulk erase. Two types, one entity, two views, three commands, three events, one error and two actors.
- **[Local binding](domains/softphone-local.md)** (`softphone.local`) — A media session carried by local audio devices. No network, no signalling, no registration — the binding whose existence is the evidence that the media layer is not the SIP layer. Three types, one entity, one view, two commands, two events, one error and one actor.
- **[Media session](domains/softphone-media.md)** (`softphone.media`) — One admitted media session, its neutral audio profile, the signals crossing it and the one reason it ended. Agnostic to SIP, RTVBP and WebRTC by construction: no relation and no field type here points at a binding domain. 11 types, one entity, two views, six commands, six events, one error and one actor.
- **[Presence](domains/softphone-presence.md)** (`softphone.presence`) — This phone's standing with the server it announced itself to, and the other phones it has been told about. One roster per page, told rather than shared. Six types, two entities, three views, six commands, six events, two errors and two actors.
- **[Phone console](domains/softphone-presentation.md)** (`softphone.presentation`) — A keypad holding a partially typed address, and one tile per call carrying the contact name to show. The only domain a page reads directly. Six types, three entities, three views, 11 commands, 11 events, two errors and one actor.
- **[SIP binding](domains/softphone-sip.md)** (`softphone.sip`) — A SIP registration and the dialogs carrying media sessions over it. The only domain that names SIP, SDP, an address-of-record or a WebSocket. The offer/answer exchange lives here as four commands and the two descriptions a dialog carries. 10 types, two entities, two views, 12 commands, 12 events, two errors and two actors.

## Components

A component is a unit of ownership, not a deployment. How many of each runs, and what each needs, is [the topology](topology.md).

**`bridge-binding`** — Owns the bridge from this page to the server that holds its SIP leg. It owns [`softphone.bridge`](domains/softphone-bridge.md). It accepts `softphone.bridge.CloseBridge`, `softphone.bridge.ConfirmBridge`, `softphone.bridge.ConnectBridge` and `softphone.bridge.FailBridge`. It publishes `softphone.bridge.BridgeClosed`, `softphone.bridge.BridgeConfirmed`, `softphone.bridge.BridgeConnecting` and `softphone.bridge.BridgeFailed`.

**`call-history`** — Owns the durable record of every finished call. It owns [`softphone.history`](domains/softphone-history.md). It accepts `softphone.history.AttributeRecord`, `softphone.history.DeleteRecord` and `softphone.history.RecordCall`. It publishes `softphone.history.CallRecorded`, `softphone.history.RecordAttributed` and `softphone.history.RecordDeleted`.

**`local-binding`** — Owns the local-audio binding, the second carrier the media layer is proved against. It owns [`softphone.local`](domains/softphone-local.md). It accepts `softphone.local.AttachLoopback` and `softphone.local.DetachLoopback`. It publishes `softphone.local.LoopbackAttached` and `softphone.local.LoopbackDetached`.

**`media-session`** — Owns the protocol-neutral media session and the signals crossing it. It owns [`softphone.media`](domains/softphone-media.md). It accepts `softphone.media.ActivateSession`, `softphone.media.InterruptOutput`, `softphone.media.OpenSession`, `softphone.media.ReceiveSignal`, `softphone.media.SendSignal` and `softphone.media.TerminateSession`. It publishes `softphone.media.OutputInterrupted`, `softphone.media.SessionActivated`, `softphone.media.SessionOpened`, `softphone.media.SessionTerminated`, `softphone.media.SignalReceived` and `softphone.media.SignalSent`.

**`phone-console`** — Owns what is on screen and what a gesture means. It owns [`softphone.presentation`](domains/softphone-presentation.md). It accepts `softphone.presentation.AttributeTile`, `softphone.presentation.ClearEntry`, `softphone.presentation.DismissCall`, `softphone.presentation.OpenKeypad`, `softphone.presentation.PressKey` and `softphone.presentation.ShowCall`. It publishes `softphone.presentation.CallDismissed`, `softphone.presentation.CallShown`, `softphone.presentation.EntryCleared`, `softphone.presentation.KeyPressed`, `softphone.presentation.KeypadOpened` and `softphone.presentation.TileAttributed`.

**`phone-control`** — Owns the call surface a human or an agent drives identically. It owns [`softphone.control`](domains/softphone-control.md). It accepts `softphone.control.Answer`, `softphone.control.AttachMedia`, `softphone.control.ConfigureEndpoint`, `softphone.control.ConfirmAnswer`, `softphone.control.Dial`, `softphone.control.FailCall`, `softphone.control.HangUp`, `softphone.control.MediaConnected`, `softphone.control.OfferCall`, `softphone.control.Reject`, `softphone.control.RingCall`, `softphone.control.SendDigits`, `softphone.control.SetHeld` and `softphone.control.SetMuted`. It publishes `softphone.control.CallAnswered`, `softphone.control.CallConfirmed`, `softphone.control.CallDialled`, `softphone.control.CallEstablished`, `softphone.control.CallFailed`, `softphone.control.CallHungUp`, `softphone.control.CallOffered`, `softphone.control.CallRejected`, `softphone.control.CallRinging`, `softphone.control.DigitsSent`, `softphone.control.EndpointConfigured`, `softphone.control.HoldChanged`, `softphone.control.MediaAttached` and `softphone.control.MuteChanged`.

**`phone-directory`** — Owns the phonebook. It owns [`softphone.directory`](domains/softphone-directory.md). It accepts `softphone.directory.AddAddress`, `softphone.directory.AddContact`, `softphone.directory.DeleteContact`, `softphone.directory.RemoveAddress` and `softphone.directory.RenameContact`. It publishes `softphone.directory.ContactAdded`, `softphone.directory.ContactAddressAdded`, `softphone.directory.ContactAddressRemoved`, `softphone.directory.ContactDeleted` and `softphone.directory.ContactRenamed`.

**`phone-presence`** — Owns who else is there, as one roster this page holds and the server only reports. It owns [`softphone.presence`](domains/softphone-presence.md). It accepts `softphone.presence.AnnouncePresence`, `softphone.presence.ConfirmPresence`, `softphone.presence.FailPresence`, `softphone.presence.NoteGone`, `softphone.presence.NotePresent` and `softphone.presence.WithdrawPresence`. It publishes `softphone.presence.PeerGone`, `softphone.presence.PeerPresent`, `softphone.presence.PresenceAnnounced`, `softphone.presence.PresenceConfirmed`, `softphone.presence.PresenceFailed` and `softphone.presence.PresenceWithdrawn`.

**`sip-binding`** — Owns the SIP registration and the dialogs carrying media sessions over it. It owns [`softphone.sip`](domains/softphone-sip.md). It accepts `softphone.sip.ApplyRemoteMedia`, `softphone.sip.CloseDialog`, `softphone.sip.ConfirmRegistration`, `softphone.sip.FailLocalMedia`, `softphone.sip.FailRegistration`, `softphone.sip.OfferLocalMedia`, `softphone.sip.OpenDialog`, `softphone.sip.RefreshRegistration`, `softphone.sip.Register`, `softphone.sip.RequestLocalMedia`, `softphone.sip.RetryRegistration` and `softphone.sip.Unregister`. It publishes `softphone.sip.LocalMediaFailed`, `softphone.sip.LocalMediaOffered`, `softphone.sip.LocalMediaRequested`, `softphone.sip.RegistrationConfirmed`, `softphone.sip.RegistrationEnded`, `softphone.sip.RegistrationFailed`, `softphone.sip.RegistrationRefreshed`, `softphone.sip.RegistrationRequested`, `softphone.sip.RegistrationRetried`, `softphone.sip.RemoteMediaApplied`, `softphone.sip.SipDialogClosed` and `softphone.sip.SipDialogOpened`.

## The other pages

| page | what is on it |
|---|---|
| [Media bridge](domains/softphone-bridge.md) | the `softphone.bridge` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Phone control](domains/softphone-control.md) | the `softphone.control` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Phonebook](domains/softphone-directory.md) | the `softphone.directory` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Call history](domains/softphone-history.md) | the `softphone.history` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Local binding](domains/softphone-local.md) | the `softphone.local` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Media session](domains/softphone-media.md) | the `softphone.media` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Presence](domains/softphone-presence.md) | the `softphone.presence` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Phone console](domains/softphone-presentation.md) | the `softphone.presentation` vocabulary: its types, entities, views, commands, events, errors and actors |
| [SIP binding](domains/softphone-sip.md) | the `softphone.sip` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Interactions](interactions.md) | every binding, with what it guarantees and what happens when it fails |
| [Type crossings](crossings.md) | every conversion this system permits, and the reason someone gave for it |
| [Topology](topology.md) | what each component needs in order to run |


---

Generated from softphone v1 · model digest `522f372d9f45cae05577d8379ac02cac26b458beceb66dea739471ba4f91e59e` · contract digest `a716fbbbe26b7fdd24cf69b5441a8a7a57b304f349ef2e63e6705b0c329ffd20`. Do not edit this file; change the specification and regenerate it with `ess generate`.
