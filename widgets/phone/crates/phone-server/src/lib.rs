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
//! `softphone.control.RingCall`, `ConfirmAnswer`, `FailCall`, `softphone.bridge.FailBridge`.
//! There is one authority over a call's state and no event log to reconcile across a process
//! boundary. [`wire`] is that vocabulary and nothing else.
//!
//! Hanging up is deliberately not in that list. The specification grants that command to the human
//! and agent actors and to no kernel, because hanging up is a local decision and this process makes
//! none; a far leg going away is a failed call with a cause. [`wire`] carries the argument.
//!
//! **That enumeration is under test, not merely written.** `tests/adversary_docs.rs` reads this doc
//! comment as text, extracts the command names out of it, and fails if any is one the compiled
//! specification grants to no kernel actor, or if the set differs from what [`wire::ToBrowser`] can
//! produce. So the prose `cargo doc` publishes is held to the model the same way the code is —
//! which is how this paragraph came to be worded without naming the command it is about, since
//! naming it is exactly what the case forbids.
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
//!
//! **One clock on both legs, or no bridge.** The forwarding decodes one leg and re-encodes for the
//! other without converting the sample rate, so a browser leg at 48 kHz against a G.711 leg at
//! 8 kHz would deliver every half second of speech as nearly three — the far end hears the caller
//! sped up. [`browser::bridgeable`] refuses that pair rather than carrying it, and the refusal
//! names the way through: a page whose offer lists PCMU ahead of Opus gets a leg this server can
//! bridge, with Opus still present in the vocabulary as the profile requires. Selecting it is the
//! page's to do, because offer/answer makes the answer's codec order the offer's. Opus with real
//! rate conversion is separate work and is deliberately not done here.

pub mod bridge;
pub mod browser;
pub mod session;
pub mod sip;
pub mod wire;
