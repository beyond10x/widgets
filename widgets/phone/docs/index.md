<!--
generated from softphone v1
model digest aba1da1149fb1642c433b23af295d777adcc159bd748b60ca1a5d9a812a4becf
contract digest ef93fa7f8584a1ef672990779da742c4036cf471dd4a5ce03ec96443be9da643
do not edit: regenerate with `ess generate`
-->

# softphone v1

A softphone in four layers: a protocol-neutral media session, a control surface a human or an agent drives identically, what it recorded and who it can call, and a presentation over both. Two bindings carry the media session, and neither appears in it.

## The system as a graph

```mermaid
flowchart TB
    subgraph who["who may ask"]
        who0["softphone.control.Agent"]
        who1["softphone.control.Human"]
        who2["softphone.control.Kernel"]
        who3["softphone.directory.Human"]
        who4["softphone.history.HistoryHost"]
        who5["softphone.history.Human"]
        who6["softphone.local.LocalHost"]
        who7["softphone.media.SessionHost"]
        who8["softphone.presentation.Human"]
        who9["softphone.sip.Kernel"]
        who10["softphone.sip.SipHost"]
    end
    subgraph unit0["call-history"]
        cmd18["softphone.history.AttributeRecord"]
        cmd19["softphone.history.DeleteRecord"]
        cmd20["softphone.history.RecordCall"]
        evt18["softphone.history.CallRecorded"]
        evt19["softphone.history.RecordAttributed"]
        evt20["softphone.history.RecordDeleted"]
    end
    subgraph unit1["local-binding"]
        cmd21["softphone.local.AttachLoopback"]
        cmd22["softphone.local.DetachLoopback"]
        evt21["softphone.local.LoopbackAttached"]
        evt22["softphone.local.LoopbackDetached"]
    end
    subgraph unit2["media-session"]
        cmd23["softphone.media.ActivateSession"]
        cmd24["softphone.media.InterruptOutput"]
        cmd25["softphone.media.OpenSession"]
        cmd26["softphone.media.ReceiveSignal"]
        cmd27["softphone.media.SendSignal"]
        cmd28["softphone.media.TerminateSession"]
        evt23["softphone.media.OutputInterrupted"]
        evt24["softphone.media.SessionActivated"]
        evt25["softphone.media.SessionOpened"]
        evt26["softphone.media.SessionTerminated"]
        evt27["softphone.media.SignalReceived"]
        evt28["softphone.media.SignalSent"]
    end
    subgraph unit3["phone-console"]
        cmd29["softphone.presentation.AttributeTile"]
        cmd30["softphone.presentation.ClearEntry"]
        cmd31["softphone.presentation.DismissCall"]
        cmd37["softphone.presentation.OpenKeypad"]
        cmd38["softphone.presentation.PressKey"]
        cmd39["softphone.presentation.ShowCall"]
        evt29["softphone.presentation.CallDismissed"]
        evt30["softphone.presentation.CallShown"]
        evt36["softphone.presentation.EntryCleared"]
        evt37["softphone.presentation.KeyPressed"]
        evt38["softphone.presentation.KeypadOpened"]
        evt39["softphone.presentation.TileAttributed"]
    end
    subgraph unit4["phone-control"]
        cmd0["softphone.control.Answer"]
        cmd1["softphone.control.AttachMedia"]
        cmd2["softphone.control.ConfigureEndpoint"]
        cmd3["softphone.control.ConfirmAnswer"]
        cmd4["softphone.control.Dial"]
        cmd5["softphone.control.FailCall"]
        cmd6["softphone.control.HangUp"]
        cmd7["softphone.control.MediaConnected"]
        cmd8["softphone.control.OfferCall"]
        cmd9["softphone.control.Reject"]
        cmd10["softphone.control.RingCall"]
        cmd11["softphone.control.SendDigits"]
        cmd12["softphone.control.SetMuted"]
        evt0["softphone.control.CallAnswered"]
        evt1["softphone.control.CallConfirmed"]
        evt2["softphone.control.CallDialled"]
        evt3["softphone.control.CallEstablished"]
        evt4["softphone.control.CallFailed"]
        evt5["softphone.control.CallHungUp"]
        evt6["softphone.control.CallOffered"]
        evt7["softphone.control.CallRejected"]
        evt8["softphone.control.CallRinging"]
        evt9["softphone.control.DigitsSent"]
        evt10["softphone.control.EndpointConfigured"]
        evt11["softphone.control.MediaAttached"]
        evt12["softphone.control.MuteChanged"]
    end
    subgraph unit5["phone-directory"]
        cmd13["softphone.directory.AddAddress"]
        cmd14["softphone.directory.AddContact"]
        cmd15["softphone.directory.DeleteContact"]
        cmd16["softphone.directory.RemoveAddress"]
        cmd17["softphone.directory.RenameContact"]
        evt13["softphone.directory.ContactAdded"]
        evt14["softphone.directory.ContactAddressAdded"]
        evt15["softphone.directory.ContactAddressRemoved"]
        evt16["softphone.directory.ContactDeleted"]
        evt17["softphone.directory.ContactRenamed"]
    end
    subgraph unit6["sip-binding"]
        cmd40["softphone.sip.ApplyRemoteMedia"]
        cmd41["softphone.sip.CloseDialog"]
        cmd42["softphone.sip.ConfirmRegistration"]
        cmd43["softphone.sip.FailLocalMedia"]
        cmd44["softphone.sip.FailRegistration"]
        cmd45["softphone.sip.OfferLocalMedia"]
        cmd46["softphone.sip.OpenDialog"]
        cmd47["softphone.sip.RefreshRegistration"]
        cmd48["softphone.sip.Register"]
        cmd49["softphone.sip.RequestLocalMedia"]
        cmd50["softphone.sip.RetryRegistration"]
        cmd51["softphone.sip.Unregister"]
        evt40["softphone.sip.LocalMediaFailed"]
        evt41["softphone.sip.LocalMediaOffered"]
        evt42["softphone.sip.LocalMediaRequested"]
        evt43["softphone.sip.RegistrationConfirmed"]
        evt44["softphone.sip.RegistrationEnded"]
        evt45["softphone.sip.RegistrationFailed"]
        evt46["softphone.sip.RegistrationRefreshed"]
        evt47["softphone.sip.RegistrationRequested"]
        evt48["softphone.sip.RegistrationRetried"]
        evt49["softphone.sip.RemoteMediaApplied"]
        evt50["softphone.sip.SipDialogClosed"]
        evt51["softphone.sip.SipDialogOpened"]
    end
    subgraph unowned["owned by no component"]
        cmd32["softphone.presentation.EnterCall"]
        cmd33["softphone.presentation.EnterDialing"]
        cmd34["softphone.presentation.EnterIncoming"]
        cmd35["softphone.presentation.LeaveCall"]
        cmd36["softphone.presentation.OpenConsole"]
        evt31["softphone.presentation.ConsoleDialing"]
        evt32["softphone.presentation.ConsoleEngaged"]
        evt33["softphone.presentation.ConsoleIncoming"]
        evt34["softphone.presentation.ConsoleOpened"]
        evt35["softphone.presentation.ConsoleReleased"]
    end
    who0 -->|"may invoke"| cmd0
    who0 -->|"may invoke"| cmd2
    who0 -->|"may invoke"| cmd4
    who0 -->|"may invoke"| cmd6
    who0 -->|"may invoke"| cmd9
    who0 -->|"may invoke"| cmd11
    who0 -->|"may invoke"| cmd12
    who1 -->|"may invoke"| cmd0
    who1 -->|"may invoke"| cmd2
    who1 -->|"may invoke"| cmd4
    who1 -->|"may invoke"| cmd6
    who1 -->|"may invoke"| cmd9
    who1 -->|"may invoke"| cmd11
    who1 -->|"may invoke"| cmd12
    who2 -->|"may invoke"| cmd1
    who2 -->|"may invoke"| cmd3
    who2 -->|"may invoke"| cmd5
    who2 -->|"may invoke"| cmd7
    who2 -->|"may invoke"| cmd8
    who2 -->|"may invoke"| cmd10
    who3 -->|"may invoke"| cmd13
    who3 -->|"may invoke"| cmd14
    who3 -->|"may invoke"| cmd15
    who3 -->|"may invoke"| cmd16
    who3 -->|"may invoke"| cmd17
    who4 -->|"may invoke"| cmd20
    who5 -->|"may invoke"| cmd18
    who5 -->|"may invoke"| cmd19
    who6 -->|"may invoke"| cmd21
    who6 -->|"may invoke"| cmd22
    who7 -->|"may invoke"| cmd23
    who7 -->|"may invoke"| cmd24
    who7 -->|"may invoke"| cmd25
    who7 -->|"may invoke"| cmd26
    who7 -->|"may invoke"| cmd27
    who7 -->|"may invoke"| cmd28
    who8 -->|"may invoke"| cmd29
    who8 -->|"may invoke"| cmd30
    who8 -->|"may invoke"| cmd31
    who8 -->|"may invoke"| cmd32
    who8 -->|"may invoke"| cmd33
    who8 -->|"may invoke"| cmd34
    who8 -->|"may invoke"| cmd35
    who8 -->|"may invoke"| cmd36
    who8 -->|"may invoke"| cmd37
    who8 -->|"may invoke"| cmd38
    who8 -->|"may invoke"| cmd39
    who9 -->|"may invoke"| cmd40
    who9 -->|"may invoke"| cmd41
    who9 -->|"may invoke"| cmd42
    who9 -->|"may invoke"| cmd44
    who9 -->|"may invoke"| cmd49
    who10 -->|"may invoke"| cmd41
    who10 -->|"may invoke"| cmd43
    who10 -->|"may invoke"| cmd45
    who10 -->|"may invoke"| cmd46
    who10 -->|"may invoke"| cmd47
    who10 -->|"may invoke"| cmd48
    who10 -->|"may invoke"| cmd50
    who10 -->|"may invoke"| cmd51
    cmd0 -->|"answered"| evt0
    cmd1 -->|"attached"| evt11
    cmd2 -->|"configured"| evt10
    cmd3 -->|"confirmed"| evt1
    cmd4 -->|"dialled"| evt2
    cmd5 -->|"failed"| evt4
    cmd6 -->|"hung-up"| evt5
    cmd7 -->|"established"| evt3
    cmd8 -->|"offered"| evt6
    cmd9 -->|"rejected"| evt7
    cmd10 -->|"ringing"| evt8
    cmd11 -->|"sent"| evt9
    cmd12 -->|"changed"| evt12
    cmd13 -->|"added"| evt14
    cmd14 -->|"added"| evt13
    cmd15 -->|"deleted"| evt16
    cmd16 -->|"removed"| evt15
    cmd17 -->|"renamed"| evt17
    cmd18 -->|"attributed"| evt19
    cmd19 -->|"deleted"| evt20
    cmd20 -->|"recorded"| evt18
    cmd21 -->|"attached"| evt21
    cmd22 -->|"detached"| evt22
    cmd23 -->|"activated"| evt24
    cmd24 -->|"interrupted"| evt23
    cmd25 -->|"opened"| evt25
    cmd26 -->|"received"| evt27
    cmd27 -->|"sent"| evt28
    cmd28 -->|"terminated"| evt26
    cmd29 -->|"attributed"| evt39
    cmd30 -->|"cleared"| evt36
    cmd31 -->|"dismissed"| evt29
    cmd32 -->|"engaged"| evt32
    cmd33 -->|"dialing"| evt31
    cmd34 -->|"incoming"| evt33
    cmd35 -->|"released"| evt35
    cmd36 -->|"opened"| evt34
    cmd37 -->|"opened"| evt38
    cmd38 -->|"pressed"| evt37
    cmd39 -->|"shown"| evt30
    cmd40 -->|"applied"| evt49
    cmd41 -->|"closed"| evt50
    cmd42 -->|"confirmed"| evt43
    cmd43 -->|"failed"| evt40
    cmd44 -->|"failed"| evt45
    cmd45 -->|"offered"| evt41
    cmd46 -->|"opened"| evt51
    cmd47 -->|"refreshed"| evt46
    cmd48 -->|"requested"| evt47
    cmd49 -->|"requested"| evt42
    cmd50 -->|"retried"| evt48
    cmd51 -->|"ended"| evt44
    evt50 -.->|"end-session-with-dialog"| cmd28
    evt22 -.->|"end-session-with-loopback"| cmd28
    evt4 -.->|"record-failed-call"| cmd20
    evt5 -.->|"record-hung-up-call"| cmd20
    evt7 -.->|"record-rejected-call"| cmd20
    evt6 -.->|"show-incoming-call"| cmd39
    evt2 -.->|"show-outbound-call"| cmd39
```

