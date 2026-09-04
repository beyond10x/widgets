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

// Where cargo actually put it. `CARGO_TARGET_DIR` is how a worktree keeps its build out of the
// tree, and a suite that reads only `./target` fails with ENOENT in every such tree.
const TARGET = process.env.CARGO_TARGET_DIR ?? "./target";
const MODULE = `${TARGET}/wasm32-unknown-unknown/release/softphone_shell.wasm`;
const SUITE = "./.build/suite.json";
// The model as the page reads it. A view assertion needs the declared `filter:` and the fields the
// row projects, and this is where both are published.
const CATALOG = "./.build/synth/web/catalog.json";
const verbose = process.argv.includes("--verbose");

// The bytes, and the bytes each time. `bridge.js` destructures `{ instance }` out of
// `WebAssembly.instantiate`, which is what that call returns for a buffer and not what it returns
// for a compiled `Module` — so passing the `Module` its own doc offers breaks it. See
// `upstream-blocker:ess-bridge-refuses-a-compiled-module`.
const bytes = await readFile(MODULE);
const suite = JSON.parse(await readFile(SUITE, "utf8"));
const declaredViews = new Map(
  JSON.parse(await readFile(CATALOG, "utf8")).views.map((view) => [view.name, view]),
);

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

/** Like `matches`, for a view assertion, whose values may name an instance an earlier step bound. */
function matchesRow(expected, row, captured) {
  for (const [field, node] of Object.entries(expected ?? {})) {
    const want = value(node, captured);
    const got = row?.[field];
    if (JSON.stringify(want) !== JSON.stringify(got)) {
      return `\`${field}\`: expected ${JSON.stringify(want)}, got ${JSON.stringify(got)}`;
    }
  }
  return null;
}

/**
 * The rows the query names, out of the projection the last response already carried.
 *
 * The wire hands every view over whole: `projected()` calls `bridge_by_id()` with no argument, so a
 * parameterised view arrives unfiltered and applying the parameters is the reader's half. This does
 * that, and it refuses rather than guesses. Each bound parameter has to appear in the view's
 * declared `filter:` as `x == param.x` and has to be a field the row projects; what is left of the
 * filter afterwards has to name no parameter at all. A remainder like `state == Active` is the
 * behaviour's own filter, already applied on the way out. A remainder still naming a parameter is a
 * filter this runner cannot evaluate — and handing back the unfiltered rows there would be an
 * assertion about the whole store wearing the query's name, which is worse than a refusal.
 */
function rowsFor(view, params, projection, captured) {
  const declared = declaredViews.get(view);
  if (!declared) {
    throw new Error(`the catalogue declares no view \`${view}\``);
  }
  const held = projection?.[view];
  if (!held) {
    throw new Error(`the response carries no rows for \`${view}\``);
  }
  const projects = new Set((declared.fields ?? []).map((field) => field.name));
  let remainder = declared.filter ?? "";
  let rows = held.rows ?? [];
  for (const [param, node] of Object.entries(params ?? {})) {
    const clause = `${param} == param.${param}`;
    if (!remainder.includes(clause)) {
      throw new Error(
        `\`${view}\` filters by \`${declared.filter}\`, which this runner cannot read as \`${clause}\``,
      );
    }
    if (!projects.has(param)) {
      throw new Error(`\`${view}\` is queried by \`${param}\`, which its row does not project`);
    }
    remainder = remainder.replace(clause, "");
    const want = value(node, captured);
    rows = rows.filter((row) => JSON.stringify(row[param]) === JSON.stringify(want));
  }
  if (remainder.includes("param.")) {
    throw new Error(
      `\`${view}\` filters by \`${declared.filter}\`, which names a parameter this step does not bind`,
    );
  }
  return rows;
}

