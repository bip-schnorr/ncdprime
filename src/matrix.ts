import type { Compressor } from "./ncd.js";
import { ncd } from "./ncd.js";
import type { InputSet } from "./inputs.js";

export type Matrix = {
  rows: string[];
  cols: string[];
  values: number[][]; // rows x cols
};

export function computeMatrix(c: Compressor, a: InputSet, b: InputSet): Matrix {
  const rows = a.items.map((i) => i.label);
  const cols = b.items.map((i) => i.label);

  const values = a.items.map((ai) => b.items.map((bi) => ncd(c, ai.bytes, bi.bytes)));

  return { rows, cols, values };
}

export function formatMatrix(m: Matrix, format: "tsv" | "csv" = "tsv", labels = true): string {
  const sep = format === "csv" ? "," : "\t";
  const lines: string[] = [];

  if (labels) {
    lines.push(["", ...m.cols].join(sep));
    for (let r = 0; r < m.rows.length; r++) {
      lines.push([m.rows[r], ...m.values[r].map((v) => v.toString())].join(sep));
    }
  } else {
    for (let r = 0; r < m.rows.length; r++) {
      lines.push(m.values[r].map((v) => v.toString()).join(sep));
    }
  }

  return lines.join("\n") + "\n";
}
