import { describe, expect, test } from "vitest";

const enc = (s: string) => new TextEncoder().encode(s);

// Import from dist to match real consumer behavior.
import { matrixAuto, ncdAuto } from "../dist/index.js";

describe("api auto", () => {
  test("ncdAuto works without native (mtime=0)", () => {
    const d = ncdAuto(enc("aaaa"), enc("aaaa"), { gzipLevel: 6, gzipMtime: 0 });
    expect(d).toBeGreaterThanOrEqual(0);
    // Should be small-ish for identical inputs.
    expect(d).toBeLessThan(0.6);
  });

  test("ncdAuto throws if gzipMtime != 0 and native is unavailable", () => {
    expect(() => ncdAuto(enc("aaaa"), enc("aaaa"), { gzipLevel: 6, gzipMtime: 123 })).toThrow(
      /requires native backend/i,
    );
  });

  test("matrixAuto returns matrix with correct shape", async () => {
    const a = [enc("aaa"), enc("bbb"), enc("ccc")];
    const b = [enc("aaa"), enc("bbb")];
    const m = await matrixAuto(a, b, { gzipLevel: 6, gzipMtime: 0 });

    expect(m.length).toBe(3);
    expect(m[0].length).toBe(2);
    expect(m[1].length).toBe(2);
    expect(m[2].length).toBe(2);
  });
});
