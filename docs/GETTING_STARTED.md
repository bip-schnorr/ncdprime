# Getting started

This repo is multi-language and ships multiple entry points:

- **Rust** (`rust/`): core library + CLI + server + language bindings
- **Node/TS** (`src/`): TypeScript API + CLI + N-API binding loader
- **Python** (`python/`): Python CLI + tooling

## Quickstart (recommended)

### 1) Node (TypeScript) CLI

```bash
npm ci
npm run build
ncdprime --help
```

Run a tiny NCD example (two byte strings):

```bash
node -e "import('./dist/index.js').then(m=>console.log(m.ncd(Buffer.from('aaaa'), Buffer.from('aaab'), 6)))"
```

### 2) Rust workspace

```bash
cd rust
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Build the Rust CLI:

```bash
cargo build -p ncdprime-cli
./target/debug/ncdprime-cli --help

List available compressors:

```bash
./target/debug/ncdprime-cli compressors
```

Run a pairwise NCD (choose compressor + parameters):

```bash
./target/debug/ncdprime-cli pair a.txt b.txt --compressor zstd --zstd-level 3
```

Compute a matrix (progress/ETA prints to stderr; TSV/CSV on stdout):

```bash
./target/debug/ncdprime-cli matrix ./dirA ./dirB --format tsv > out.tsv
```
```

### 3) Python package

```bash
cd python
python -m venv .venv
. .venv/bin/activate
pip install -e '.[dev]'
ruff check .
mypy src/ncdprime
pytest

ncd --help
```

## Repo layout diagram

```mermaid
flowchart TB
  subgraph TS[TypeScript/Node]
    TS1[src/ (TS library + CLI)]
    TS2[index.js loaders]
  end

  subgraph R[Rust workspace]
    R1[rust/ncdprime-core]
    R2[rust/ncdprime-cli]
    R3[rust/ncdprime-server]
    R4[rust/ncdprime-node (N-API)]
    R5[rust/ncdprime-py]
  end

  subgraph PY[Python]
    P1[python/src/ncdprime]
  end

  TS1 --> TS2
  TS2 --> R4
  P1 --> R5
  R2 --> R1
  R3 --> R1
  R4 --> R1
  R5 --> R1
```

## CI

CI runs on PRs and pushes to `main`:
- Python: ruff + mypy + pytest
- Node: npm ci + build + vitest
- Rust: fmt + clippy (deny warnings) + tests

See `.github/workflows/ci.yml`.
