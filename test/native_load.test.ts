import { describe, expect, test } from "vitest";
import { existsSync } from "node:fs";
import { join } from "node:path";

// This test is intentionally "soft": it only runs assertions if the native
// module has been built (npm/index.node exists).

describe("native binding", () => {
  test("ncdprime/native loads when built", async () => {
    const p = join(process.cwd(), "npm", "index.node");
    if (!existsSync(p)) {
      // Not built in this environment.
      return;
    }

    const native = await import("ncdprime/native");
    expect(typeof native.ncd).toBe("function");
    expect(typeof native.matrix).toBe("function");
  });
});
