//! The control channel carries the specification's vocabulary and nothing else.
//!
//! Held against the **compiled specification**, not against a list in this file. `AGENTS.md` makes
//! the installed `ess` binary the authority on the model, so that is what this asks; a command
//! name, an input field or a kernel grant that only exists in a comment fails here.
//!
//! Three claims, and the third is the one that closes the class:
//!
//! 1. every command this server sends is declared by the model;
//! 2. its input carries exactly the fields the model declares for it;
//! 3. every command the model grants to a **kernel** actor is classified — either this server
//!    sends it, or there is a written reason it does not. A specification that grows a kernel
//!    command fails this test until somebody decides which side of that line it falls on, which is
//!    what stops the enumeration needing an adversary to extend it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::Command;

use phone_server::wire::{
    BridgeCause, EndCause, FromBrowser, PresenceCause, TerminationReason, ToBrowser,
};
use serde_json::{json, Value};

/// The commands this server sends the page.
const SENT: &[&str] = &[
    "softphone.bridge.ConfirmBridge",
    "softphone.bridge.FailBridge",
    "softphone.control.ConfirmAnswer",
    "softphone.control.FailCall",
    "softphone.control.RingCall",
    "softphone.presence.ConfirmPresence",
    "softphone.presence.FailPresence",
    "softphone.presence.NoteGone",
    "softphone.presence.NotePresent",
];

/// Every other command the model grants a kernel, and why this server is not the one issuing it.
const NOT_SENT: &[(&str, &str)] = &[
    (
        "softphone.control.OfferCall",
        "inbound calls are not this arrangement: nothing here accepts an INVITE",
    ),
    (
        "softphone.control.MediaConnected",
        "only the page watches its own RTCPeerConnection, so only the page can say media is up",
    ),
    (
        "softphone.control.AttachMedia",
        "the page created the media session before it said anything here, so it attaches its own",
    ),
    (
        "softphone.sip.ConfirmRegistration",
        "registration is story:signalling-over-wss, and this story places one call",
    ),
    (
        "softphone.sip.FailRegistration",
        "registration is story:signalling-over-wss, and this story places one call",
    ),
    (
        "softphone.sip.CloseDialog",
        "the page holds no SIP dialog on this arrangement; softphone.bridge is its carrier",
    ),
    (
        "softphone.sip.RequestLocalMedia",
        "the page holds no SIP dialog on this arrangement; softphone.bridge is its carrier",
    ),
    (
        "softphone.sip.ApplyRemoteMedia",
        "the page holds no SIP dialog on this arrangement; softphone.bridge is its carrier",
    ),
];

