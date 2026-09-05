// The harness: this widget, on a page of its own, with no devcenter and no build of anything else.
//
// It exists so the browser half can be tried before it is packaged — `task widget` serves it — and
// so a failure has one fewer place to be. The module's bytes come from a URL because that is what a
// served page has; a bundler hands the same component an asset instead.

import { createApp, h, ref } from "vue";
import PhonePanel from "./PhonePanel.vue";

const parameters = new URLSearchParams(window.location.search);
const to = parameters.get("to") ?? "";

// `?dial` places the call as soon as the panel is ready, with no click.
//
// That is what makes an audio check runnable without a person: a headless browser given a fake
// capture device can be driven this far and no further, because nothing here can click a button.
// It is off unless asked for, so opening the harness never dials by itself.
//
// It waits on the panel's own `ready` rather than on a delay. A delay was wrong in both
// directions: too short and the call is placed before the module is open, which fails by doing
// nothing; too long and every check pays for it.
const panel = ref<{ dial: () => Promise<void>; ready: Promise<void> } | null>(null);
const automatic = parameters.has("dial") && to !== "";

createApp({
  render: () =>
    h(PhonePanel, {
      ref: panel,
      wasm: parameters.get("wasm") ?? "./softphone.wasm",
      endpoint: parameters.get("endpoint") ?? "ws://127.0.0.1:8780",
      label: parameters.get("label") ?? "Harness",
      destination: to,
    }),
  mounted() {
    if (!automatic) return;
    void (async () => {
      await panel.value?.ready;
      await panel.value?.dial();
    })();
  },
}).mount("#phone");
