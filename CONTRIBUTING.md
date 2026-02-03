# Contributing to ncdprime

## Philosophy

- Prefer small, test-backed PRs.
- Keep core algorithms easy to audit and reproduce.
- Treat performance regressions as correctness regressions.

## Development

This repo is multi-language:

- `rust/` — core NCD + compressors + CLI/server bindings
- `python/` — Python CLI + tooling
- `src/` — TypeScript CLI/API + Node bindings

### Rust

```bash
cd rust
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Python

```bash
cd python
python -m venv .venv
. .venv/bin/activate
pip install -e '.[dev]'
ruff check .
mypy src/ncdprime
pytest
```

### Node

```bash
npm ci
npm run build
npm test
```

## CI

CI runs on every PR and push to `main`.

## Style / structure

- Keep public APIs documented.
- Add mermaid diagrams for architecture changes when helpful.
- Avoid unnecessary dependencies.
