import { stat, readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

export type InputItem = {
  label: string;
  bytes: Uint8Array;
};

export type InputSet = {
  name: string;
  items: InputItem[];
};

export type SetSpec =
  | { kind: "dir"; path: string }
  | { kind: "file"; path: string }
  | { kind: "list"; path: string }
  | { kind: "literal"; text: string; label?: string };

export async function autoDetectSetSpec(arg: string, opts?: { list?: boolean }): Promise<SetSpec> {
  if (opts?.list) return { kind: "list", path: arg };

  try {
    const st = await stat(arg);
    if (st.isDirectory()) return { kind: "dir", path: arg };
    if (st.isFile()) return { kind: "file", path: arg };
  } catch {
    // Not a path → treat as literal
  }
  return { kind: "literal", text: arg };
}

export async function loadSet(spec: SetSpec): Promise<InputSet> {
  switch (spec.kind) {
    case "dir": {
      const entries = (await readdir(spec.path, { withFileTypes: true }))
        .filter((e) => e.isFile())
        .map((e) => e.name)
        .sort();

      const items = await Promise.all(
        entries.map(async (name) => {
          const path = join(spec.path, name);
          const bytes = await readFile(path);
          return { label: name, bytes };
        }),
      );

      return { name: spec.path, items };
    }

    case "file": {
      const bytes = await readFile(spec.path);
      const label = spec.path.split("/").pop() ?? spec.path;
      return { name: spec.path, items: [{ label, bytes }] };
    }

    case "list": {
      const raw = await readFile(spec.path, "utf8");
      const paths = raw
        .split(/\r?\n/)
        .map((s) => s.trim())
        .filter((s) => s.length > 0 && !s.startsWith("#"));

      const items = await Promise.all(
        paths.map(async (p) => {
          const bytes = await readFile(p);
          const label = p.split("/").pop() ?? p;
          return { label, bytes };
        }),
      );

      return { name: spec.path, items };
    }

    case "literal": {
      const label = spec.label ?? "literal";
      const bytes = new TextEncoder().encode(spec.text);
      return { name: label, items: [{ label, bytes }] };
    }
  }
}
