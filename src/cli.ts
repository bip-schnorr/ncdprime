#!/usr/bin/env node
import { Command } from "commander";
import { readFile } from "node:fs/promises";
import { gzipCompressor, ncd } from "./ncd.js";
import { tryLoadNative } from "./backend.js";

const program = new Command();

program
  .name("ncdprime")
  .description("Modern NCD (Normalized Compression Distance) utility")
  .version("0.1.0");

program
  .command("pair")
  .description("Compute NCD between two files")
  .argument("<fileA>")
  .argument("<fileB>")
  .option("--compressor <id>", "Compressor id (currently only: gzip)", "gzip")
  .option("--gzip-level <n>", "gzip level (0-9)", (v) => Number.parseInt(v, 10), 6)
  .option(
    "--gzip-mtime <n>",
    "gzip header mtime (timestamp). Use 0 for deterministic output",
    (v) => Number.parseInt(v, 10),
    0,
  )
  .action(async (fileA, fileB, opts) => {
    if (opts.compressor !== "gzip") {
      console.error(`Unsupported compressor: ${opts.compressor}`);
      process.exitCode = 2;
      return;
    }
    const [a, b] = await Promise.all([readFile(fileA), readFile(fileB)]);

    const native = tryLoadNative();
    if (native) {
      const d = native.ncd(a, b, opts.gzipLevel, opts.gzipMtime);
      process.stdout.write(d.toString() + "\n");
      return;
    }

    // Fallback: pure JS implementation (still deterministic because gzipCompressor uses mtime=0)
    if (opts.gzipMtime !== 0) {
      console.error("Nonzero --gzip-mtime requires the native backend.");
      process.exitCode = 2;
      return;
    }

    const c = gzipCompressor(opts.gzipLevel);
    const d = ncd(c, a, b);
    process.stdout.write(d.toString() + "\n");
  });

program
  .command("matrix")
  .description(
    "Compute an NCD matrix between two sets (dirs, files, list files, or literals).\n\n" +
      "Set detection: directories enumerate files; otherwise pass --list to treat arg as newline-separated file list; otherwise treat as single file (or literal if path does not exist).",
  )
  .argument("<setA>")
  .argument("[setB]")
  .option("-s, --square", "Use setA for both axes", false)
  .option("--list", "Interpret set args as newline-separated file-list files", false)
  .option("--no-labels", "Omit row/column labels")
  .option("--format <fmt>", "Output format (tsv|csv)", "tsv")
  .option("--compressor <id>", "Compressor id (currently only: gzip)", "gzip")
  .option("--gzip-level <n>", "gzip level (0-9)", (v) => Number.parseInt(v, 10), 6)
  .option(
    "--gzip-mtime <n>",
    "gzip header mtime (timestamp). Use 0 for deterministic output",
    (v) => Number.parseInt(v, 10),
    0,
  )
  .action(async (setA, setB, opts) => {
    const { autoDetectSetSpec, loadSet } = await import("./inputs.js");
    const { computeMatrix, formatMatrix } = await import("./matrix.js");

    if (opts.compressor !== "gzip") {
      console.error(`Unsupported compressor: ${opts.compressor}`);
      process.exitCode = 2;
      return;
    }
    const specA = await autoDetectSetSpec(setA, { list: opts.list });
    const specB = opts.square ? specA : await autoDetectSetSpec(setB ?? setA, { list: opts.list });

    const [a, b] = await Promise.all([loadSet(specA), loadSet(specB)]);

    const fmt = opts.format === "csv" ? "csv" : "tsv";

    const native = tryLoadNative();
    if (native) {
      const values = native.matrix(
        a.items.map((i) => i.bytes),
        b.items.map((i) => i.bytes),
        opts.gzipLevel,
        opts.gzipMtime,
      );
      const m = { rows: a.items.map((i) => i.label), cols: b.items.map((i) => i.label), values };
      process.stdout.write(formatMatrix(m, fmt, Boolean(opts.labels)));
      return;
    }

    // Fallback: pure JS implementation (still deterministic because gzipCompressor uses mtime=0)
    if (opts.gzipMtime !== 0) {
      console.error("Nonzero --gzip-mtime requires the native backend.");
      process.exitCode = 2;
      return;
    }

    const c = gzipCompressor(opts.gzipLevel);
    const m = computeMatrix(c, a, b);
    process.stdout.write(formatMatrix(m, fmt, Boolean(opts.labels)));
  });

await program.parseAsync(process.argv);