A command is accepted by the component that owns its context, emits the events one of its outcomes declares, and a dashed edge is a binding carrying an event into the next command. Design §9 begins one step earlier, at the actor who invokes the first command, and so does this graph: a solid edge out of an actor is a grant, and an actor drawn with no edge at all may invoke nothing — which is something the model says, not an arrow somebody forgot.

## Bounded contexts

- **[Phone control](domains/softphone-control.md)** (`softphone.control`) — Calls this phone is placing or receiving, and the commands a human or an agent issues identically to drive them. No protocol type and no contact appears here. Nine types, two entities, three views, 13 commands, 13 events, one error and three actors.
- **[Phonebook](domains/softphone-directory.md)** (`softphone.directory`) — Contacts and the addresses they can be reached at. Owned by the person using the phone; the control surface never names a contact. Five types, two entities, three views, five commands, five events, two errors and one actor.
- **[Call history](domains/softphone-history.md)** (`softphone.history`) — One durable record per finished call, in the neutral termination vocabulary, optionally attributed to a contact. Deleted one record at a time; there is no bulk erase. Two types, one entity, two views, three commands, three events, one error and two actors.
- **[Local binding](domains/softphone-local.md)** (`softphone.local`) — A media session carried by local audio devices. No network, no signalling, no registration — the binding whose existence is the evidence that the media layer is not the SIP layer. Three types, one entity, one view, two commands, two events, one error and one actor.
- **[Media session](domains/softphone-media.md)** (`softphone.media`) — One admitted media session, its neutral audio profile, the signals crossing it and the one reason it ended. Agnostic to SIP, RTVBP and WebRTC by construction: no relation and no field type here points at a binding domain. 11 types, one entity, two views, six commands, six events, one error and one actor.
- **[Phone console](domains/softphone-presentation.md)** (`softphone.presentation`) — A keypad holding a partially typed address, and one tile per call carrying the contact name to show. The only domain a page reads directly. Six types, three entities, three views, 11 commands, 11 events, two errors and one actor.
- **[SIP binding](domains/softphone-sip.md)** (`softphone.sip`) — A SIP registration and the dialogs carrying media sessions over it. The only domain that names SIP, SDP, an address-of-record or a WebSocket. The offer/answer exchange lives here as four commands and the two descriptions a dialog carries. 10 types, two entities, two views, 12 commands, 12 events, two errors and two actors.