/// The compiled specification, from the installed `ess`.
fn model() -> Value {
    let spec = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ess");
    let output = Command::new("ess")
        .args(["compile", "--path"])
        .arg(&spec)
        .args(["--format", "json"])
        .output()
        .expect("`ess` on PATH — it is this repository's authority on the specification");
    assert!(
        output.status.success(),
        "ess compile failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("`ess compile --format json` answers JSON")
}

/// One sample of every `ToBrowser` variant.
///
/// The `match` below has no wildcard arm, so a variant added to `ToBrowser` stops this file
/// compiling until it is named here — and its command then has to be in [`SENT`], which has to
/// partition the model's kernel grants.
fn every_message() -> Vec<ToBrowser> {
    let samples = vec![
        ToBrowser::ConfirmBridge {
            bridge_id: "b".to_owned(),
            session_id: "s".to_owned(),
            answer: "v=0\r\n".to_owned(),
        },
        ToBrowser::RingCall {
            call_id: "c".to_owned(),
        },
        ToBrowser::ConfirmAnswer {
            call_id: "c".to_owned(),
        },
        ToBrowser::FailCall {
            call_id: "c".to_owned(),
            cause: EndCause::Remote,
        },
        ToBrowser::FailBridge {
            bridge_id: "b".to_owned(),
            session_id: "s".to_owned(),
            cause: BridgeCause::Transport,
            reason: TerminationReason::TransportLost,
        },
        ToBrowser::ConfirmPresence {
            presence_id: "p".to_owned(),
        },
        ToBrowser::FailPresence {
            presence_id: "p".to_owned(),
            cause: PresenceCause::Refused,
        },
        ToBrowser::NotePresent {
            handle: "grace@phone.dev.test".to_owned(),
            label: "Grace".to_owned(),
        },
        ToBrowser::NoteGone {
            handle: "grace@phone.dev.test".to_owned(),
        },
    ];
    let named: BTreeSet<&str> = samples
        .iter()
        .map(|sample| match sample {
            ToBrowser::ConfirmBridge { .. } => "ConfirmBridge",
            ToBrowser::RingCall { .. } => "RingCall",
            ToBrowser::ConfirmAnswer { .. } => "ConfirmAnswer",
            ToBrowser::FailCall { .. } => "FailCall",
            ToBrowser::FailBridge { .. } => "FailBridge",
            ToBrowser::ConfirmPresence { .. } => "ConfirmPresence",
            ToBrowser::FailPresence { .. } => "FailPresence",
            ToBrowser::NotePresent { .. } => "NotePresent",
            ToBrowser::NoteGone { .. } => "NoteGone",
        })
        .collect();
    assert_eq!(
        named.len(),
        samples.len(),
        "one sample per variant, and no variant sampled twice"
    );
    samples
}

/// Every command in every `*.Kernel` actor's `may:` list, as the model declares it.
fn kernel_grants(model: &Value) -> BTreeSet<String> {
    let actors = model["actors"]
        .as_object()
        .expect("the model declares actors");
    let mut granted = BTreeSet::new();
    for (name, actor) in actors {
        if !name.ends_with(".Kernel") {
            continue;
        }
        for command in actor["may"]
            .as_array()
            .expect("an actor's `may:` is a list")
        {
            granted.insert(
                command
                    .as_str()
                    .expect("a granted command is a name")
                    .to_owned(),
            );
        }
    }
    assert!(
        !granted.is_empty(),
        "the model has kernel actors; if this is empty the reader is wrong, not the model"
    );
    granted
}

#[test]
fn every_command_the_server_sends_is_one_the_model_declares() {
    let model = model();
    let commands = model["commands"]
        .as_object()
        .expect("the model declares commands");

    for message in every_message() {
        let name = message.command();
        let declared = commands.get(name).unwrap_or_else(|| {
            panic!("`{name}` is not a command this specification declares, so no page runs it")
        });

        let expected: BTreeSet<&str> = declared["input"]
            .as_array()
            .expect("a command's input is a list")
            .iter()
            .map(|field| field["name"].as_str().expect("an input field has a name"))
            .collect();
        let request = message.request();
        let sent: BTreeSet<&str> = request["input"]
            .as_object()
            .expect("the request carries an input object")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            sent, expected,
            "`{name}` is sent with fields the specification does not declare, or without ones it does"
        );
    }
}

#[test]
fn the_envelope_is_the_one_the_pages_own_bridge_dispatches() {
    // `{"request":"command","command":…,"input":{…}}` — the three words
    // `.build/synth/web/crates/softphone-web/src/lib.rs`'s `fn answer` reads. A fourth spelling
    // would be a protocol of this crate's invention.
    let message = ToBrowser::FailBridge {
        bridge_id: "b7".to_owned(),
        session_id: "s7".to_owned(),
        cause: BridgeCause::Refused,
        reason: TerminationReason::ProtocolError,
    };
    assert_eq!(
        message.request(),
        json!({
            "request": "command",
            "command": "softphone.bridge.FailBridge",
            "input": {
                "bridge_id": "b7",
                "session_id": "s7",
                "cause": "Refused",
                "reason": "ProtocolError",
            },
        }),
        "an enum value goes on the wire as the variant's own name, which is what the generated \
         decoder reads"
    );
}

#[test]
fn every_kernel_command_is_either_sent_or_has_a_reason_it_is_not() {
    let model = model();
    let granted = kernel_grants(&model);

    let mut classified: BTreeMap<&str, &str> = BTreeMap::new();
    for name in SENT {
        assert!(
            classified.insert(name, "sent").is_none(),
            "`{name}` is classified twice"
        );
    }
    for (name, reason) in NOT_SENT {
        assert!(
            classified.insert(name, reason).is_none(),
            "`{name}` is classified twice"
        );
    }

    let classified_names: BTreeSet<String> =
        classified.keys().map(|name| (*name).to_owned()).collect();
    let unclassified: Vec<&String> = granted.difference(&classified_names).collect();
    assert!(
        unclassified.is_empty(),
        "the specification grants a kernel actor {unclassified:?}, and nothing in this crate says \
         whether phone-server issues it. Add it to SENT with a variant, or to NOT_SENT with the \
         reason it is somebody else's."
    );
    let invented: Vec<&String> = classified_names.difference(&granted).collect();
    assert!(
        invented.is_empty(),
        "{invented:?} is classified here but granted to no kernel actor in the specification"
    );

    let sent_by_messages: BTreeSet<String> = every_message()
        .iter()
        .map(|message| message.command().to_owned())
        .collect();
    let expected: BTreeSet<String> = SENT.iter().map(|name| (*name).to_owned()).collect();
    assert_eq!(
        sent_by_messages, expected,
        "`ToBrowser` and SENT disagree about what this server sends"
    );
}

