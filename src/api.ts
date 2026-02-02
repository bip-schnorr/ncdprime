import { gzipCompressor } from "./ncd.js";
import type { Bytes } from "./ncd.js";
import { ncd } from "./ncd.js";
import { ncdFromSizes } from "./ncd.js";
import { joinFrame64 } from "./ncd.js";
import { tryLoadNative } from "./backend.js";

export type NcdApiOptions = {
  gzipLevel?: number;
  gzipMtime?: number;
};

/**
 * Library API: compute NCD, preferring the native backend when available.
 */
export function ncdAuto(x: Bytes, y: Bytes, opts: NcdApiOptions = {}): number {
  const gzipLevel = opts.gzipLevel ?? 6;
  const gzipMtime = opts.gzipMtime ?? 0;

  const native = tryLoadNative();
  if (native) return native.ncd(x, y, gzipLevel, gzipMtime);

  if (gzipMtime !== 0) {
    throw new Error("Nonzero gzipMtime requires native backend");
  }

  const c = gzipCompressor(gzipLevel);
  return ncd(c, x, y);
}

export type MatrixOptions = {
  gzipLevel?: number;
  gzipMtime?: number;
};

/**
 * Library API: compute an NCD matrix, preferring native backend when available.
 */
export async function matrixAuto(
  a: Bytes[],
  b: Bytes[],
  opts: MatrixOptions = {},
): Promise<number[][]> {
  const gzipLevel = opts.gzipLevel ?? 6;
  const gzipMtime = opts.gzipMtime ?? 0;

  const native = tryLoadNative();
  if (native) return native.matrix(a, b, gzipLevel, gzipMtime);

  if (gzipMtime !== 0) {
    throw new Error("Nonzero gzipMtime requires native backend");
  }

  // Pure JS fallback with singleton-size caching + ncdFromSizes.
  const c = gzipCompressor(gzipLevel);

  const { createHash } = await import("node:crypto");
  const cache = new Map<string, number>();
  const sizeOf = (x: Uint8Array): number => {
    const key = createHash("sha256").update(x).digest("hex");
    const hit = cache.get(key);
    if (hit != null) return hit;
    const v = c.compress(x).byteLength;
    cache.set(key, v);
    return v;
  };

  const aSizes = a.map(sizeOf);
  const bSizes = b.map(sizeOf);

  const out: number[][] = new Array(a.length);
  for (let i = 0; i < a.length; i++) {
    const row: number[] = new Array(b.length);
    for (let j = 0; j < b.length; j++) {
      row[j] = ncdFromSizes(c, a[i], b[j], aSizes[i], bSizes[j]);
    }
    out[i] = row;
  }

  return out;
}

// Re-export join helper (useful for callers who want to match the Rust framing).
export { joinFrame64 };
