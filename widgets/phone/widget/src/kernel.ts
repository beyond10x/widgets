// The phone's state, which is the module's and never a copy of it.
//
// Every command goes through the same three wasm exports a browser uses, and the answer already
// carries the whole observation — `{ok, command, outcome, log, invocations, views}` — so there is
// no second request to read state back and no window in which this file's idea of a call differs
// from the module's. A field held here beside the module would be exactly that window.

import { open, type Driver } from "#ess-bridge";

/** One projected row. Fields are the view's own; nothing here knows their names. */
export type Row = Record<string, unknown>;

/** What the module published for one command. */
export interface Published {
  event: string;
  payload: Record<string, unknown>;
}

/** A declared refusal: the error's name, and the fields it carries. */
export interface Refusal {
  error: string;
  payload: Record<string, unknown>;
}

/** One entry of the event log, in the order the module wrote it. */
export interface Entry {
  occurrence: number;
  event: string;
  payload: Record<string, unknown>;
}

/** What one command answered, and the whole store as it stands afterwards. */
export interface Observation {
  /** False only for a request the module would not parse — a refusal is `true` with an outcome. */
  ok: boolean;
  command?: string;
  outcome?: {
    outcome: string;
    published?: Published[];
    /** Present when the outcome is a declared refusal. */
    refusal?: Refusal;
  };
  /** Present when `ok` is false. */
  error?: unknown;
  log: Entry[];
  invocations: unknown[];
  views: Record<string, { rows: Row[] }>;
}

/** A command the module would not parse, or a module that answered nothing. */
export class DispatchFailed extends Error {}

/**
 * The realized system, over one instantiated module.
 *
 * `Kernel.open` takes the bytes rather than fetching them, because where the module comes from is
 * the host's business: the harness page fetches it from a URL, and a bundler hands the consumer an
 * asset. Bytes and not a compiled `WebAssembly.Module`: the emitted bridge destructures
 * `{instance}` out of `WebAssembly.instantiate`, which a compiled module's result does not carry.
 */
export class Kernel {
  #driver: Driver;

  private constructor(driver: Driver) {
    this.#driver = driver;
  }

  /**
   * Instantiates the module and checks that it is the realized one.
   *
   * A module without the realization hook answers every command with an unmet obligation, which
   * looks like a phone that refuses everything for no stated reason. Refusing here names it once.
   */
  static async open(wasm: BufferSource): Promise<Kernel> {
    const driver = await open(wasm);
    if (!driver.realized) {
      throw new DispatchFailed(
        "this module exports no `ess_realize`, so every command would refuse as an unmet " +
          "obligation — build `softphone-shell`, not the generated `softphone-web`",
      );
    }
    return new Kernel(driver);
  }

  /** The catalogue the model determines: commands, views, types, provenance. */
  catalog(): Record<string, unknown> {
    const answer = this.#driver.request({ request: "catalog" });
    return (answer["catalog"] ?? {}) as Record<string, unknown>;
  }

  /** The log, the binding invocations and every view's rows, without issuing a command. */
  observe(): Observation {
    return this.#driver.request({ request: "observe" }) as unknown as Observation;
  }

  /**
   * Runs one command and answers the observation that follows it.
   *
   * A declared refusal is a return value, not a throw: `wrong-state` is a fact about the phone
   * that a person should see on the screen. Only a request the module will not parse throws, since
   * that is this file disagreeing with the model rather than a user doing something out of turn.
   */
  run(command: string, input: Record<string, unknown>): Observation {
    const answer = this.#driver.request({
      request: "command",
      command,
      input,
    }) as unknown as Observation;
    if (!answer.ok) {
      throw new DispatchFailed(`\`${command}\` was not accepted: ${JSON.stringify(answer.error)}`);
    }
    return answer;
  }
}

/**
 * One field of the event a command published, or `undefined` when it published no such event.
 *
 * How every identifier in this package is obtained: the module mints them, and the page reads them
 * back out of the outcome. The alternative — minting one here and telling the module about it —
 * would put two authorities on identity in one phone.
 */
export function published(
  observation: Observation,
  event: string,
  field: string,
): string | undefined {
  const found = (observation.outcome?.published ?? []).find((one) => one.event === event);
  const value = found?.payload[field];
  return typeof value === "string" ? value : undefined;
}

/** The rows of one view, or none when the module does not project it. */
export function rows(observation: Observation, view: string): Row[] {
  return observation.views[view]?.rows ?? [];
}
