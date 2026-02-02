# ncdprime

Modern NCD (Normalized Compression Distance).

## Install / build

```bash
npm install
npm run build
```

## CLI

Build the TypeScript CLI and run:

```bash
npm run build
node dist/cli.js pair <fileA> <fileB> --gzip-level 6
node dist/cli.js matrix <setA> [setB] --format tsv
```

Notes:
- gzip output is deterministic by default (fixed gzip header fields)
- the CLI will use the native Rust backend when available, otherwise it falls back to a pure-JS implementation

## Library

Default import is pure-JS (and will automatically use native if available):

```js
import { ncdAuto, matrixAuto } from "ncdprime";

const enc = (s) => new TextEncoder().encode(s);

console.log(ncdAuto(enc("aaaa"), enc("aaaa"), { gzipLevel: 6 }));
console.log(await matrixAuto([enc("aaa"), enc("bbb")], [enc("aaa")], { gzipLevel: 6 }));
```

## Native backend (optional)

To build the Rust-powered Node backend:

```bash
npm run build:native
```

Then the CLI and `ncdAuto`/`matrixAuto` will prefer it automatically.

If you want to import the native bindings directly (and accept that it may not exist unless built), use:

```js
import * as native from "ncdprime/native";
// native.ncd(...)
// native.matrix(...)
```