#[test]
fn the_page_speaks_one_offer_one_destination_and_nothing_else() {
    let opened: FromBrowser = serde_json::from_str(
        r#"{"leg":"open-bridge","bridge_id":"b1","session_id":"s1","call_id":"c1",
            "destination":"sip:1000@asterisk.internal","offer":"v=0\r\n"}"#,
    )
    .expect("the page's own message");
    assert_eq!(
        opened,
        FromBrowser::OpenBridge {
            bridge_id: "b1".to_owned(),
            session_id: "s1".to_owned(),
            call_id: "c1".to_owned(),
            destination: "sip:1000@asterisk.internal".to_owned(),
            offer: "v=0\r\n".to_owned(),
        }
    );

    // Non-trickle is enforced by there being nothing to send: a candidate arriving on its own is
    // refused rather than ignored, because ignoring it would look like accepting it.
    let trickled = serde_json::from_str::<FromBrowser>(
        r#"{"leg":"candidate","bridge_id":"b1","candidate":"candidate:1 1 UDP 1 10.0.0.1 4 typ host"}"#,
    );
    assert!(
        trickled.is_err(),
        "one offer and one answer: there is no message that adds a candidate afterwards"
    );
}

/// `EndCause::Refused` reaches the page from the far leg and from nowhere else.
///
/// `wire::EndCause` documents `Refused` as "the far end refused the attempt", so a bridge *this
/// server* declined must not borrow the word — a page cannot otherwise tell a PBX rejecting a call
/// from this server declining a codec pair the far end never saw. Enumerated over every
/// `OpenFailed` variant rather than over the one the adversary reported, because the class is "a
/// failure this server originates" and there are five of them.
#[test]
fn a_refusal_of_ours_is_never_reported_as_the_far_ends() {
    use phone_server::browser::{BridgeRefused, Refusal};
    use phone_server::session::OpenFailed;
    use phone_server::wire::{BridgeCause, TerminationReason};

    let ours = [
        OpenFailed::Browser(BridgeRefused {
            cause: BridgeCause::Refused,
            reason: TerminationReason::ProtocolError,
            source: Refusal::NotSdp,
        }),
        OpenFailed::Browser(BridgeRefused {
            cause: BridgeCause::Refused,
            reason: TerminationReason::ProtocolError,
            source: Refusal::NoRateConversion {
                browser: "Opus",
                browser_rate: 48_000,
                sip_rate: 8_000,
            },
        }),
        OpenFailed::NoMediaPort(std::io::Error::other("no port")),
        OpenFailed::NoIdentity,
        OpenFailed::NotDialable("not a uri".to_owned()),
    ];

    for failure in &ours {
        let ToBrowser::FailCall { cause, .. } = failure.fail_call(&"c1".to_owned()) else {
            panic!("a failed open tells the page about the call");
        };
        assert_ne!(
            cause,
            EndCause::Refused,
            "`{failure}` is this server's own refusal and it reaches the page as the far end's"
        );
    }

    // And the one variant that may carry it, does: a far leg that really was refused is the only
    // source of the word, which is what makes it worth anything to a page.
    let far = OpenFailed::FarLeg(phone_server::sip::LegFailed {
        cause: EndCause::Refused,
        source: sipx_call::Error::Rejected {
            status: 486,
            reason: "Busy Here".to_owned(),
        },
    });
    let ToBrowser::FailCall { cause, .. } = far.fail_call(&"c1".to_owned()) else {
        panic!("a failed open tells the page about the call");
    };
    assert_eq!(
        cause,
        EndCause::Refused,
        "a 486 from the far end is the far end refusing the attempt"
    );
}
