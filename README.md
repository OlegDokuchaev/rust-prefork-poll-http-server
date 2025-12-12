Rust Prefork + Poll HTTP Server
================================

Minimal static HTTP server showcasing a prefork model with `poll()`-based multiplexing. No frameworks, only standard library + `nix`.

Features
--------
- Prefork master/worker model; each worker handles multiple sockets via `poll()`.
- `GET` and `HEAD` with correct `Content-Length`; 405 for other methods.
- HTML served from a configured file path (no built-in fallback).
- Non-blocking sockets end-to-end.
- Make targets for fmt/clippy; GitHub Actions CI for linting.

Architecture (high level)
-------------------------
- `src/main.rs` — load settings, init HTML, bind listener, call `server::run`.
- `src/config.rs` — `Settings` loaded from env (`SERVER__*`).
- `src/page.rs` — one-time HTML load into memory.
- `src/server.rs` — prefork + child supervision.
- `src/worker.rs` — `poll()` loop, accepts and dispatches to connections.
- `src/conn.rs` — per-connection read/parse/write state.
- `src/handler.rs`, `src/http.rs` — request parsing and HTTP responses.

Configuration (env vars, required)
----------------------------------
```
SERVER__ADDR=0.0.0.0:8080
SERVER__WORKERS=4
SERVER__POLL_TIMEOUT_MS=1000
SERVER__READ_CHUNK=4096
SERVER__HTML_PATH=public/index.html
```

Running locally
---------------
```bash
# optional: prepare .env
cat > .env <<'EOF'
SERVER__ADDR=0.0.0.0:8080
SERVER__WORKERS=4
SERVER__POLL_TIMEOUT_MS=1000
SERVER__READ_CHUNK=4096
SERVER__HTML_PATH=public/index.html
EOF

make run
```

Make targets
------------
- `make fmt-check` — `cargo fmt --check`
- `make clippy` — `cargo clippy --all-targets --all-features -- -D warnings`
- `make run` — loads `$(ENV_FILE)` (default `.env`) then `cargo run`

CI
--
`.github/workflows/ci.yml` runs fmt-check and clippy on pushes/PRs (master).

Notes / limits
--------------
- HTML must exist at `SERVER__HTML_PATH`; startup fails otherwise.
- Only `GET`/`HEAD`; no range/TLS/directory listing.
- Focused on demonstrating low-level networking (prefork + poll).***

