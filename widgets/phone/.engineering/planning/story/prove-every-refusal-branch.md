---
format: aep.planning-md/1
id: story:prove-every-refusal-branch
kind: story
status: draft
title: Prove every refusal branch the model declares
relations:
- decomposes: epic:browser-softphone
- depends_on: story:call-lifecycle
scope:
- confidence: cited
  path: scenarios
- confidence: cited
  path: tests/suite.mjs
revision: 2
---
# Prove every refusal branch the model declares

## Outcome

`node tests/suite.mjs` refuses when the specification declares a `wrong-state` branch no scenario
takes. Today it counts 26 such branches; `story:call-lifecycle` proved 12, and the gate cannot be
turned on until the other 14 have scenarios.

## Why a check and not a list

A refusal nobody wrote a scenario for is invisible, and a green suite says nothing about it. That
is exactly the state `softphone.control.CallStateConflict` was in through eight commands: fully
implemented, and an implementation that answered `Answered` for a call already `Ended` would have
passed every scenario in the repository. `catalog.json` enumerates the branches, so a check reads
them and a new `wrong-state` outcome in the specification adds an obligation here without anybody
remembering to.

## The 14 that remain

```
softphone.directory.DeleteContact      softphone.presentation.EnterDialing
softphone.directory.RemoveAddress      softphone.presentation.EnterIncoming
softphone.history.DeleteRecord         softphone.presentation.LeaveCall
softphone.local.DetachLoopback         softphone.sip.CloseDialog
softphone.presentation.DismissCall     softphone.sip.ConfirmRegistration
softphone.presentation.EnterCall       softphone.sip.FailRegistration
                                       softphone.sip.RetryRegistration
                                       softphone.sip.Unregister
```

Five domains, so the scenarios belong with those domains' own stories rather than in one document.

## The check, as it was written and measured

`story:call-lifecycle`'s implementor wrote it, ran it, confirmed it names exactly those 14 and exits
1, and then took it back out — a gate that fails on work nobody was assigned breaks the integration
branch. `git apply --check` exited 0 against the tree at that point. It is recorded here because
the worktree it was written in does not survive the wave.

```diff
--- a/widgets/phone/tests/suite.mjs
+++ b/widgets/phone/tests/suite.mjs
@@
 const MODULE = "./target/wasm32-unknown-unknown/release/softphone_shell.wasm";
 const SUITE = "./.build/suite.json";
+// The model as the page reads it, which is where the refusal branches are enumerated.
+const CATALOG = "./.build/synth/web/catalog.json";
@@
 const suite = JSON.parse(await readFile(SUITE, "utf8"));
+const catalog = JSON.parse(await readFile(CATALOG, "utf8"));
@@
+// Every refusal the model declares has to be taken by some scenario.
+const declared = catalog.commands
+  .filter((command) => (command.outcomes ?? []).some((o) => o.name === "wrong-state"))
+  .map((command) => command.name);
+const taken = new Set();
+for (const scenario of Object.values(suite.scenarios)) {
+  for (const step of scenario.steps) {
+    if (step.step === "expect_outcome" && step.outcome.outcome === "wrong-state") {
+      taken.add(step.outcome.command);
+    }
+  }
+}
+const unproven = declared.filter((command) => !taken.has(command)).sort();
+for (const command of unproven) {
+  console.log(`FAIL  ${command} — declares a wrong-state branch no scenario takes`);
+}
+
 console.log(
   `\n${names.length} scenario(s), ${steps} step(s) executed, ${failed} failure(s)` +
+    `\n${declared.length - unproven.length}/${declared.length} declared refusal(s) proven` +
     `\nspec digest ${suite.provenance?.spec_digest?.slice(0, 12) ?? "unknown"}`,
 );
-process.exit(failed === 0 ? 0 : 1);
+process.exit(failed === 0 && unproven.length === 0 ? 0 : 1);
```

## Acceptance

Each of the 14 has a scenario that takes its `wrong-state` branch, asserting the error the model
declares and that nothing was published; the patch above is applied; and
`node tests/suite.mjs` prints `26/26 declared refusal(s) proven` and exits 0.

## Scope

Derived by the coordinator from the unit's report, confidence **high** — the branch list is read
from `.build/synth/web/catalog.json` and the patch was run.

- `scenarios` — cited, the 14 scenarios land here
- `tests/suite.mjs` — cited, the patch above
- **Would collide with** any unit touching `scenarios/` or `tests/suite.mjs`.
