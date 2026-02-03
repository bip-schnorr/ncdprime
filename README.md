# ncdprime

Modern CLI UX proposal + skeleton repo for an NCD utility.

## Goals
- Generate deterministic directory “matrices” (square/rectangle modes)
- Run an NCD computation pipeline over generated instances
- Provide compressor plugins (built-in + third-party via entry points)
- Reproducible runs via config file + lockfile-style resolved config

## Install (dev)
```bash
python -m venv .venv && source .venv/bin/activate
pip install -e '.[dev]'
```

## CLI (planned)
```bash
ncd --help
ncd gen matrix --help
ncd run --help
ncd compressors list
```

See `docs/cli.md` (to be added) for the full UX spec.