## Components

A component is a unit of ownership, not a deployment. How many of each runs, and what each needs, is [the topology](topology.md).

**`call-history`** — Owns the durable record of every finished call. It owns [`softphone.history`](domains/softphone-history.md). It accepts `softphone.history.AttributeRecord`, `softphone.history.DeleteRecord` and `softphone.history.RecordCall`. It publishes `softphone.history.CallRecorded`, `softphone.history.RecordAttributed` and `softphone.history.RecordDeleted`.

**`local-binding`** — Owns the local-audio binding, the second carrier the media layer is proved against. It owns [`softphone.local`](domains/softphone-local.md). It accepts `softphone.local.AttachLoopback` and `softphone.local.DetachLoopback`. It publishes `softphone.local.LoopbackAttached` and `softphone.local.LoopbackDetached`.

**`media-session`** — Owns the protocol-neutral media session and the signals crossing it. It owns [`softphone.media`](domains/softphone-media.md). It accepts `softphone.media.ActivateSession`, `softphone.media.InterruptOutput`, `softphone.media.OpenSession`, `softphone.media.ReceiveSignal`, `softphone.media.SendSignal` and `softphone.media.TerminateSession`. It publishes `softphone.media.OutputInterrupted`, `softphone.media.SessionActivated`, `softphone.media.SessionOpened`, `softphone.media.SessionTerminated`, `softphone.media.SignalReceived` and `softphone.media.SignalSent`.