async function run(name, scenario, tally) {
  const phone = await open(bytes);
  if (!phone.realized) throw new Error("the module exports no realization hook");
  const captured = {};
  let last = null;
  let command = null;
  let queried = null;

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
      // A refusal, which the wire writes as `{"outcome":"wrong-state","published":[],"refusal":{…}}`.
      //
      // The error and its fields are the measured half. The `published.length` line below is **not
      // a check and must not be read as one**: the generated wire writes `published` as an empty
      // literal on all 22 `WrongState` arms, so no implementation reachable through this bridge can
      // fail it, and no scenario can drive it red. It is a guard against a wire that changes its
      // mind, kept because the cost is one comparison and the failure it would catch is silent.
      //
      // What actually proves a refusal changed nothing is the `assert:` block — `query_view` and
      // `expect_view` below — because the store is the only place the difference shows. A refusal
      // that deleted the entity answers byte-for-byte identically to one that left it alone.
      case "expect_error": {
        const refusal = last?.outcome?.refusal;
        if (refusal?.error !== step.error) {
          throw new Error(
            `${at}: \`${command}\` reported \`${refusal?.error ?? "no declared error"}\`, ` +
              `not \`${step.error}\``,
          );
        }
        const failure = matches(step.fields, refusal.payload);
        if (failure !== null) {
          throw new Error(`${at}: \`${step.error}\` — ${failure}`);
        }
        const published = last?.outcome?.published ?? [];
        if (published.length !== 0) {
          const names = published.map((e) => e.event).join(", ");
          throw new Error(`${at}: \`${command}\` refused and published ${names}`);
        }
        break;
      }
      // The claim that stands on its own where there is no error to name: `HangUp`'s wrong-state
      // branch is `refuses: false`, so the only thing left to require of it is silence.
      //
      // Worth having only where `published` can be non-empty. After a refusal it cannot be, so each
      // of the three scenarios that assert refusals also puts one of these on a command that *did*
      // publish — rejecting a call must not answer it — which is the case where the comparison
      // below decides something rather than iterating an empty list.
      case "expect_no_event": {
        const published = last?.outcome?.published ?? [];
        if (published.some((e) => e.event === step.event)) {
          throw new Error(`${at}: \`${command}\` published \`${step.event}\` and must not have`);
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
      // The read, and the claim about it, are two steps because the read is an act: the same rows
      // answer every expectation that follows until the next query.
      case "query_view": {
        queried = {
          view: step.view,
          rows: rowsFor(step.view, step.params, last?.views, captured),
        };
        break;
      }
      case "expect_view": {
        // The step repeats the view rather than trusting position, so an expectation that does not
        // belong to the query before it is a suite defect and gets said out loud.
        if (queried?.view !== step.view) {
          throw new Error(
            `${at}: expects \`${step.view}\`, but the query before it read ` +
              `\`${queried?.view ?? "nothing"}\``,
          );
        }
        const { rows } = queried;
        const claim = step.expectation;
        switch (claim.expect) {
          case "counts": {
            if (claim.at_least === undefined && claim.at_most === undefined) {
              throw new Error(`${at}: a count with neither bound asks nothing of \`${step.view}\``);
            }
            if (claim.at_least !== undefined && rows.length < claim.at_least) {
              throw new Error(
                `${at}: \`${step.view}\` holds ${rows.length} row(s), fewer than ${claim.at_least}`,
              );
            }
            if (claim.at_most !== undefined && rows.length > claim.at_most) {
              throw new Error(
                `${at}: \`${step.view}\` holds ${rows.length} row(s), more than ${claim.at_most}`,
              );
            }
            break;
          }
          case "contains": {
            // An empty view fails this, which is the point: it is what tells a refusal that left
            // the entity alone from one that dropped it.
            const failures = rows.map((row) => matchesRow(claim.fields, row, captured));
            if (!failures.some((failure) => failure === null)) {
              const why = failures[0] ?? "it holds no rows at all";
              throw new Error(`${at}: no row of \`${step.view}\` matches — ${why}`);
            }
            break;
          }
          case "excludes": {
            const found = rows.findIndex((row) => matchesRow(claim.fields, row, captured) === null);
            if (found !== -1) {
              throw new Error(
                `${at}: \`${step.view}\` holds a row it must exclude, at ${found}: ` +
                  JSON.stringify(rows[found]),
              );
            }
            break;
          }
          // `at`, `ranked` and `satisfies` are refused rather than approximated. `at` and `ranked`
          // need the order the rows are relative to, and the only view here that declares one is
          // `softphone.history.RecentCalls`, which no scenario reads — so an implementation would
          // ship an untested comparator that mis-asserts in silence. `satisfies` needs a predicate
          // evaluator this runner does not have. Write the scenario first, then the case.
          default:
            throw new Error(
              `${at}: \`${claim.expect}\` is an expectation this runner does not implement`,
            );
        }
        break;
      }
      default:
        throw new Error(`${at}: a step kind this runner does not know`);
    }
    // Counted here, after the step held. A scenario that fails part-way still reports the steps it
    // ran: the total used to be added only for a scenario that completed, so two scenarios could be
    // added, break, and leave the headline figure unchanged.
    tally.executed += 1;
  }
  return { steps: scenario.steps.length, captured, observation: last };
}

let failed = 0;
const tally = { executed: 0 };
const names = Object.keys(suite.scenarios).sort();
for (const name of names) {
  try {
    const result = await run(name, suite.scenarios[name], tally);
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

const declared = names.reduce((total, name) => total + suite.scenarios[name].steps.length, 0);
console.log(
  `\n${names.length} scenario(s), ${tally.executed} of ${declared} step(s) executed, ` +
    `${failed} failure(s)` +
    `\nspec digest ${suite.provenance?.spec_digest?.slice(0, 12) ?? "unknown"}`,
);
process.exit(failed === 0 ? 0 : 1);
