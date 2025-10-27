### Workshop Walker – Developer Guidelines (project‑specific)

#### Build and configuration
- Toolchain
  - Rust 1.86+ (MSRV follows stable). The workspace sets `edition = "2024"` and enforces strict Clippy lints via `Cargo.toml`.
  - Target tuning: the root `Cargo.toml` sets ` [target.x86_64-unknown-linux-gnu] rustflags = "-C target-cpu=x86-64-v3"`.
    - On legacy CPUs, override at build time: `RUSTFLAGS="-C target-cpu=x86-64" cargo build`.
  - Release profile is optimized and small (LTO, strip, `panic = abort`, `codegen-units = 1`).
- Workspace and patched deps
  - This is a multi-crate workspace (`macros`, `serde-hack`, root binary). Some heavy ML crates (candle*) are in workspace deps but not required for day‑to‑day development paths.
  - `surrealdb` and `surrealdb-migrations` are patched to git repos/tags in `[patch.crates-io]`. Avoid unintentional upgrades: do not run `cargo update -p surrealdb*` unless you intend to move to a new version/fork.
- System packages occasionally needed
  - Building in the provided Dockerfile uses `libclang-dev` (used by some transitive deps) and Node for the UI pipeline. On a typical developer machine, a plain `cargo build` usually succeeds without Node.
- Local run prerequisites
  - Runtime config must exist at `config/config.toml`. The app reads it directly (see `src/main.rs`). Fields of note:
    - `[database] user/password` are used to create/sign in the root user on a local RocksDB store.
    - `[steam] api_token` and `appid` configure access to Steam Web API (not used during tests).
    - `[biscuit] public_key/private_key` control auth token verification/issuance.
  - Local database is an embedded SurrealDB RocksDB at `./workshopdb`. First run will create the store; migrations are applied automatically on startup via `surrealdb-migrations`.
- Building the binary
  - Debug: `cargo build`
  - Release: `cargo build --release`
  - Docker (multi‑stage): `docker build -t workshop-walker .`
    - Stage 1 builds Rust (installs `npm` and `libclang-dev` as build deps), Stage 2 builds the UI (`ui/`), final image is distroless and contains: binary at `/workshop-walker`, UI assets in `/ui/build`, `migrations/` and `schemas/` directories.
- Running locally
  - Ensure `config/config.toml` exists and contains valid values (a sample is committed already). Then run `cargo run`.
  - Web server binds `0.0.0.0:5800` (see `src/web/mod.rs`). Static UI is served from `ui/build/` under non‑API routes.

#### Testing
- Keep tests pure by default
  - Do not touch the DB, network, web server, or spawn actors from unit/integration tests unless explicitly opted in. The binary’s startup path: config load → RocksDB → root user → signin → migrations → actors → web server. None of this should run inside normal tests.
  - Prefer pure unit tests in modules with `#[cfg(test)] mod tests { .. }` and integration tests under `tests/` that do not require external services.
- Running tests
  - Standard: `cargo test`
  - With logs (recommended for debugging): `RUST_LOG=debug cargo test -- --nocapture`
- Adding new tests
  - Unit tests (inside a module):
    ```rust
    #[cfg(test)]
    mod tests {
        #[test]
        fn add_works() {
            assert_eq!(1 + 1, 2);
        }
    }
    ```
  - Integration tests (preferred for crate‑level behavior without side effects): create a file under `tests/`, e.g. `tests/smoke.rs`:
    ```rust
    #[test]
    fn smoke_test_arithmetic() {
        assert_eq!(2 + 2, 4);
    }
    ```
  - Executing: `cargo test`. These examples are designed to compile and pass without initializing async runtimes or database connections.
- End‑to‑end (optional) strategy
  - If you need HTTP or DB integration, gate such tests behind a feature flag and a separate CI job, e.g. run with `--features e2e`.
  - For DB, use a temp directory for RocksDB (avoid `./workshopdb`) and a disposable config. Do not start the actual server in unit tests; instead, test handlers with `salvo` test utilities or by factoring logic into pure functions.

#### Migrations and schema
- Migrations are applied at process start via `surrealdb_migrations::MigrationRunner::new(&db).up()`.
- Migration files are under `migrations/`; schema helpers and other `.surql` files live under `schemas/`.
- To add a migration:
  - Add a new file in `migrations/` following the existing conventions used by `surrealdb-migrations` (see that crate’s README for file naming and up/down semantics). Ensure it is idempotent or appropriately versioned.
  - Keep queries compatible with the embedded RocksDB engine used locally.

#### Code style and quality gates
- Formatting: `rustfmt` is configured in `rustfmt.toml` (imports grouping, max width 100, etc.). Run `cargo fmt --all`.
- Linting: Clippy lint groups are set in `Cargo.toml` with multiple `deny` groups (`suspicious`, `style`, `perf`, `pedantic`, `correctness`, etc.). Run:
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Error handling: The codebase uses `snafu::Whatever` for top‑level error aggregation, and a custom `web::Error` adapter for Salvo. Follow these patterns when extending APIs.
- Web/API:
  - The server is built with `salvo` and `oapi` docs; swagger is available under `/swagger-ui` with the spec at `/api-doc/openapi.json`.
  - Prefer adding routes to `src/web/mod.rs`, use hoops for auth (`auth::validate_biscuit_token`, `auth::enforce_admin`) and `affix_state::inject(config)` for config injection.

#### Practical tips specific to this repo
- Config values in `config/config.toml` are read at runtime; avoid baking secrets into Docker images for production. For local dev, the committed file is sufficient.
- The embedded DB directory `./workshopdb` is safe to delete between runs if you need a clean slate (migrations will recreate structure).
- UI assets are served from `ui/build/`; in development you can run the UI separately and point `base_url` in config to your dev server.
- When working on auth or admin flows, check the routes under `src/web/admin.rs` and `src/web/auth.rs` for expected payloads and token handling.

#### Simple test demo (for local verification)
- Integration test example to verify your toolchain and project setup:
  - Create `tests/smoke.rs` with:
    ```rust
    #[test]
    fn smoke_test_arithmetic() {
        assert_eq!(2 + 2, 4);
    }
    ```
  - Run: `cargo test`
  - Expected: the test should compile and pass quickly without starting the DB or server.

Notes
- Some transitive crates (e.g., tokenizers or candle) may require a working `libclang` on certain platforms; if a local build fails with clang/LLVM errors, install your distro’s `libclang-dev` and retry.
- If you target non‑x86 Linux or older CPUs, see the `rustflags` note above; you can also build inside the provided Dockerfile to normalize the environment.