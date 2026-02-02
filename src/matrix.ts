import type { Compressor } from "./ncd.js";
import { ncd } from "./ncd.js";
import type { InputSet } from "./inputs.js";

export type Matrix = {
  rows: string[];
  cols: string[];
  values: number[][]; // rows x cols
};

const sha256 = async (bytes: Uint8Array): Promise<string> => {
  // Used only for deduping singleton compression sizes in the pure-JS fallback.
  const { createHash } = await import("node:crypto");
  return createHash("sha256").update(bytes).digest("hex");
};

export async function computeMatrix(c: Compressor, a: InputSet, b: InputSet): Promise<Matrix> {
  const rows = a.items.map((i) => i.label);
  const cols = b.items.map((i) => i.label);

  // Cache C(x) by content hash to avoid recompressing duplicates.
  const sizeCache = new Map<string, number>();
  const getSize = async (bytes: Uint8Array): Promise<number> => {
    const key = await sha256(bytes);
    const hit = sizeCache.get(key);
    if (hit != null) return hit;
    const v = c.compress(bytes).byteLength;
    sizeCache.set(key, v);
    return v;
  };

  const aSizes = await Promise.all(a.items.map((i) => getSize(i.bytes)));
  const bSizes = await Promise.all(b.items.map((i) => getSize(i.bytes)));

  const values: number[][] = [];
  for (let i = 0; i < a.items.length; i++) {
    const row: number[] = [];
    for (let j = 0; j < b.items.length; j++) {
      // We still call ncd() for the per-pair terms; it will recompute C(x)/C(y).
      // That's acceptable for the JS fallback, but we keep the singleton cache
      // ready in case we later expose an ncd_from_sizes equivalent here.
      //
      // For now, use scalar ncd (which is now symmetric + frame64 join).
      row.push(ncd(c, a.items[i].bytes, b.items[j].bytes));

      // (Optional future optimization: implement an ncdFromSizes in JS and reuse aSizes/bSizes.)
      void aSizes[i];
      void bSizes[j];
    }
    values.push(row);
  }

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
