#!/usr/bin/env node
import { Command } from "commander";
import { readFile } from "node:fs/promises";
import { gzipCompressor, ncd } from "./ncd.js";

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
  .action(async (fileA, fileB, opts) => {
    if (opts.compressor !== "gzip") {
      console.error(`Unsupported compressor: ${opts.compressor}`);
      process.exitCode = 2;
      return;
    }
    const c = gzipCompressor(opts.gzipLevel);
    const [a, b] = await Promise.all([readFile(fileA), readFile(fileB)]);
    const d = ncd(c, a, b);
    process.stdout.write(d.toString() + "\n");
  });

await program.parseAsync(process.argv);
