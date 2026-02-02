import { gzipSync, type ZlibOptions } from "node:zlib";

export type Bytes = Uint8Array;

export interface Compressor {
  /** Unique id (e.g. "gzip"). */
  id: string;
  /** Human name (e.g. "gzip (zlib)"). */
  name: string;
  compress(input: Bytes): Bytes;
}

export const gzipCompressor = (level = 6): Compressor => ({
  id: "gzip",
  name: `gzip (zlib, level=${level})`,
  compress(input) {
    // Deterministic gzip output requires fixed mtime.
    // @types/node may not expose mtime on gzip options, but Node supports it.
    return gzipSync(input, { level, mtime: 0 } as unknown as ZlibOptions);
  },
});

export const frame64 = (b: Bytes): Bytes => {
  // u64_le(len) || bytes
  const out = new Uint8Array(8 + b.length);
  const dv = new DataView(out.buffer, out.byteOffset, out.byteLength);
  // lengths > 2^53 aren't representable in JS numbers precisely; for our typical inputs
  // this is fine.
  dv.setBigUint64(0, BigInt(b.length), true);
  out.set(b, 8);
  return out;
};

export const joinFrame64 = (a: Bytes, b: Bytes): Bytes => {
  const fa = frame64(a);
  const fb = frame64(b);
  const out = new Uint8Array(fa.length + fb.length);
  out.set(fa, 0);
  out.set(fb, fa.length);
  return out;
};

export const compressedSize = (c: Compressor, x: Bytes): number => c.compress(x).byteLength;

/**
 * Normalized Compression Distance:
 *   NCD(x,y) = (C(xy) - min(C(x), C(y))) / max(C(x), C(y))
 */
export function ncd(c: Compressor, x: Bytes, y: Bytes): number {
  // Match the Rust core defaults:
  // - join=frame64
  // - symmetry=min(C(xy), C(yx))
  const cx = compressedSize(c, x);
  const cy = compressedSize(c, y);
  const min = Math.min(cx, cy);
  const max = Math.max(cx, cy);
  if (max === 0) return 0;

  const cxy = compressedSize(c, joinFrame64(x, y));
  const cyx = compressedSize(c, joinFrame64(y, x));
  const ccat = Math.min(cxy, cyx);

  const d = (ccat - min) / max;
  if (Number.isNaN(d)) return 0;
  return d;
}
