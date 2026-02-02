import { gzipSync } from "node:zlib";
export const gzipCompressor = (level = 6) => ({
    id: "gzip",
    name: `gzip (zlib, level=${level})`,
    compress(input) {
        // Deterministic gzip output requires fixed mtime.
        // @types/node may not expose mtime on gzip options, but Node supports it.
        return gzipSync(input, { level, mtime: 0 });
    },
});
export const concatWithSentinel = (a, b) => {
    // NCD for files/strings relies on C(xy). For binary data, we include a sentinel
    // that cannot be confused with stream concatenation in some compressors.
    // 0x00 is fine for most practical compressors; we’ll make this configurable later.
    const out = new Uint8Array(a.length + 1 + b.length);
    out.set(a, 0);
    out[a.length] = 0;
    out.set(b, a.length + 1);
    return out;
};
export const compressedSize = (c, x) => c.compress(x).byteLength;
/**
 * Normalized Compression Distance:
 *   NCD(x,y) = (C(xy) - min(C(x), C(y))) / max(C(x), C(y))
 */
export function ncd(c, x, y) {
    const cx = compressedSize(c, x);
    const cy = compressedSize(c, y);
    const cxy = compressedSize(c, concatWithSentinel(x, y));
    const min = Math.min(cx, cy);
    const max = Math.max(cx, cy);
    if (max === 0)
        return 0;
    return (cxy - min) / max;
}
