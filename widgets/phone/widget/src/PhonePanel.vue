<!--
  The phone, as a person sees it.

  Every field on this screen is a row the module projected. There is no reactive copy of a call
  here and no local `state` variable: `observed` replaces the whole projection, so what is drawn is
  what the module holds. That is the difference the specification draws between a call and a tile —
  state against screen — and the way to keep it is to render the projection rather than shadow it.

  It reads `softphone.control.CallById` rather than `ActiveCalls`, which is not a mistake:
  `ActiveCalls` filters `state == Active`, so a call that is ringing is not in it, and an
  `observe` that binds no parameter answers the parameterised view over the whole source. Measured
  2026-09-05 — a dialled call appears there as `Requested` and walks to `Active`.

  History carries no time. The module's timestamps are ordinal — `1970-01-01T00:00:01Z` and up —
  because `bridge.js` instantiates the module with no imports at all, so the wasm has no clock to
  read. Showing them as if they were wall-clock times would be the page inventing a fact.
  `story:the-module-has-no-clock` carries what it would take to have one.
-->
<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, shallowRef } from "vue";
import { Kernel, type Observation, type Row } from "./kernel";
import { Phone } from "./call";

const props = withDefaults(
  defineProps<{
    /** The module's bytes, or a URL to fetch them from. */
    wasm: BufferSource | string;
    /** Where `phone-server` listens. */
    endpoint?: string;
    /** What this phone calls itself in the model. */
    label?: string;
    /** `RTCPeerConnection` configuration — `{iceServers}` for STUN. */
    configuration?: RTCConfiguration;
    /** What the dial field starts with. A host that knows where this phone calls sets it. */
    destination?: string;
  }>(),
  {
    endpoint: "ws://127.0.0.1:8780",
    label: "Devcenter",
    configuration: () => ({}),
    destination: "",
  },
);

const phone = shallowRef<Phone | null>(null);
const observation = ref<Observation | null>(null);
const said = ref<string>("");
const destination = ref<string>(props.destination);
const entry = ref<string>("");
const failed = ref<string>("");

// Resolved once the module is open and the endpoint configured, or once that has failed.
//
// Exposed rather than kept private because a host that places a call without a person clicking has
// to know when there is something to place it with. The alternative — a fixed delay — is what this
// replaces: 400 ms was enough for a warm module and not for a cold one, so the harness sometimes
// dialled before `phone` existed and then did nothing at all, silently.
let settle: () => void = () => {};
const ready = new Promise<void>((resolve) => {
  settle = resolve;
});

const KEYS = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "*", "0", "#"] as const;

function rows(view: string): Row[] {
  return observation.value?.views[view]?.rows ?? [];
}

/** The one call this phone holds, whatever state it is in. */
const call = computed<Row | undefined>(() =>
  rows("softphone.control.CallById").find((row) => row["state"] !== "Ended"),
);
const live = computed(() => call.value !== undefined);
const muted = computed(() => call.value?.["muted"] === true);
const held = computed(() => call.value?.["held"] === true);
const history = computed(() => rows("softphone.history.RecentCalls"));
const log = computed(() => (observation.value?.log ?? []).slice(-8).reverse());

onMounted(async () => {
  try {
    const bytes =
      typeof props.wasm === "string"
        ? await (await fetch(props.wasm)).arrayBuffer()
        : props.wasm;
    const kernel = await Kernel.open(bytes);
    const it = new Phone(kernel, props.endpoint, {
      observed: (next) => (observation.value = next),
      said: (words) => (said.value = words),
    });
    it.configure(props.label);
    phone.value = it;
    observation.value = it.observe();
  } catch (error) {
    failed.value = String(error);
  } finally {
    settle();
  }
});

onBeforeUnmount(() => phone.value?.hangUp());

/** Every action reports its own failure rather than throwing into the render. */
async function attempt(what: () => void | Promise<void>): Promise<void> {
  said.value = "";
  try {
    await what();
  } catch (error) {
    said.value = String(error);
  }
}

const dial = () =>
  attempt(async () => {
    const to = destination.value.trim() || entry.value.trim();
    if (!to) throw new Error("nothing to dial");
    await phone.value?.dial(to, props.configuration);
  });
defineExpose({ dial, ready });

const press = (key: string) => {
  entry.value += key;
  if (live.value) void attempt(() => phone.value?.sendDigits(key));
};
</script>

