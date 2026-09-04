// The emitted `bridge.js`, as this package sees it.
//
// Hand-written because the file it describes carries `do not edit` and no types: `vite.config.ts`
// resolves `#ess-bridge` to `.build/synth/web/bridge.js`, and this declaration is the only place
// its three exports are named in TypeScript. It is a description of a generated contract, so it
// changes when the emitter does and never on its own.
declare module "#ess-bridge" {
  /** What one dispatch answers. Every request kind returns an object with `ok`. */
  export interface Answer {
    ok: boolean;
    [member: string]: unknown;
  }

  /** A module, instantiated, with its realization hook already called. */
  export interface Driver {
    /** One request in, one answer out. Synchronous: the module is not async. */
    request: (body: unknown) => Answer;
    /** Whether the module exported `ess_realize` — false means every command refuses. */
    realized: boolean;
    exports: WebAssembly.Exports;
  }

  /** Instantiates the module. `source` is the bytes of a `.wasm` or a compiled `Module`. */
  export function open(source: BufferSource | WebAssembly.Module): Promise<Driver>;

  export const EXPORTS: readonly string[];
  export const REALIZE: string;
}
