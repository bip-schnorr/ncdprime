import { describe, expect, test } from "vitest";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

async function runCli(args: string[]): Promise<{ stdout: string; stderr: string; code: number }> {
  const node = process.execPath;
  const cli = join(process.cwd(), "dist", "cli.js");
  try {
    const { stdout, stderr } = await execFileAsync(node, [cli, ...args], { encoding: "utf8" });
    return { stdout, stderr, code: 0 };
  } catch (err: any) {
    return { stdout: err.stdout ?? "", stderr: err.stderr ?? String(err), code: err.code ?? 1 };
  }
}

describe("matrix CLI", () => {
  test("square directory mode emits header + NxN table", async () => {
    const dir = await mkdtemp(join(tmpdir(), "ncdprime-"));
    await writeFile(join(dir, "a.txt"), "aaaaaa".repeat(100));
    await writeFile(join(dir, "b.txt"), "bbbbbb".repeat(100));

    // Ensure dist exists.
    await execFileAsync("npm", ["run", "build"], { encoding: "utf8" });

    const r = await runCli(["matrix", "--square", "--format", "tsv", dir]);
    expect(r.code).toBe(0);

    const lines = r.stdout.trimEnd().split(/\r?\n/);
    // Header row: empty cell + 2 cols
    expect(lines[0].split("\t").length).toBe(1 + 2);
    // Then 2 data rows.
    expect(lines.length).toBe(1 + 2);
  });
});
