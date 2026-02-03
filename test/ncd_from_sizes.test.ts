import { describe, expect, test } from "vitest";

import { gzipCompressor, ncd, ncdFromSizes } from "../src/ncd.js";

const enc = (s: string) => new TextEncoder().encode(s);

describe("ncdFromSizes", () => {
  test("matches ncd() when cx/cy provided", () => {
    const c = gzipCompressor(6);
    const x = enc("the quick brown fox\n".repeat(50));
    const y = enc("the quick brown box\n".repeat(50));

    const cx = c.compress(x).byteLength;
    const cy = c.compress(y).byteLength;

    const d1 = ncd(c, x, y);
    const d2 = ncdFromSizes(c, x, y, cx, cy);

    expect(Math.abs(d1 - d2)).toBeLessThan(1e-12);
  });
});