**`phone-console`** — Owns what is on screen and what a gesture means. It owns [`softphone.presentation`](domains/softphone-presentation.md). It accepts `softphone.presentation.AttributeTile`, `softphone.presentation.ClearEntry`, `softphone.presentation.DismissCall`, `softphone.presentation.OpenKeypad`, `softphone.presentation.PressKey` and `softphone.presentation.ShowCall`. It publishes `softphone.presentation.CallDismissed`, `softphone.presentation.CallShown`, `softphone.presentation.EntryCleared`, `softphone.presentation.KeyPressed`, `softphone.presentation.KeypadOpened` and `softphone.presentation.TileAttributed`.

**`phone-control`** — Owns the call surface a human or an agent drives identically. It owns [`softphone.control`](domains/softphone-control.md). It accepts `softphone.control.Answer`, `softphone.control.AttachMedia`, `softphone.control.ConfigureEndpoint`, `softphone.control.ConfirmAnswer`, `softphone.control.Dial`, `softphone.control.FailCall`, `softphone.control.HangUp`, `softphone.control.MediaConnected`, `softphone.control.OfferCall`, `softphone.control.Reject`, `softphone.control.RingCall`, `softphone.control.SendDigits` and `softphone.control.SetMuted`. It publishes `softphone.control.CallAnswered`, `softphone.control.CallConfirmed`, `softphone.control.CallDialled`, `softphone.control.CallEstablished`, `softphone.control.CallFailed`, `softphone.control.CallHungUp`, `softphone.control.CallOffered`, `softphone.control.CallRejected`, `softphone.control.CallRinging`, `softphone.control.DigitsSent`, `softphone.control.EndpointConfigured`, `softphone.control.MediaAttached` and `softphone.control.MuteChanged`.

