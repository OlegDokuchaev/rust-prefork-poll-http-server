Rust Prefork + Poll HTTP Server
================================

Minimal static HTTP server showcasing a prefork model with `poll()`-based multiplexing. No frameworks, only standard library + `nix`.

Features
--------
- Prefork master/worker model; each worker handles multiple sockets via `poll()`.
- `GET`/`HEAD` with correct `Content-Length`; 405 for other methods.
- Static files served from configurable `doc_root`; 403 for forbidden paths, 404 for missing files.
- No built-in fallback HTML: place your own `index.html` under `doc_root`.
- Non-blocking sockets end-to-end.
- Make targets for fmt/clippy; GitHub Actions CI for linting.

Architecture (high level)
-------------------------
- `src/main.rs` — load settings, bind listener, call `server::run`.
- `src/config.rs` — `Settings` loaded from env (`SERVER__*`).
- `src/server.rs` — prefork + child supervision.
- `src/worker.rs` — `poll()` loop, accepts and dispatches to connections.
- `src/conn.rs` — per-connection read/parse/write state.
- `src/static_files.rs` — secure path resolution + file read (GET/HEAD).
- `src/handler.rs`, `src/http.rs` — request parsing and HTTP responses.

Configuration (env vars, required)
----------------------------------
```
SERVER__ADDR=0.0.0.0:8080
SERVER__WORKERS=4
SERVER__POLL_TIMEOUT_MS=1000
SERVER__READ_CHUNK=4096
SERVER__DOC_ROOT=public
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
SERVER__DOC_ROOT=public
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
- Put your files under `SERVER__DOC_ROOT`; 403 for attempts to escape root or open non-files, 404 if missing.
- HEAD returns no body, only headers with correct `Content-Length`.
- Only `GET`/`HEAD`; no range/TLS/directory listing.
- Focused on demonstrating low-level networking (prefork + poll).

