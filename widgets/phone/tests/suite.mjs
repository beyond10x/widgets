// The authored scenarios, executed against the realized module.
//
// `task scenarios` compiles the documents under `scenarios/` into an `ess-conformance/3` suite;
// this replays every step of it through the same three WebAssembly exports and the same
// `bridge.js` a browser uses, and asserts what each step declares. So a green run says the
// specification is executable and this implementation obeys it — not that a page looks right.
//
// One module instance per scenario: a scenario starts from nothing, and sharing a store between
// two of them would make the order they run in part of the result.
//
// Usage: node tests/suite.mjs [--verbose]

import { readFile } from "node:fs/promises";
import { open } from "../.build/synth/web/bridge.js";

const MODULE = "./target/wasm32-unknown-unknown/release/softphone_shell.wasm";
const SUITE = "./.build/suite.json";
const verbose = process.argv.includes("--verbose");

// The bytes, and the bytes each time. `bridge.js` destructures `{ instance }` out of
// `WebAssembly.instantiate`, which is what that call returns for a buffer and not what it returns
// for a compiled `Module` — so passing the `Module` its own doc offers breaks it. See
// `upstream-blocker:ess-bridge-refuses-a-compiled-module`.
const bytes = await readFile(MODULE);
const suite = JSON.parse(await readFile(SUITE, "utf8"));

/** A step's input value: a literal as it stands, an instance as whatever captured it. */
function value(node, captured) {
  if (node === null || typeof node !== "object") return node;
  if (node.kind === "literal") return node.value;
  if (node.kind === "instance") {
    if (!(node.instance in captured)) {
      throw new Error(`no instance \`${node.instance}\` has been captured yet`);
    }
    return captured[node.instance];
  }
  throw new Error(`an input node this runner cannot read: ${JSON.stringify(node)}`);
}

/** Every field the step names has to be there, with that value. Extra fields are the event's. */
function matches(expected, actual) {
  for (const [field, want] of Object.entries(expected ?? {})) {
    const got = actual?.[field];
    if (JSON.stringify(want) !== JSON.stringify(got)) {
      return `\`${field}\`: expected ${JSON.stringify(want)}, got ${JSON.stringify(got)}`;
    }
  }
  return null;
}

async function run(name, scenario) {
  const phone = await open(bytes);
  if (!phone.realized) throw new Error("the module exports no realization hook");
  const captured = {};
  let last = null;
  let command = null;

  for (const [index, step] of scenario.steps.entries()) {
    const at = `${name} step ${index + 1} (${step.step})`;
    switch (step.step) {
      case "execute_command": {
        const input = {};
        for (const [field, node] of Object.entries(step.input ?? {})) {
          input[field] = value(node, captured);
        }
        command = step.command;
        last = phone.request({ request: "command", command: step.command, input });
        if (!last.ok) {
          throw new Error(`${at}: \`${step.command}\` refused — ${JSON.stringify(last.error)}`);
        }
        break;
      }
      case "expect_outcome": {
        const want = step.outcome.outcome;
        if (last?.outcome?.outcome !== want) {
          throw new Error(
            `${at}: \`${command}\` took \`${last?.outcome?.outcome}\`, not \`${want}\``,
          );
        }
        break;
      }
      case "expect_event": {
        const published = last?.outcome?.published ?? [];
        const emitted = published.filter((e) => e.event === step.event);
        if (emitted.length === 0) {
          const names = published.map((e) => e.event).join(", ") || "nothing";
          throw new Error(`${at}: \`${step.event}\` was not published; ${names} was`);
        }
        const failures = emitted.map((e) => matches(step.payload, e.payload));
        if (!failures.some((f) => f === null)) {
          throw new Error(`${at}: \`${step.event}\` payload — ${failures[0]}`);
        }
        break;
      }
      case "capture_instance": {
        const event = (last?.outcome?.published ?? []).find((e) => e.event === step.event);
        if (!event) {
          throw new Error(`${at}: nothing published \`${step.event}\` to capture from`);
        }
        const held = event.payload?.[step.field];
        if (held === undefined) {
          throw new Error(`${at}: \`${step.event}\` carries no \`${step.field}\``);
        }
        captured[step.instance] = held;
        break;
      }
      default:
        throw new Error(`${at}: a step kind this runner does not know`);
    }
  }
  return { steps: scenario.steps.length, captured, observation: last };
}

let failed = 0;
let steps = 0;
const names = Object.keys(suite.scenarios).sort();
for (const name of names) {
  try {
    const result = await run(name, suite.scenarios[name]);
    steps += result.steps;
    console.log(`ok    ${name} — ${result.steps} step(s)`);
    if (verbose) {
      console.log(`      instances: ${JSON.stringify(result.captured)}`);
    }
  } catch (error) {
    failed += 1;
    console.log(`FAIL  ${name}`);
    console.log(`      ${error.message}`);
  }
}

console.log(
  `\n${names.length} scenario(s), ${steps} step(s) executed, ${failed} failure(s)` +
    `\nspec digest ${suite.provenance?.spec_digest?.slice(0, 12) ?? "unknown"}`,
);
process.exit(failed === 0 ? 0 : 1);
