# tcl-live

Live map of TCL (Lyon public transit) buses and metros, rendered with Bevy in
the browser. The backend (axum) fetches and processes the Grand Lyon open-data
feed and serves the wasm frontend plus two JSON endpoints.

```
back/      axum server, host target (x86_64-unknown-linux-gnu)
front/     Bevy app, wasm32-unknown-unknown, exposed via wasm-bindgen
static/    index.html + pkg/ (wasm-bindgen output, written by back's build.rs)
```

## Requirements

- Rust toolchain (stable). `rustup target add wasm32-unknown-unknown`.
- `wasm-pack` (only used for `--release` builds): `cargo install wasm-pack`.
- `mold` linker (used for the host `back` build, see `.cargo/config.toml`):
  install via your package manager (e.g. `sudo apt install mold`). Remove the
  `link-arg=-fuse-ld=mold` line from `.cargo/config.toml` if you don't want it.
- A `.env` file at the workspace root with TCL open-data credentials:

  ```env
  USER=<grandlyon-data-username>
  PASSWORD=<grandlyon-data-password>
  PORT=3000
  ```

  `USER` and `PASSWORD` authenticate against `data.grandlyon.com`. `PORT` is
  optional, defaults to `3000`.

The first `cargo run` will additionally download `wasm-bindgen-cli` (matching
the crate version) into `~/.cache/.wasm-pack/`. The build script reuses that
binary on subsequent runs; no separate `cargo install wasm-bindgen-cli` needed.

## Run

```bash
cargo run
```

This compiles `back`, runs `back/build.rs` which compiles `front` to wasm and
post-processes it with `wasm-bindgen`, then starts the server on
`http://localhost:3000`.

To iterate on the backend without rebuilding the wasm front each time:

```bash
SKIP_WASM_BUILD=1 cargo run
```

The build script will skip the wasm step and reuse whatever is currently in
`static/pkg/`.

For a release build (uses `wasm-pack` so `wasm-opt` size optimization runs):

```bash
cargo run --release
```

## Build architecture

Three target directories are kept fully isolated to avoid cargo lock
contention (a previous setup deadlocked because two cargo invocations fought
for `target/debug/.cargo-lock`):

| Dir            | Used by                                              |
|----------------|------------------------------------------------------|
| `target/`      | `cargo run`/`cargo build` for `back` (host)          |
| `target-front/`| Inner `cargo build` for `front` wasm, from build.rs  |
| `target-ra/`   | rust-analyzer's background `cargo check` (see below) |

### Why `target-front/` exists

Calling `cargo build` from within a build script of the same workspace
deadlocks if both invocations target the same `target/` dir. The build script
sets `CARGO_TARGET_DIR=target-front` for its inner cargo invocation so the
two never share a lockfile.

### rust-analyzer config

`rust-analyzer.toml` at the workspace root pins rust-analyzer to
`target-ra/`:

```toml
[cargo]
targetDir = "target-ra"

[check]
targetDir = "target-ra"
```

This is editor-agnostic — Neovim, VS Code, Helix, JetBrains all pick it up
automatically.

### Profiles

- `dev` (default): `opt-level = 1` for workspace, `3` for deps.
- `wasm-dev` (used by build.rs for the front in dev): inherits `dev`, but
  `debug = "line-tables-only"` and `debug = false` for deps. This drastically
  reduces the size of the input to `wasm-bindgen` (which is the bottleneck of
  the inner-loop dev build), while keeping line numbers in stack traces.
- `release` / `wasm-release`: production builds. `wasm-release` strips
  debuginfo and optimizes for size.

## Iteration speed

Typical wall times on this codebase (Bevy + a handful of front modules):

| Scenario                                | Time     |
|-----------------------------------------|----------|
| Cold full build                         | ~3 min   |
| Warm rebuild, no source changes         | < 0.5 s  |
| Backend-only change                     | ~1-3 s   |
| Single front-source change (with bindgen)| ~10 s    |
| `SKIP_WASM_BUILD=1`, backend-only change | < 1 s    |

The build script short-circuits the `wasm-bindgen` step if its output is
already fresher than the inner `cargo build`'s wasm artifact.

## Endpoints

- `GET /` — returns `static/index.html` (the page that bootstraps the wasm).
- `GET /pkg/*` — serves `static/pkg/` (`front.js`, `front_bg.wasm`, ...).
  The path is resolved relative to the `back/` source dir (compile-time), so
  `cargo run` works from any cwd in the workspace. Override at runtime with
  `STATIC_PKG_DIR`.
- `GET /api/lignes` — list of TCL lines with their stops.
- `GET /api/positions` — current vehicle positions, computed from the open
  feed.
