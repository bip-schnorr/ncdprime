import { describe, expect, test } from "vitest";
import { gzipCompressor, ncd } from "../src/ncd.js";
const enc = (s) => new TextEncoder().encode(s);
describe("ncd", () => {
    test("identity is small (x vs x)", () => {
        const c = gzipCompressor(6);
        const x = enc("the quick brown fox jumps over the lazy dog\n".repeat(20));
        const d = ncd(c, x, x);
        expect(d).toBeGreaterThanOrEqual(0);
        // In practice should be close-ish to 0, but allow some slack.
        expect(d).toBeLessThan(0.25);
    });
    test("different inputs are farther than identical (typical)", () => {
        const c = gzipCompressor(6);
        const x = enc("aaaaaa".repeat(200));
        // Deterministic pseudo-random bytes (so test is stable).
        let seed = 123456789;
        const rand = () => {
            seed = (1103515245 * seed + 12345) % 2 ** 31;
            return seed & 0xff;
        };
        const y = new Uint8Array(x.length);
        for (let i = 0; i < y.length; i++)
            y[i] = rand();
        const dxx = ncd(c, x, x);
        const dxy = ncd(c, x, y);
        expect(dxy).toBeGreaterThan(dxx);
    });
    test("symmetry is approximately true", () => {
        const c = gzipCompressor(6);
        const x = enc("abc".repeat(500));
        const y = enc("abd".repeat(500));
        const dxy = ncd(c, x, y);
        const dyx = ncd(c, y, x);
        expect(Math.abs(dxy - dyx)).toBeLessThan(0.1);
    });
});
