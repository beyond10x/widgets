<!--
  generated from softphone v1
  model digest 3d66648c3a80c2b5ba8ed8e6e8d9bfcbe91e4312afd50f56a5fea9297e7297a6
  contract digest b44b97c39f6c56280f3034e682da190a85a54d31c2b621422d55377b30a6864a
  do not edit: regenerate with `ess synthesize`
-->
# Synthesis plan — softphone v1

Scope: `component-skeletons`, planned by `ess-synth`. Regenerate with `ess synthesize`.

344 capabilities: **248 generated**, **75 obligations**, **21 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `softphone.bridge.BridgeCause` |
| domain type | `softphone.bridge.BridgeId` |
| domain type | `softphone.bridge.BridgeSession.State` |
| domain type | `softphone.bridge.BridgeSessionRow` |
| domain type | `softphone.bridge.ServerEndpoint` |
| domain type | `softphone.bridge.SessionDescription` |
| domain type | `softphone.control.Call.State` |
| domain type | `softphone.control.CallDirection` |
| domain type | `softphone.control.CallId` |
| domain type | `softphone.control.CallRow` |
| domain type | `softphone.control.DigitString` |
| domain type | `softphone.control.EndCause` |
| domain type | `softphone.control.EndpointId` |
| domain type | `softphone.control.EndpointRow` |
| domain type | `softphone.control.PhoneEndpoint.State` |
| domain type | `softphone.control.RejectStatus` |
| domain type | `softphone.control.RemoteAddress` |
| domain type | `softphone.directory.AddressLabel` |
| domain type | `softphone.directory.Contact.State` |
| domain type | `softphone.directory.ContactAddress.State` |
| domain type | `softphone.directory.ContactAddressId` |
| domain type | `softphone.directory.ContactAddressRow` |
| domain type | `softphone.directory.ContactId` |
| domain type | `softphone.directory.ContactRow` |
| domain type | `softphone.history.CallRecord.State` |
| domain type | `softphone.history.CallRecordId` |
| domain type | `softphone.history.CallRecordRow` |
| domain type | `softphone.local.DeviceRef` |
| domain type | `softphone.local.LoopbackDevice.State` |
| domain type | `softphone.local.LoopbackDeviceId` |
| domain type | `softphone.local.LoopbackDeviceRow` |
| domain type | `softphone.media.BindingKind` |
| domain type | `softphone.media.ChannelSignal` |
| domain type | `softphone.media.ContextTrust` |
| domain type | `softphone.media.DigitString` |
| domain type | `softphone.media.MediaProfile` |
| domain type | `softphone.media.MediaSession.State` |
| domain type | `softphone.media.MediaSessionId` |
| domain type | `softphone.media.MediaSessionRow` |
| domain type | `softphone.media.Participant` |
| domain type | `softphone.media.SampleFormat` |
| domain type | `softphone.media.SessionRef` |
| domain type | `softphone.media.TerminationReason` |
| domain type | `softphone.presentation.CallTile.State` |
| domain type | `softphone.presentation.CallTileId` |
| domain type | `softphone.presentation.CallTileRow` |
| domain type | `softphone.presentation.Console.State` |
| domain type | `softphone.presentation.ConsoleId` |
| domain type | `softphone.presentation.ConsoleRow` |
| domain type | `softphone.presentation.Keypad.State` |
| domain type | `softphone.presentation.KeypadId` |
| domain type | `softphone.presentation.KeypadRow` |
| domain type | `softphone.sip.AddressOfRecord` |
| domain type | `softphone.sip.MediaSecurity` |
| domain type | `softphone.sip.Registration.State` |
| domain type | `softphone.sip.RegistrationId` |
| domain type | `softphone.sip.RegistrationRow` |
| domain type | `softphone.sip.SessionDescription` |
| domain type | `softphone.sip.SignallingTransport` |
| domain type | `softphone.sip.SipCause` |
| domain type | `softphone.sip.SipDialog.State` |
| domain type | `softphone.sip.SipDialogId` |
| domain type | `softphone.sip.SipDialogRow` |
| domain type | `softphone.sip.SipUri` |
| entity lifecycle | `softphone.bridge.BridgeSession` |
| entity lifecycle | `softphone.control.Call` |
| entity lifecycle | `softphone.control.PhoneEndpoint` |
| entity lifecycle | `softphone.directory.Contact` |
| entity lifecycle | `softphone.directory.ContactAddress` |
| entity lifecycle | `softphone.history.CallRecord` |
| entity lifecycle | `softphone.local.LoopbackDevice` |
| entity lifecycle | `softphone.media.MediaSession` |
| entity lifecycle | `softphone.presentation.CallTile` |
| entity lifecycle | `softphone.presentation.Console` |
| entity lifecycle | `softphone.presentation.Keypad` |
| entity lifecycle | `softphone.sip.Registration` |
| entity lifecycle | `softphone.sip.SipDialog` |
| command contract | `softphone.bridge.CloseBridge` |
| command contract | `softphone.bridge.ConfirmBridge` |
| command contract | `softphone.bridge.ConnectBridge` |
| command contract | `softphone.bridge.FailBridge` |
| command contract | `softphone.control.Answer` |
| command contract | `softphone.control.AttachMedia` |
| command contract | `softphone.control.ConfigureEndpoint` |
| command contract | `softphone.control.ConfirmAnswer` |
| command contract | `softphone.control.Dial` |
| command contract | `softphone.control.FailCall` |
| command contract | `softphone.control.HangUp` |
| command contract | `softphone.control.MediaConnected` |
| command contract | `softphone.control.OfferCall` |
| command contract | `softphone.control.Reject` |
| command contract | `softphone.control.RingCall` |
| command contract | `softphone.control.SendDigits` |
| command contract | `softphone.control.SetHeld` |
| command contract | `softphone.control.SetMuted` |
| command contract | `softphone.directory.AddAddress` |
| command contract | `softphone.directory.AddContact` |
| command contract | `softphone.directory.DeleteContact` |
| command contract | `softphone.directory.RemoveAddress` |
| command contract | `softphone.directory.RenameContact` |
| command contract | `softphone.history.AttributeRecord` |
| command contract | `softphone.history.DeleteRecord` |
| command contract | `softphone.history.RecordCall` |
| command contract | `softphone.local.AttachLoopback` |
| command contract | `softphone.local.DetachLoopback` |
| command contract | `softphone.media.ActivateSession` |
| command contract | `softphone.media.InterruptOutput` |
| command contract | `softphone.media.OpenSession` |
| command contract | `softphone.media.ReceiveSignal` |
| command contract | `softphone.media.SendSignal` |
| command contract | `softphone.media.TerminateSession` |
| command contract | `softphone.presentation.AttributeTile` |
| command contract | `softphone.presentation.ClearEntry` |
| command contract | `softphone.presentation.DismissCall` |
| command contract | `softphone.presentation.EnterCall` |
| command contract | `softphone.presentation.EnterDialing` |
| command contract | `softphone.presentation.EnterIncoming` |
| command contract | `softphone.presentation.LeaveCall` |
| command contract | `softphone.presentation.OpenConsole` |
| command contract | `softphone.presentation.OpenKeypad` |
| command contract | `softphone.presentation.PressKey` |
| command contract | `softphone.presentation.ShowCall` |
| command contract | `softphone.sip.ApplyRemoteMedia` |
| command contract | `softphone.sip.CloseDialog` |
| command contract | `softphone.sip.ConfirmRegistration` |
| command contract | `softphone.sip.FailLocalMedia` |
| command contract | `softphone.sip.FailRegistration` |
| command contract | `softphone.sip.OfferLocalMedia` |
| command contract | `softphone.sip.OpenDialog` |
| command contract | `softphone.sip.RefreshRegistration` |
| command contract | `softphone.sip.Register` |
| command contract | `softphone.sip.RequestLocalMedia` |
| command contract | `softphone.sip.RetryRegistration` |
| command contract | `softphone.sip.Unregister` |
| event type | `softphone.bridge.BridgeClosed` |
| event type | `softphone.bridge.BridgeConfirmed` |
| event type | `softphone.bridge.BridgeConnecting` |
| event type | `softphone.bridge.BridgeFailed` |
| event type | `softphone.control.CallAnswered` |
| event type | `softphone.control.CallConfirmed` |
| event type | `softphone.control.CallDialled` |
| event type | `softphone.control.CallEstablished` |
| event type | `softphone.control.CallFailed` |
| event type | `softphone.control.CallHungUp` |
| event type | `softphone.control.CallOffered` |
| event type | `softphone.control.CallRejected` |
| event type | `softphone.control.CallRinging` |
| event type | `softphone.control.DigitsSent` |
| event type | `softphone.control.EndpointConfigured` |
| event type | `softphone.control.HoldChanged` |
| event type | `softphone.control.MediaAttached` |
| event type | `softphone.control.MuteChanged` |
| event type | `softphone.directory.ContactAdded` |
| event type | `softphone.directory.ContactAddressAdded` |
| event type | `softphone.directory.ContactAddressRemoved` |
| event type | `softphone.directory.ContactDeleted` |
| event type | `softphone.directory.ContactRenamed` |
| event type | `softphone.history.CallRecorded` |
| event type | `softphone.history.RecordAttributed` |
| event type | `softphone.history.RecordDeleted` |
| event type | `softphone.local.LoopbackAttached` |
| event type | `softphone.local.LoopbackDetached` |
| event type | `softphone.media.OutputInterrupted` |
| event type | `softphone.media.SessionActivated` |
| event type | `softphone.media.SessionOpened` |
| event type | `softphone.media.SessionTerminated` |
| event type | `softphone.media.SignalReceived` |
| event type | `softphone.media.SignalSent` |
| event type | `softphone.presentation.CallDismissed` |
| event type | `softphone.presentation.CallShown` |
| event type | `softphone.presentation.ConsoleDialing` |
| event type | `softphone.presentation.ConsoleEngaged` |
| event type | `softphone.presentation.ConsoleIncoming` |
| event type | `softphone.presentation.ConsoleOpened` |
| event type | `softphone.presentation.ConsoleReleased` |
| event type | `softphone.presentation.EntryCleared` |
| event type | `softphone.presentation.KeyPressed` |
| event type | `softphone.presentation.KeypadOpened` |
| event type | `softphone.presentation.TileAttributed` |
| event type | `softphone.sip.LocalMediaFailed` |
| event type | `softphone.sip.LocalMediaOffered` |
| event type | `softphone.sip.LocalMediaRequested` |
| event type | `softphone.sip.RegistrationConfirmed` |
| event type | `softphone.sip.RegistrationEnded` |
| event type | `softphone.sip.RegistrationFailed` |
| event type | `softphone.sip.RegistrationRefreshed` |
| event type | `softphone.sip.RegistrationRequested` |
| event type | `softphone.sip.RegistrationRetried` |
| event type | `softphone.sip.RemoteMediaApplied` |
| event type | `softphone.sip.SipDialogClosed` |
| event type | `softphone.sip.SipDialogOpened` |
| error type | `softphone.bridge.BridgeStateConflict` |
| error type | `softphone.control.CallStateConflict` |
| error type | `softphone.directory.ContactAddressStateConflict` |
| error type | `softphone.directory.ContactStateConflict` |
| error type | `softphone.history.CallRecordStateConflict` |
| error type | `softphone.local.LoopbackDeviceStateConflict` |
| error type | `softphone.media.MediaSessionStateConflict` |
| error type | `softphone.presentation.CallTileStateConflict` |
| error type | `softphone.presentation.ConsoleStateConflict` |
| error type | `softphone.sip.RegistrationStateConflict` |
| error type | `softphone.sip.SipDialogStateConflict` |
| view type | `softphone.bridge.BridgeById` |
| view type | `softphone.bridge.LiveBridges` |
| view type | `softphone.control.ActiveCalls` |
| view type | `softphone.control.CallById` |
| view type | `softphone.control.EndpointById` |
| view type | `softphone.directory.ContactAddresses` |
| view type | `softphone.directory.ContactById` |
| view type | `softphone.directory.Contacts` |
| view type | `softphone.history.CallRecordById` |
| view type | `softphone.history.RecentCalls` |
| view type | `softphone.local.LoopbackDeviceById` |
| view type | `softphone.media.LiveSessions` |
| view type | `softphone.media.MediaSessionById` |
| view type | `softphone.presentation.CallTiles` |
| view type | `softphone.presentation.ConsoleById` |
| view type | `softphone.presentation.KeypadById` |
| view type | `softphone.sip.RegistrationById` |
| view type | `softphone.sip.SipDialogById` |
| binding transformation | `activate-session-with-bridge` |
| binding delivery | `activate-session-with-bridge` |
| binding transformation | `end-session-with-bridge` |
| binding delivery | `end-session-with-bridge` |
| binding transformation | `end-session-with-dialog` |
| binding delivery | `end-session-with-dialog` |
| binding transformation | `end-session-with-failed-bridge` |
| binding delivery | `end-session-with-failed-bridge` |
| binding transformation | `end-session-with-loopback` |
| binding delivery | `end-session-with-loopback` |
| binding transformation | `record-failed-call` |
| binding delivery | `record-failed-call` |
| binding transformation | `record-hung-up-call` |
| binding delivery | `record-hung-up-call` |
| binding transformation | `record-rejected-call` |
| binding delivery | `record-rejected-call` |
| binding transformation | `show-incoming-call` |
| binding delivery | `show-incoming-call` |
| binding transformation | `show-outbound-call` |
| binding delivery | `show-outbound-call` |
| component port | `bridge-binding` |
| component port | `call-history` |
| component port | `local-binding` |
| component port | `media-session` |
| component port | `phone-console` |
| component port | `phone-control` |
| component port | `phone-directory` |
| component port | `sip-binding` |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| command behaviour | `softphone.bridge.CloseBridge` | the contract is declared; the algorithm is not | given `softphone.bridge.CloseBridge` input, decide and enact exactly one outcome — `closed` otherwise, takes `close` of `softphone.bridge.BridgeSession`, emits `softphone.bridge.BridgeClosed`; `wrong-state` from a state no declared move starts in, error `softphone.bridge.BridgeStateConflict` |
| command behaviour | `softphone.bridge.ConfirmBridge` | the contract is declared; the algorithm is not | given `softphone.bridge.ConfirmBridge` input, decide and enact exactly one outcome — `confirmed` otherwise, takes `establish` of `softphone.bridge.BridgeSession`, emits `softphone.bridge.BridgeConfirmed`; `wrong-state` from a state no declared move starts in, error `softphone.bridge.BridgeStateConflict` |
| command behaviour | `softphone.bridge.ConnectBridge` | the contract is declared; the algorithm is not | given `softphone.bridge.ConnectBridge` input, decide and enact exactly one outcome — `connecting` otherwise, creates `softphone.bridge.BridgeSession`, emits `softphone.bridge.BridgeConnecting` |
| command behaviour | `softphone.bridge.FailBridge` | the contract is declared; the algorithm is not | given `softphone.bridge.FailBridge` input, decide and enact exactly one outcome — `failed` otherwise, takes `close` of `softphone.bridge.BridgeSession`, emits `softphone.bridge.BridgeFailed`; `wrong-state` from a state no declared move starts in, error `softphone.bridge.BridgeStateConflict` |
| command behaviour | `softphone.control.Answer` | the contract is declared; the algorithm is not | given `softphone.control.Answer` input, decide and enact exactly one outcome — `answered` otherwise, takes `accept` of `softphone.control.Call`, emits `softphone.control.CallAnswered`; `wrong-state` from a state no declared move starts in, error `softphone.control.CallStateConflict` |
| command behaviour | `softphone.control.AttachMedia` | the contract is declared; the algorithm is not | given `softphone.control.AttachMedia` input, decide and enact exactly one outcome — `attached` otherwise, updates `softphone.control.Call`, emits `softphone.control.MediaAttached` |
| command behaviour | `softphone.control.ConfigureEndpoint` | the contract is declared; the algorithm is not | given `softphone.control.ConfigureEndpoint` input, decide and enact exactly one outcome — `configured` otherwise, creates `softphone.control.PhoneEndpoint`, emits `softphone.control.EndpointConfigured` |
| command behaviour | `softphone.control.ConfirmAnswer` | the contract is declared; the algorithm is not | given `softphone.control.ConfirmAnswer` input, decide and enact exactly one outcome — `confirmed` otherwise, takes `accept` of `softphone.control.Call`, emits `softphone.control.CallConfirmed`; `wrong-state` from a state no declared move starts in, error `softphone.control.CallStateConflict` |
| command behaviour | `softphone.control.Dial` | the contract is declared; the algorithm is not | given `softphone.control.Dial` input, decide and enact exactly one outcome — `dialled` otherwise, creates `softphone.control.Call`, emits `softphone.control.CallDialled` |
| command behaviour | `softphone.control.FailCall` | the contract is declared; the algorithm is not | given `softphone.control.FailCall` input, decide and enact exactly one outcome — `failed` otherwise, takes `end` of `softphone.control.Call`, emits `softphone.control.CallFailed`; `wrong-state` from a state no declared move starts in, error `softphone.control.CallStateConflict` |
| command behaviour | `softphone.control.HangUp` | the contract is declared; the algorithm is not | given `softphone.control.HangUp` input, decide and enact exactly one outcome — `hung-up` otherwise, takes `end` of `softphone.control.Call`, emits `softphone.control.CallHungUp`; `wrong-state` from a state no declared move starts in |
| command behaviour | `softphone.control.MediaConnected` | the contract is declared; the algorithm is not | given `softphone.control.MediaConnected` input, decide and enact exactly one outcome — `established` otherwise, takes `activate` of `softphone.control.Call`, emits `softphone.control.CallEstablished`; `wrong-state` from a state no declared move starts in, error `softphone.control.CallStateConflict` |
| command behaviour | `softphone.control.OfferCall` | the contract is declared; the algorithm is not | given `softphone.control.OfferCall` input, decide and enact exactly one outcome — `offered` otherwise, creates `softphone.control.Call`, emits `softphone.control.CallOffered` |
| command behaviour | `softphone.control.Reject` | the contract is declared; the algorithm is not | given `softphone.control.Reject` input, decide and enact exactly one outcome — `rejected` otherwise, takes `end` of `softphone.control.Call`, emits `softphone.control.CallRejected`; `wrong-state` from a state no declared move starts in, error `softphone.control.CallStateConflict` |
| command behaviour | `softphone.control.RingCall` | the contract is declared; the algorithm is not | given `softphone.control.RingCall` input, decide and enact exactly one outcome — `ringing` otherwise, takes `ring` of `softphone.control.Call`, emits `softphone.control.CallRinging`; `wrong-state` from a state no declared move starts in, error `softphone.control.CallStateConflict` |
| command behaviour | `softphone.control.SendDigits` | the contract is declared; the algorithm is not | given `softphone.control.SendDigits` input, decide and enact exactly one outcome — `sent` otherwise, emits `softphone.control.DigitsSent` |
| command behaviour | `softphone.control.SetHeld` | the contract is declared; the algorithm is not | given `softphone.control.SetHeld` input, decide and enact exactly one outcome — `changed` otherwise, updates `softphone.control.Call`, emits `softphone.control.HoldChanged` |
| command behaviour | `softphone.control.SetMuted` | the contract is declared; the algorithm is not | given `softphone.control.SetMuted` input, decide and enact exactly one outcome — `changed` otherwise, updates `softphone.control.Call`, emits `softphone.control.MuteChanged` |
| command behaviour | `softphone.directory.AddAddress` | the contract is declared; the algorithm is not | given `softphone.directory.AddAddress` input, decide and enact exactly one outcome — `added` otherwise, creates `softphone.directory.ContactAddress`, emits `softphone.directory.ContactAddressAdded` |
| command behaviour | `softphone.directory.AddContact` | the contract is declared; the algorithm is not | given `softphone.directory.AddContact` input, decide and enact exactly one outcome — `added` otherwise, creates `softphone.directory.Contact`, emits `softphone.directory.ContactAdded` |
| command behaviour | `softphone.directory.DeleteContact` | the contract is declared; the algorithm is not | given `softphone.directory.DeleteContact` input, decide and enact exactly one outcome — `deleted` otherwise, takes `delete` of `softphone.directory.Contact`, emits `softphone.directory.ContactDeleted`; `wrong-state` from a state no declared move starts in, error `softphone.directory.ContactStateConflict` |
| command behaviour | `softphone.directory.RemoveAddress` | the contract is declared; the algorithm is not | given `softphone.directory.RemoveAddress` input, decide and enact exactly one outcome — `removed` otherwise, takes `remove` of `softphone.directory.ContactAddress`, emits `softphone.directory.ContactAddressRemoved`; `wrong-state` from a state no declared move starts in, error `softphone.directory.ContactAddressStateConflict` |
| command behaviour | `softphone.directory.RenameContact` | the contract is declared; the algorithm is not | given `softphone.directory.RenameContact` input, decide and enact exactly one outcome — `renamed` otherwise, updates `softphone.directory.Contact`, emits `softphone.directory.ContactRenamed` |
| command behaviour | `softphone.history.AttributeRecord` | the contract is declared; the algorithm is not | given `softphone.history.AttributeRecord` input, decide and enact exactly one outcome — `attributed` otherwise, updates `softphone.history.CallRecord`, emits `softphone.history.RecordAttributed` |
| command behaviour | `softphone.history.DeleteRecord` | the contract is declared; the algorithm is not | given `softphone.history.DeleteRecord` input, decide and enact exactly one outcome — `deleted` otherwise, takes `delete` of `softphone.history.CallRecord`, emits `softphone.history.RecordDeleted`; `wrong-state` from a state no declared move starts in, error `softphone.history.CallRecordStateConflict` |
| command behaviour | `softphone.history.RecordCall` | the contract is declared; the algorithm is not | given `softphone.history.RecordCall` input, decide and enact exactly one outcome — `recorded` otherwise, creates `softphone.history.CallRecord`, emits `softphone.history.CallRecorded` |
| command behaviour | `softphone.local.AttachLoopback` | the contract is declared; the algorithm is not | given `softphone.local.AttachLoopback` input, decide and enact exactly one outcome — `attached` otherwise, creates `softphone.local.LoopbackDevice`, emits `softphone.local.LoopbackAttached` |
| command behaviour | `softphone.local.DetachLoopback` | the contract is declared; the algorithm is not | given `softphone.local.DetachLoopback` input, decide and enact exactly one outcome — `detached` otherwise, takes `detach` of `softphone.local.LoopbackDevice`, emits `softphone.local.LoopbackDetached`; `wrong-state` from a state no declared move starts in, error `softphone.local.LoopbackDeviceStateConflict` |
| command behaviour | `softphone.media.ActivateSession` | the contract is declared; the algorithm is not | given `softphone.media.ActivateSession` input, decide and enact exactly one outcome — `activated` otherwise, takes `establish` of `softphone.media.MediaSession`, emits `softphone.media.SessionActivated`; `wrong-state` from a state no declared move starts in, error `softphone.media.MediaSessionStateConflict` |
| command behaviour | `softphone.media.InterruptOutput` | the contract is declared; the algorithm is not | given `softphone.media.InterruptOutput` input, decide and enact exactly one outcome — `interrupted` otherwise, emits `softphone.media.OutputInterrupted` |
| command behaviour | `softphone.media.OpenSession` | the contract is declared; the algorithm is not | given `softphone.media.OpenSession` input, decide and enact exactly one outcome — `opened` otherwise, creates `softphone.media.MediaSession`, emits `softphone.media.SessionOpened` |
| command behaviour | `softphone.media.ReceiveSignal` | the contract is declared; the algorithm is not | given `softphone.media.ReceiveSignal` input, decide and enact exactly one outcome — `received` otherwise, emits `softphone.media.SignalReceived` |
| command behaviour | `softphone.media.SendSignal` | the contract is declared; the algorithm is not | given `softphone.media.SendSignal` input, decide and enact exactly one outcome — `sent` otherwise, emits `softphone.media.SignalSent` |
| command behaviour | `softphone.media.TerminateSession` | the contract is declared; the algorithm is not | given `softphone.media.TerminateSession` input, decide and enact exactly one outcome — `terminated` otherwise, takes `terminate` of `softphone.media.MediaSession`, emits `softphone.media.SessionTerminated`; `wrong-state` from a state no declared move starts in, error `softphone.media.MediaSessionStateConflict` |
| command behaviour | `softphone.presentation.AttributeTile` | the contract is declared; the algorithm is not | given `softphone.presentation.AttributeTile` input, decide and enact exactly one outcome — `attributed` otherwise, updates `softphone.presentation.CallTile`, emits `softphone.presentation.TileAttributed` |
| command behaviour | `softphone.presentation.ClearEntry` | the contract is declared; the algorithm is not | given `softphone.presentation.ClearEntry` input, decide and enact exactly one outcome — `cleared` otherwise, updates `softphone.presentation.Keypad`, emits `softphone.presentation.EntryCleared` |
| command behaviour | `softphone.presentation.DismissCall` | the contract is declared; the algorithm is not | given `softphone.presentation.DismissCall` input, decide and enact exactly one outcome — `dismissed` otherwise, takes `dismiss` of `softphone.presentation.CallTile`, emits `softphone.presentation.CallDismissed`; `wrong-state` from a state no declared move starts in, error `softphone.presentation.CallTileStateConflict` |
| command behaviour | `softphone.presentation.EnterCall` | the contract is declared; the algorithm is not | given `softphone.presentation.EnterCall` input, decide and enact exactly one outcome — `engaged` otherwise, takes `engage` of `softphone.presentation.Console`, emits `softphone.presentation.ConsoleEngaged`; `wrong-state` from a state no declared move starts in, error `softphone.presentation.ConsoleStateConflict` |
| command behaviour | `softphone.presentation.EnterDialing` | the contract is declared; the algorithm is not | given `softphone.presentation.EnterDialing` input, decide and enact exactly one outcome — `dialing` otherwise, takes `dial` of `softphone.presentation.Console`, emits `softphone.presentation.ConsoleDialing`; `wrong-state` from a state no declared move starts in, error `softphone.presentation.ConsoleStateConflict` |
| command behaviour | `softphone.presentation.EnterIncoming` | the contract is declared; the algorithm is not | given `softphone.presentation.EnterIncoming` input, decide and enact exactly one outcome — `incoming` otherwise, takes `offer` of `softphone.presentation.Console`, emits `softphone.presentation.ConsoleIncoming`; `wrong-state` from a state no declared move starts in, error `softphone.presentation.ConsoleStateConflict` |
| command behaviour | `softphone.presentation.LeaveCall` | the contract is declared; the algorithm is not | given `softphone.presentation.LeaveCall` input, decide and enact exactly one outcome — `released` otherwise, takes `release` of `softphone.presentation.Console`, emits `softphone.presentation.ConsoleReleased`; `wrong-state` from a state no declared move starts in, error `softphone.presentation.ConsoleStateConflict` |
| command behaviour | `softphone.presentation.OpenConsole` | the contract is declared; the algorithm is not | given `softphone.presentation.OpenConsole` input, decide and enact exactly one outcome — `opened` otherwise, creates `softphone.presentation.Console`, emits `softphone.presentation.ConsoleOpened` |
| command behaviour | `softphone.presentation.OpenKeypad` | the contract is declared; the algorithm is not | given `softphone.presentation.OpenKeypad` input, decide and enact exactly one outcome — `opened` otherwise, creates `softphone.presentation.Keypad`, emits `softphone.presentation.KeypadOpened` |
| command behaviour | `softphone.presentation.PressKey` | the contract is declared; the algorithm is not | given `softphone.presentation.PressKey` input, decide and enact exactly one outcome — `pressed` otherwise, updates `softphone.presentation.Keypad`, emits `softphone.presentation.KeyPressed` |
| command behaviour | `softphone.presentation.ShowCall` | the contract is declared; the algorithm is not | given `softphone.presentation.ShowCall` input, decide and enact exactly one outcome — `shown` otherwise, creates `softphone.presentation.CallTile`, emits `softphone.presentation.CallShown` |
| command behaviour | `softphone.sip.ApplyRemoteMedia` | the contract is declared; the algorithm is not | given `softphone.sip.ApplyRemoteMedia` input, decide and enact exactly one outcome — `applied` otherwise, updates `softphone.sip.SipDialog`, emits `softphone.sip.RemoteMediaApplied` |
| command behaviour | `softphone.sip.CloseDialog` | the contract is declared; the algorithm is not | given `softphone.sip.CloseDialog` input, decide and enact exactly one outcome — `closed` otherwise, takes `close` of `softphone.sip.SipDialog`, emits `softphone.sip.SipDialogClosed`; `wrong-state` from a state no declared move starts in, error `softphone.sip.SipDialogStateConflict` |
| command behaviour | `softphone.sip.ConfirmRegistration` | the contract is declared; the algorithm is not | given `softphone.sip.ConfirmRegistration` input, decide and enact exactly one outcome — `confirmed` otherwise, takes `confirm` of `softphone.sip.Registration`, emits `softphone.sip.RegistrationConfirmed`; `wrong-state` from a state no declared move starts in, error `softphone.sip.RegistrationStateConflict` |
| command behaviour | `softphone.sip.FailLocalMedia` | the contract is declared; the algorithm is not | given `softphone.sip.FailLocalMedia` input, decide and enact exactly one outcome — `failed` otherwise, emits `softphone.sip.LocalMediaFailed` |
| command behaviour | `softphone.sip.FailRegistration` | the contract is declared; the algorithm is not | given `softphone.sip.FailRegistration` input, decide and enact exactly one outcome — `failed` otherwise, takes `fail` of `softphone.sip.Registration`, emits `softphone.sip.RegistrationFailed`; `wrong-state` from a state no declared move starts in, error `softphone.sip.RegistrationStateConflict` |
| command behaviour | `softphone.sip.OfferLocalMedia` | the contract is declared; the algorithm is not | given `softphone.sip.OfferLocalMedia` input, decide and enact exactly one outcome — `offered` otherwise, updates `softphone.sip.SipDialog`, emits `softphone.sip.LocalMediaOffered` |
| command behaviour | `softphone.sip.OpenDialog` | the contract is declared; the algorithm is not | given `softphone.sip.OpenDialog` input, decide and enact exactly one outcome — `opened` otherwise, creates `softphone.sip.SipDialog`, emits `softphone.sip.SipDialogOpened` |
| command behaviour | `softphone.sip.RefreshRegistration` | the contract is declared; the algorithm is not | given `softphone.sip.RefreshRegistration` input, decide and enact exactly one outcome — `refreshed` otherwise, updates `softphone.sip.Registration`, emits `softphone.sip.RegistrationRefreshed` |
| command behaviour | `softphone.sip.Register` | the contract is declared; the algorithm is not | given `softphone.sip.Register` input, decide and enact exactly one outcome — `requested` otherwise, creates `softphone.sip.Registration`, emits `softphone.sip.RegistrationRequested` |
| command behaviour | `softphone.sip.RequestLocalMedia` | the contract is declared; the algorithm is not | given `softphone.sip.RequestLocalMedia` input, decide and enact exactly one outcome — `requested` otherwise, emits `softphone.sip.LocalMediaRequested` |
| command behaviour | `softphone.sip.RetryRegistration` | the contract is declared; the algorithm is not | given `softphone.sip.RetryRegistration` input, decide and enact exactly one outcome — `retried` otherwise, takes `retry` of `softphone.sip.Registration`, emits `softphone.sip.RegistrationRetried`; `wrong-state` from a state no declared move starts in, error `softphone.sip.RegistrationStateConflict` |
| command behaviour | `softphone.sip.Unregister` | the contract is declared; the algorithm is not | given `softphone.sip.Unregister` input, decide and enact exactly one outcome — `ended` otherwise, takes `end` of `softphone.sip.Registration`, emits `softphone.sip.RegistrationEnded`; `wrong-state` from a state no declared move starts in, error `softphone.sip.RegistrationStateConflict` |
| view query | `softphone.bridge.BridgeById` | how the projection is kept current is a storage decision | a query answering `softphone.bridge.BridgeById` with rows projected from `softphone.bridge.BridgeSession` at `read_your_writes` consistency, containing instances where `bridge_id == param.bridge_id` |
| view query | `softphone.bridge.LiveBridges` | how the projection is kept current is a storage decision | a query answering `softphone.bridge.LiveBridges` with rows projected from `softphone.bridge.BridgeSession` at `read_your_writes` consistency, containing instances where `state == Live` |
| view query | `softphone.control.ActiveCalls` | how the projection is kept current is a storage decision | a query answering `softphone.control.ActiveCalls` with rows projected from `softphone.control.Call` at `read_your_writes` consistency, containing instances where `state == Active` |
| view query | `softphone.control.CallById` | how the projection is kept current is a storage decision | a query answering `softphone.control.CallById` with rows projected from `softphone.control.Call` at `read_your_writes` consistency, containing instances where `call_id == param.call_id` |
| view query | `softphone.control.EndpointById` | how the projection is kept current is a storage decision | a query answering `softphone.control.EndpointById` with rows projected from `softphone.control.PhoneEndpoint` at `read_your_writes` consistency, containing instances where `endpoint_id == param.endpoint_id` |
| view query | `softphone.directory.ContactAddresses` | how the projection is kept current is a storage decision | a query answering `softphone.directory.ContactAddresses` with rows projected from `softphone.directory.ContactAddress` at `read_your_writes` consistency, containing instances where `(contact_id == param.contact_id and state == Active)` |
| view query | `softphone.directory.ContactById` | how the projection is kept current is a storage decision | a query answering `softphone.directory.ContactById` with rows projected from `softphone.directory.Contact` at `read_your_writes` consistency, containing instances where `contact_id == param.contact_id` |
| view query | `softphone.directory.Contacts` | how the projection is kept current is a storage decision | a query answering `softphone.directory.Contacts` with rows projected from `softphone.directory.Contact` at `read_your_writes` consistency, containing instances where `state == Active` |
| view query | `softphone.history.CallRecordById` | how the projection is kept current is a storage decision | a query answering `softphone.history.CallRecordById` with rows projected from `softphone.history.CallRecord` at `read_your_writes` consistency, containing instances where `record_id == param.record_id` |
| view query | `softphone.history.RecentCalls` | how the projection is kept current is a storage decision | a query answering `softphone.history.RecentCalls` with rows projected from `softphone.history.CallRecord` at `read_your_writes` consistency, containing instances where `state == Recorded` |
| view query | `softphone.local.LoopbackDeviceById` | how the projection is kept current is a storage decision | a query answering `softphone.local.LoopbackDeviceById` with rows projected from `softphone.local.LoopbackDevice` at `read_your_writes` consistency, containing instances where `device_id == param.device_id` |
| view query | `softphone.media.LiveSessions` | how the projection is kept current is a storage decision | a query answering `softphone.media.LiveSessions` with rows projected from `softphone.media.MediaSession` at `read_your_writes` consistency, containing instances where `state == Live` |
| view query | `softphone.media.MediaSessionById` | how the projection is kept current is a storage decision | a query answering `softphone.media.MediaSessionById` with rows projected from `softphone.media.MediaSession` at `read_your_writes` consistency, containing instances where `session_id == param.session_id` |
| view query | `softphone.presentation.CallTiles` | how the projection is kept current is a storage decision | a query answering `softphone.presentation.CallTiles` with rows projected from `softphone.presentation.CallTile` at `read_your_writes` consistency, containing instances where `state == Shown` |
| view query | `softphone.presentation.ConsoleById` | how the projection is kept current is a storage decision | a query answering `softphone.presentation.ConsoleById` with rows projected from `softphone.presentation.Console` at `read_your_writes` consistency, containing instances where `console_id == param.console_id` |
| view query | `softphone.presentation.KeypadById` | how the projection is kept current is a storage decision | a query answering `softphone.presentation.KeypadById` with rows projected from `softphone.presentation.Keypad` at `read_your_writes` consistency, containing instances where `keypad_id == param.keypad_id` |
| view query | `softphone.sip.RegistrationById` | how the projection is kept current is a storage decision | a query answering `softphone.sip.RegistrationById` with rows projected from `softphone.sip.Registration` at `read_your_writes` consistency, containing instances where `registration_id == param.registration_id` |
| view query | `softphone.sip.SipDialogById` | how the projection is kept current is a storage decision | a query answering `softphone.sip.SipDialogById` with rows projected from `softphone.sip.SipDialog` at `read_your_writes` consistency, containing instances where `dialog_id == param.dialog_id` |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
| actor grants | `softphone.bridge.BridgeHost` | planning | may invoke `softphone.bridge.CloseBridge`, `softphone.bridge.ConnectBridge`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.bridge.Kernel` | planning | may invoke `softphone.bridge.ConfirmBridge`, `softphone.bridge.FailBridge`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.control.Agent` | planning | may invoke `softphone.control.Answer`, `softphone.control.ConfigureEndpoint`, `softphone.control.Dial`, `softphone.control.HangUp`, `softphone.control.Reject`, `softphone.control.SendDigits`, `softphone.control.SetHeld`, `softphone.control.SetMuted`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.control.Human` | planning | may invoke `softphone.control.Answer`, `softphone.control.ConfigureEndpoint`, `softphone.control.Dial`, `softphone.control.HangUp`, `softphone.control.Reject`, `softphone.control.SendDigits`, `softphone.control.SetHeld`, `softphone.control.SetMuted`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.control.Kernel` | planning | may invoke `softphone.control.AttachMedia`, `softphone.control.ConfirmAnswer`, `softphone.control.FailCall`, `softphone.control.MediaConnected`, `softphone.control.OfferCall`, `softphone.control.RingCall`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.directory.Human` | planning | may invoke `softphone.directory.AddAddress`, `softphone.directory.AddContact`, `softphone.directory.DeleteContact`, `softphone.directory.RemoveAddress`, `softphone.directory.RenameContact`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.history.HistoryHost` | planning | may invoke `softphone.history.RecordCall`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.history.Human` | planning | may invoke `softphone.history.AttributeRecord`, `softphone.history.DeleteRecord`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.local.LocalHost` | planning | may invoke `softphone.local.AttachLoopback`, `softphone.local.DetachLoopback`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.media.SessionHost` | planning | may invoke `softphone.media.ActivateSession`, `softphone.media.InterruptOutput`, `softphone.media.OpenSession`, `softphone.media.ReceiveSignal`, `softphone.media.SendSignal`, `softphone.media.TerminateSession`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.presentation.Human` | planning | may invoke `softphone.presentation.AttributeTile`, `softphone.presentation.ClearEntry`, `softphone.presentation.DismissCall`, `softphone.presentation.EnterCall`, `softphone.presentation.EnterDialing`, `softphone.presentation.EnterIncoming`, `softphone.presentation.LeaveCall`, `softphone.presentation.OpenConsole`, `softphone.presentation.OpenKeypad`, `softphone.presentation.PressKey`, `softphone.presentation.ShowCall`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.sip.Kernel` | planning | may invoke `softphone.sip.ApplyRemoteMedia`, `softphone.sip.CloseDialog`, `softphone.sip.ConfirmRegistration`, `softphone.sip.FailRegistration`, `softphone.sip.RequestLocalMedia`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| actor grants | `softphone.sip.SipHost` | planning | may invoke `softphone.sip.CloseDialog`, `softphone.sip.FailLocalMedia`, `softphone.sip.OfferLocalMedia`, `softphone.sip.OpenDialog`, `softphone.sip.RefreshRegistration`, `softphone.sip.Register`, `softphone.sip.RetryRegistration`, `softphone.sip.Unregister`; a grant is checked against a caller identity, which types do not carry, and enforcement belongs to the layer that knows who is calling |
| workload | `bridge-binding` | planning | requires at least 1 replica(s); topology synthesis is deferred with its design |
| workload | `call-history` | planning | requires at least 1 replica(s); topology synthesis is deferred with its design |
| workload | `local-binding` | planning | requires at least 1 replica(s); topology synthesis is deferred with its design |
| workload | `media-session` | planning | requires at least 1 replica(s); topology synthesis is deferred with its design |
| workload | `phone-console` | planning | requires at least 1 replica(s); topology synthesis is deferred with its design |
| workload | `phone-control` | planning | requires at least 1 replica(s); topology synthesis is deferred with its design |
| workload | `phone-directory` | planning | requires at least 1 replica(s); topology synthesis is deferred with its design |
| workload | `sip-binding` | planning | requires at least 1 replica(s); topology synthesis is deferred with its design |
