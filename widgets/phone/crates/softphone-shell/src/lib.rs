//! The module a page loads.
//!
//! The generated web bridge answers JSON over three WebAssembly exports and drives whichever system
//! a host installs. It ships with none — every obligation is unfilled there, by design — so this
//! crate is the host: it links `softphone-behaviour`, assembles the eight generated ports over one
//! store, and installs the result through the bridge's own `install` seam.
//!
//! `ess_realize` is the hook the generated `bridge.js` calls once, before any request, when a
//! module exports it. So the page, the wire and the catalogue stay exactly as emitted, and the only
//! difference between this module and the unrealized one is that this one answers.

#![deny(missing_docs)]

/// Installs this application's behaviour as the system the bridge serves.
///
/// Called once by `bridge.js` immediately after instantiation. Calling it again replaces the
/// system, and with it the store — which is a reset, not a merge.
#[no_mangle]
pub extern "C" fn ess_realize() {
    softphone_web::install(Box::new(softphone_behaviour::system()));
}
