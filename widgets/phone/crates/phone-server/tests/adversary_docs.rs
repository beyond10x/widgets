//! The crate's own prose, held to the compiled specification the same way `tests/wire.rs` holds
//! the code to it.
//!
//! `src/lib.rs`'s crate doc states a rule and then enumerates the commands it covers: "each fact
//! leaves as one command the specification already grants to a kernel actor — …". That sentence is
//! a contract with the next author, it is what `cargo doc` publishes, and nothing in the suite
//! compares it against the model. This file does.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use phone_server::wire::{BridgeCause, EndCause, TerminationReason, ToBrowser};
use serde_json::Value;

/// The crate's own source, read as text: the prose is the subject here, not the behaviour.
fn source(name: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The compiled specification, from the installed `ess` — this repository's authority on the model.
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

/// Every command in every `*.Kernel` actor's `may:` list.
fn kernel_grants(model: &Value) -> BTreeSet<String> {
    let mut granted = BTreeSet::new();
    for (name, actor) in model["actors"]
        .as_object()
        .expect("the model declares actors")
    {
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
    assert!(!granted.is_empty(), "the model has kernel actors");
    granted
}

fn is_command_leaf(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_uppercase())
        && token.chars().all(|letter| letter.is_ascii_alphanumeric())
}

/// The command names one doc comment enumerates.
///
/// The enumeration abbreviates: after `softphone.control.RingCall` it writes `ConfirmAnswer` and
/// `HangUp` bare, and a reader carries the domain across. So does this — within one line, which is
/// as far as the abbreviation ever reaches and keeps a capitalised word elsewhere in the prose
/// (`RTCPeerConnection`) out of the set.
fn commands_named_in(doc: &str) -> BTreeSet<String> {
    let mut named = BTreeSet::new();
    for line in doc
        .lines()
        .filter(|line| line.trim_start().starts_with("//!"))
    {
        let mut domain: Option<String> = None;
        for (index, span) in line.split('`').enumerate() {
            if index % 2 == 0 {
                continue;
            }
            let token = span.trim();
            if let Some(rest) = token.strip_prefix("softphone.") {
                let mut parts = rest.split('.');
                let (Some(found), Some(leaf), None) = (parts.next(), parts.next(), parts.next())
                else {
                    continue;
                };
                if !found.chars().all(|letter| letter.is_ascii_lowercase())
                    || !is_command_leaf(leaf)
                {
                    continue;
                }
                named.insert(format!("softphone.{found}.{leaf}"));
                domain = Some(found.to_owned());
            } else if is_command_leaf(token) {
                if let Some(found) = domain.as_deref() {
                    named.insert(format!("softphone.{found}.{token}"));
                }
            }
        }
    }
    named
}

/// `src/lib.rs:11-14` says every fact leaves as one command "the specification already grants to a
/// kernel actor", and then lists them. Every name in that list has to be in a kernel's `may:`.
#[test]
fn the_crate_doc_names_only_commands_a_kernel_actor_may_issue() {
    let named = commands_named_in(&source("lib.rs"));
    // A guard on the reader rather than on the crate: if the prose is rewrapped so that nothing
    // is found, this case must fail loudly instead of passing on an empty set.
    for expected in [
        "softphone.bridge.ConfirmBridge",
        "softphone.bridge.FailBridge",
        "softphone.control.RingCall",
    ] {
        assert!(
            named.contains(expected),
            "the crate doc's enumeration was not read: `{expected}` is missing from {named:?}"
        );
    }

    let granted = kernel_grants(&model());
    let ungranted: Vec<&String> = named.difference(&granted).collect();
    assert!(
        ungranted.is_empty(),
        "the crate doc says every fact leaves as a command a kernel actor may issue, and names \
         {ungranted:?}, which the specification grants to no kernel actor"
    );
}

/// The same enumeration, against the code rather than the model: the doc must name what
/// [`ToBrowser`] can actually produce, and nothing else.
#[test]
fn the_crate_doc_names_exactly_what_the_server_can_send() {
    let sendable: BTreeSet<String> = [
        ToBrowser::ConfirmBridge {
            bridge_id: String::new(),
            session_id: String::new(),
            answer: String::new(),
        },
        ToBrowser::RingCall {
            call_id: String::new(),
        },
        ToBrowser::ConfirmAnswer {
            call_id: String::new(),
        },
        ToBrowser::FailCall {
            call_id: String::new(),
            cause: EndCause::Remote,
        },
        ToBrowser::FailBridge {
            bridge_id: String::new(),
            session_id: String::new(),
            cause: BridgeCause::Refused,
            reason: TerminationReason::ProtocolError,
        },
    ]
    .iter()
    .map(|message| message.command().to_owned())
    .collect();

    assert_eq!(
        commands_named_in(&source("lib.rs")),
        sendable,
        "the crate doc and `ToBrowser` disagree about what this server sends"
    );
}

/// The `ToBrowser` variants `src/wire.rs` declares, and the command each `command()` arm returns.
///
/// Read from the declaration rather than from a list of samples, which is the hole
/// `tests/wire.rs` leaves open: `every_message` builds its `named` set *from* `samples`, so a
/// variant that gains a `match` arm and no sample is counted in neither and its command is never
/// checked against the model.
fn declared_variants(source: &str) -> BTreeMap<String, String> {
    let mut variants: BTreeSet<String> = BTreeSet::new();
    let mut in_enum = false;
    for line in source.lines() {
        if line.starts_with("pub enum ToBrowser {") {
            in_enum = true;
            continue;
        }
        if in_enum {
            if line == "}" {
                in_enum = false;
                continue;
            }
            if let Some(name) = line.strip_prefix("    ").and_then(|rest| {
                rest.split_once(" {")
                    .map(|(name, _)| name)
                    .filter(|name| is_command_leaf(name))
            }) {
                assert!(
                    variants.insert(name.to_owned()),
                    "`{name}` is declared twice"
                );
            }
        }
    }
    assert!(
        !variants.is_empty(),
        "`pub enum ToBrowser` was not found in src/wire.rs"
    );

    let mut arms = BTreeMap::new();
    for line in source.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("Self::") else {
            continue;
        };
        let Some((variant, tail)) = rest.split_once(" { .. } => \"") else {
            continue;
        };
        let Some((command, _)) = tail.split_once('"') else {
            continue;
        };
        if variants.contains(variant) {
            arms.insert(variant.to_owned(), command.to_owned());
        }
    }

    let unmapped: Vec<&String> = variants
        .iter()
        .filter(|variant| !arms.contains_key(*variant))
        .collect();
    assert!(
        unmapped.is_empty(),
        "{unmapped:?} is declared on `ToBrowser` and `command()` has no arm naming it"
    );
    arms
}

/// Closes the hole named above: every declared variant's command has to be a kernel grant,
/// whether or not anybody sampled it.
#[test]
fn every_declared_to_browser_variant_is_a_command_a_kernel_may_issue() {
    let arms = declared_variants(&source("wire.rs"));
    let granted = kernel_grants(&model());
    let ungranted: Vec<(&String, &String)> = arms
        .iter()
        .filter(|(_, command)| !granted.contains(*command))
        .collect();
    assert!(
        ungranted.is_empty(),
        "{ungranted:?}: `ToBrowser` declares a variant whose command the specification grants to \
         no kernel actor, so no page will run it"
    );
}