<template>
  <section class="phone">
    <p v-if="failed" class="phone-failed">{{ failed }}</p>
    <template v-else>
      <div class="phone-bezel">
        <div class="phone-screen">
          <template v-if="call">
            <p class="phone-state">{{ call.state }}</p>
            <p class="phone-remote">{{ call.remote }}</p>
            <p class="phone-flags">
              <span v-if="muted">muted</span>
              <span v-if="held">held</span>
            </p>
          </template>
          <p v-else class="phone-idle">no call</p>
          <p class="phone-entry">{{ entry || "—" }}</p>
        </div>

        <div class="phone-pad">
          <button v-for="key of KEYS" :key="key" type="button" @click="press(key)">
            {{ key }}
          </button>
        </div>

        <label class="phone-to">
          <span>call</span>
          <input v-model="destination" type="text" placeholder="sip:1001@…" @keyup.enter="dial" />
        </label>

        <div class="phone-actions">
          <button type="button" :disabled="live || !phone" @click="dial">dial</button>
          <button type="button" :disabled="!live" @click="attempt(() => phone?.hangUp())">
            hang up
          </button>
          <button type="button" :disabled="!live" @click="attempt(() => phone?.setMuted(!muted))">
            {{ muted ? "unmute" : "mute" }}
          </button>
          <button type="button" :disabled="!live" @click="attempt(() => phone?.setHeld(!held))">
            {{ held ? "resume" : "hold" }}
          </button>
          <button type="button" :disabled="!entry" @click="entry = ''">clear</button>
        </div>

        <p v-if="said" class="phone-said">{{ said }}</p>
      </div>

      <div class="phone-beside">
        <h3>calls</h3>
        <p v-if="!history.length" class="phone-idle">nothing recorded yet</p>
        <table v-else>
          <tbody>
            <tr v-for="record of history" :key="String(record.record_id)">
              <td>{{ record.direction }}</td>
              <td class="phone-mono">{{ record.remote }}</td>
              <td>{{ record.termination }}</td>
            </tr>
          </tbody>
        </table>

        <h3>log</h3>
        <ol class="phone-log">
          <li v-for="line of log" :key="line.occurrence" class="phone-mono">
            {{ line.event.replace("softphone.", "") }}
          </li>
        </ol>
      </div>
    </template>
  </section>
</template>

<style scoped>
.phone {
  display: flex;
  gap: 14px;
  flex-wrap: wrap;
  font: 14px/1.5 ui-sans-serif, system-ui, sans-serif;
}
.phone-bezel {
  width: 15rem;
  border: 1px solid currentColor;
  border-radius: 16px;
  padding: 10px;
  opacity: 0.95;
}
.phone-screen {
  min-height: 6rem;
  border: 1px solid currentColor;
  border-radius: 9px;
  padding: 8px;
}
.phone-state {
  margin: 0;
  font-family: ui-monospace, monospace;
  font-size: 0.75rem;
  text-transform: lowercase;
}
.phone-remote {
  margin: 0.2rem 0 0;
  word-break: break-all;
}
.phone-flags {
  margin: 0.2rem 0 0;
  display: flex;
  gap: 0.4rem;
  font-size: 0.7rem;
  text-transform: uppercase;
}
.phone-idle,
.phone-entry {
  margin: 0.3rem 0 0;
  opacity: 0.6;
}
.phone-entry {
  font-family: ui-monospace, monospace;
  text-align: right;
}
.phone-pad {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 4px;
  margin-top: 8px;
}
.phone-to {
  display: grid;
  gap: 0.15rem;
  margin-top: 0.6rem;
  font-size: 0.75rem;
}
.phone-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 0.6rem;
}
.phone-actions button,
.phone-pad button {
  font: inherit;
  padding: 0.3rem 0.5rem;
  border: 1px solid currentColor;
  border-radius: 5px;
  background: transparent;
  color: inherit;
  cursor: pointer;
}
.phone-actions button:disabled,
.phone-pad button:disabled {
  opacity: 0.4;
  cursor: default;
}
.phone-to input {
  font: inherit;
  padding: 0.3rem;
  border: 1px solid currentColor;
  border-radius: 5px;
  background: transparent;
  color: inherit;
}
.phone-said,
.phone-failed {
  margin: 0.5rem 0 0;
  font-size: 0.8rem;
}
.phone-beside {
  flex: 1;
  min-width: 14rem;
}
.phone-beside h3 {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  opacity: 0.6;
  margin: 0.5rem 0 0.2rem;
}
.phone-beside table {
  border-collapse: collapse;
  width: 100%;
}
.phone-beside td {
  padding: 0.2rem 0.4rem 0.2rem 0;
  border-bottom: 1px solid currentColor;
  font-size: 0.8rem;
}
.phone-mono {
  font-family: ui-monospace, monospace;
  font-size: 0.75rem;
}
.phone-log {
  margin: 0;
  padding-left: 1.1rem;
}
</style>
