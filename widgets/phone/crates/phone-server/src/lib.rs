//! The one process in this application that holds a SIP stack.
//!
//! A browser owns `RTCPeerConnection`, ICE, DTLS-SRTP, capture and render, and it owns no SIP
//! stack. So the page negotiates one audio leg against this server, and this server places the
//! SIP call and forwards audio between the two.
//! `architecture-decision-record:browser-holds-no-sip-stack` is the decision; this crate is its
//! server half.
//!
//! **It runs no ESS system and holds no phone state.** The browser holds the phone's state, and
//! this process is a device: it reports facts about legs, and each fact leaves as one command the
//! specification already grants to a kernel actor — `softphone.bridge.ConfirmBridge`,
//! `softphone.control.RingCall`, `ConfirmAnswer`, `HangUp`, `FailCall`,
//! `softphone.bridge.FailBridge`. There is one authority over a call's state and no event log to
//! reconcile across a process boundary. [`wire`] is that vocabulary and nothing else.
//!
//! Three legs, each of them sipx's:
//!
//! | leg | what it is |
//! | --- | --- |
//! | [`browser`] | a `sipx_media` session with no SIP in it at all, on the profile `docs/specs/webrtc-audio.md` §1 names |
//! | [`sip`] | an ordinary `sipx_call` outbound call, G.711 |
//! | [`bridge`] | `sipx_media::Bridge`, forwarding between the two |
//!
//! Both legs are sipx media sessions, which is why no PCM is ever injected into a live call here:
//! the bridge moves payloads between two sessions over their own channels.

pub mod bridge;
pub mod browser;
pub mod session;
pub mod sip;
pub mod wire;
