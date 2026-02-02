import { createRequire } from "node:module";

export type NativeBinding = {
  ncd: (x: Uint8Array, y: Uint8Array, gzipLevel?: number, gzipMtime?: number) => number;
  matrix: (
    a: Array<Uint8Array>,
    b: Array<Uint8Array>,
    gzipLevel?: number,
    gzipMtime?: number,
  ) => number[][];
};

/**
 * Best-effort native binding loader.
 *
 * Returns null if the native module isn't built/available.
 */
export function tryLoadNative(): NativeBinding | null {
  try {
    // dist/* is ESM; use createRequire to load the CJS entrypoint.
    const require = createRequire(import.meta.url);
    // Note: backend.ts compiles to dist/backend.js; from there, ../index.cjs is the root.
    return require("../index.cjs") as NativeBinding;
  } catch {
    return null;
  }
}
