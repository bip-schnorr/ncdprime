# ncdprime-server

Minimal HTTP server for ncdprime.

## Run

```bash
cd rust
source "$HOME/.cargo/env"

# bind address can be overridden
export NCDPRIME_BIND=127.0.0.1:8787

cargo run -p ncdprime-server
```

## Endpoints

### GET /health

Returns `ok`.

### POST /ncd/pair

Request JSON:

```json
{
  "a_b64": "...",
  "b_b64": "...",
  "gzip_level": 9
}
```

Response:

```json
{ "ncd": 0.123 }
```

### POST /ncd/matrix

Request JSON:

```json
{
  "a": ["...base64...", "..."],
  "b": ["...base64..."],
  "gzip_level": 9
}
```

Response:

```json
{ "values": [[0.1], [0.2]] }
```

Notes:
- gzip output is deterministic by default (header mtime fixed to 0).
- join strategy and symmetry match the Rust core defaults.
