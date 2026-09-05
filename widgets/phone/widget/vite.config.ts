import { fileURLToPath, URL } from "node:url";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

// `#ess-bridge` is the emitted `bridge.js`, resolved out of `.build/` at build time and inlined
// into `dist/`.
//
// An alias rather than a committed copy, and rather than a re-implementation. That file is 30 lines
// of glue over three wasm exports, and its own header says why it exists: the page imports it "so
// what the test exercises is the page's own glue and not a second implementation that happens to
// agree with it". Writing those three calls again here would be exactly the second implementation.
// Copying it into `src/` would put a generated file under version control, where it drifts from the
// specification silently.
//
// The consequence is stated rather than hidden: a build needs `task synthesize` to have run, so a
// consumer installing this package from git needs the built `dist/` in the package rather than a
// `prepare` script that would have no `.build/` to read.
const ESS_BRIDGE = fileURLToPath(new URL("../.build/synth/web/bridge.js", import.meta.url));

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { "#ess-bridge": ESS_BRIDGE },
  },
  build: {
    lib: {
      entry: fileURLToPath(new URL("./src/index.ts", import.meta.url)),
      formats: ["es"],
      fileName: () => "index.js",
    },
    // Vue stays the consumer's: devcenter has its own, and two Vue runtimes in one page do not
    // share a reactivity system.
    rollupOptions: { external: ["vue"] },
    sourcemap: true,
  },
});
