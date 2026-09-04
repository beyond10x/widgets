// The harness: this widget, on a page of its own, with no devcenter and no build of anything else.
//
// It exists so the browser half can be tried before it is packaged — `task widget` serves it — and
// so a failure has one fewer place to be. The module's bytes come from a URL because that is what a
// served page has; a bundler hands the same component an asset instead.

import { createApp, h } from "vue";
import PhonePanel from "./PhonePanel.vue";

const parameters = new URLSearchParams(window.location.search);

createApp({
  render: () =>
    h(PhonePanel, {
      wasm: parameters.get("wasm") ?? "./softphone.wasm",
      endpoint: parameters.get("endpoint") ?? "ws://127.0.0.1:8780",
      label: parameters.get("label") ?? "Harness",
    }),
}).mount("#phone");