**`phone-directory`** — Owns the phonebook. It owns [`softphone.directory`](domains/softphone-directory.md). It accepts `softphone.directory.AddAddress`, `softphone.directory.AddContact`, `softphone.directory.DeleteContact`, `softphone.directory.RemoveAddress` and `softphone.directory.RenameContact`. It publishes `softphone.directory.ContactAdded`, `softphone.directory.ContactAddressAdded`, `softphone.directory.ContactAddressRemoved`, `softphone.directory.ContactDeleted` and `softphone.directory.ContactRenamed`.

**`sip-binding`** — Owns the SIP registration and the dialogs carrying media sessions over it. It owns [`softphone.sip`](domains/softphone-sip.md). It accepts `softphone.sip.ApplyRemoteMedia`, `softphone.sip.CloseDialog`, `softphone.sip.ConfirmRegistration`, `softphone.sip.FailLocalMedia`, `softphone.sip.FailRegistration`, `softphone.sip.OfferLocalMedia`, `softphone.sip.OpenDialog`, `softphone.sip.RefreshRegistration`, `softphone.sip.Register`, `softphone.sip.RequestLocalMedia`, `softphone.sip.RetryRegistration` and `softphone.sip.Unregister`. It publishes `softphone.sip.LocalMediaFailed`, `softphone.sip.LocalMediaOffered`, `softphone.sip.LocalMediaRequested`, `softphone.sip.RegistrationConfirmed`, `softphone.sip.RegistrationEnded`, `softphone.sip.RegistrationFailed`, `softphone.sip.RegistrationRefreshed`, `softphone.sip.RegistrationRequested`, `softphone.sip.RegistrationRetried`, `softphone.sip.RemoteMediaApplied`, `softphone.sip.SipDialogClosed` and `softphone.sip.SipDialogOpened`.

## The other pages

| page | what is on it |
|---|---|
| [Phone control](domains/softphone-control.md) | the `softphone.control` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Phonebook](domains/softphone-directory.md) | the `softphone.directory` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Call history](domains/softphone-history.md) | the `softphone.history` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Local binding](domains/softphone-local.md) | the `softphone.local` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Media session](domains/softphone-media.md) | the `softphone.media` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Phone console](domains/softphone-presentation.md) | the `softphone.presentation` vocabulary: its types, entities, views, commands, events, errors and actors |
| [SIP binding](domains/softphone-sip.md) | the `softphone.sip` vocabulary: its types, entities, views, commands, events, errors and actors |
| [Interactions](interactions.md) | every binding, with what it guarantees and what happens when it fails |
| [Type crossings](crossings.md) | every conversion this system permits, and the reason someone gave for it |
| [Topology](topology.md) | what each component needs in order to run |


---

Generated from softphone v1 · model digest `aba1da1149fb1642c433b23af295d777adcc159bd748b60ca1a5d9a812a4becf` · contract digest `ef93fa7f8584a1ef672990779da742c4036cf471dd4a5ce03ec96443be9da643`. Do not edit this file; change the specification and regenerate it with `ess generate`.
