# Workshop Walker

Workshop Walker is a better interface to the Steam Workshop focused initially on RimWorld mods. It aims to make
browsing and understanding workshop content easier with:

- Language filtering support
- Discovering dependants for a mod (reverse dependency lookups)
- Community-supported classification for existing mods

A live version may be available at: https://workshop-walker.disconsented.com/ (status subject to change).


## Overview

The project is a full-stack application:
- Backend: Rust 2024 edition using the Salvo web framework and SurrealDB (embedded RocksDB engine) for storage.
- Frontend: SvelteKit + Vite static site served by the backend in production.
- Data model: SurrealDB relations allow efficient reverse lookups for “dependants”.
- Language detection: heuristic using the `lingua` crate.


## Requirements

- Rust toolchain >= 1.86 (see `Dockerfile` base image) and Cargo
- Node.js >= 20 (Dockerfile uses node:23) and npm
- Build tools per platform (e.g., libclang, etc. are installed in the container build)
- Git

Optional, for containerized builds:
- Docker or compatible OCI tooling


## Project Stack

- Language: Rust (edition 2024)
- Web framework: `salvo`
- Async runtime: `tokio`
- Database: `surrealdb` using local `RocksDb` engine (embedded)
- Migrations: `surrealdb-migrations`
- Auth: Steam OpenID + Biscuit tokens
- UI: SvelteKit + Vite (static build served by backend)

Key crates (see `Cargo.toml`): `salvo`, `tokio`, `surrealdb`, `surrealdb-migrations`, `snafu`, `tracing`, `lingua`,
`ractor`, `biscuit-auth`, etc.


## Entry points

- Application binary main: `src/main.rs`
  - Starts the web server on `0.0.0.0:5800`
  - Initializes SurrealDB at `./workshopdb` and runs migrations
  - Serves API under `/api/*`
  - Serves static UI from `ui/build/` in production

- Web router: `src/web/mod.rs`
  - OpenAPI is exposed at `/api-doc/openapi.json` with Swagger UI at `/swagger-ui`


## Configuration

Application configuration is read from `config/config.toml`.
Important keys include:

- `base_url`: URL used by the UI (e.g., in dev `http://localhost:5173`)
- `[steam]`
  - `api_token`: your Steam Web API token
  - `appid`: initial game AppID (e.g., 294100 for RimWorld)
- `[database]`
  - `user` / `password`: credentials used to create/sign into SurrealDB root
- `[biscuit]`
  - `public_key` / `private_key`: keys used to sign/verify Biscuit tokens

Environment variables used:
- `RUST_LOG` controls logging level (e.g., `RUST_LOG=info`)

Port configuration:
- Server currently binds to `0.0.0.0:5800` in code. TODO: Make this configurable via config/env.

Secrets note:
- Do not commit real API tokens or private keys. Replace values in `config.toml` with your own for deployment.


## Setup (development)

1) Clone and prepare
```
git clone https://github.com/disconsented/workshop-walker.git
cd workshop-walker
```

2) Configure
- Copy and edit `config/config.toml` as needed (see Configuration).

3) Install frontend dependencies and run the dev server
```
cd ui
npm install
npm run dev
```
This starts Vite dev server (default `http://localhost:5173`).

4) Run the backend
```
# In project root
cargo run
```
The backend serves API on `http://localhost:5800`.

Note:
- The UI dev server and API are on different ports. Configure the UI to call the API accordingly.
  - TODO: Document the UI’s API base URL configuration and any CORS/proxy behavior if/when added.


## Building for production

Option A — Native builds
```
# Build the UI
cd ui
npm install
npm run build
cd ..

# Build the backend
cargo build --release

# Run the binary (it will serve static files from ui/build if present)
./target/release/workshop-walker
```

Option B — Docker
```
docker build -t workshop-walker .
docker run --rm -p 5800:5800 -v $(pwd)/workshopdb:/workshopdb workshop-walker
```
The Docker image:
- Builds Rust (release) and the UI
- Copies `migrations/` and `schemas/`
- Runs the binary exposing port 5800


## Database and migrations

- Local embedded database path: `./workshopdb` (created automatically)
- SurrealDB namespace/database: `workshop/workshop`
- Migrations are run automatically on startup via `surrealdb_migrations::MigrationRunner`.
- Migration and schema files are in `migrations/` and `schemas/`.

Data sources and references:
- Steam Web API is used for workshop details; see references below.


## API

- Base: `http://localhost:5800/api`
- OpenAPI: `http://localhost:5800/api-doc/openapi.json`
- Swagger UI: `http://localhost:5800/swagger-ui`

Example routes (see `src/web/mod.rs` and modules under `src/web/`):
- `GET /api/list`
- `GET /api/item/{id}`
- `POST /api/property` (Biscuit-protected)
- `POST /api/vote/property` and `DELETE /api/vote/property` (Biscuit-protected)
- `GET /api/admin/*` (admin-protected)


## Scripts

Rust workspace uses Cargo (no custom scripts). Useful commands:
- `cargo run` — run backend in debug
- `cargo build --release` — production build
- `cargo test` — run Rust tests

Frontend scripts (from `ui/package.json`):
- `npm run dev` — start Vite dev server
- `npm run build` — build static site
- `npm run preview` — preview static build
- `npm run check` — type-check and svelte-check
- `npm test` — run unit tests (Vitest)


## Tests

- Rust: `cargo test`
  - Note: minimal tests currently (e.g., `#[cfg(test)]` in `src/main.rs`).
  - TODO: Expand backend tests and add integration tests for API.

- Frontend: from `ui/`
```
npm test
```


## Project structure

Top-level directories and files:

- `src/` — Rust backend source
  - `main.rs` — entrypoint
  - `web/` — routes and HTTP handling (Salvo)
  - other modules: `actors/`, `db/`, `processing/`, `steam/`, etc.
- `ui/` — SvelteKit frontend
- `migrations/` and `schemas/` — SurrealDB migrations and schema definitions
- `config/config.toml` — application configuration
- `workshopdb/` — local SurrealDB RocksDB storage (created at runtime)
- `Dockerfile` — multi-stage build (Rust + UI) and runtime image
- `LICENSE.md` — license (MPL-2)
- `CHANGELOG.md` — changelog
- Additional folders: `classification/`, `serde-hack/`, `macros/`, `surrealdb-migrations/`, `tests/`, etc.


## Suggest a Game/Tag

Please put requests/suggestions for games & tags in the Discussions section:
https://github.com/disconsented/workshop-walker/discussions


## Contributing

Want to contribute a feature? Great!

Please open an issue to discuss first. Every contribution adds to maintenance burden; this process helps manage scope and
avoid burnout.

- Code style: project uses `rustfmt`, `clippy` (see `clippy.toml`), and common Rust idioms.
- Commits/CI: see `CHANGELOG.md`, `cliff.toml`, and `release-plz.toml` for release tooling.
- TODO: Document contribution guidelines and code of conduct in more detail.


## License

This project is licensed under the Mozilla Public License 2.0 (MPL-2). See `LICENSE.md`.


## References

- Steam Published File Service docs: https://partner.steamgames.com/doc/webapi/ipublishedfileservice
- Unofficial docs: https://steamapi.xpaw.me/#IPublishedFileService/GetDetails
- Another reference: https://steamwebapi.azurewebsites.net/


## TODOs / Open questions

- Make server bind address/port configurable (currently hardcoded `0.0.0.0:5800`).
- Document UI -> API base URL configuration and CORS/proxy behavior for dev workflow.
- Add more backend tests and API integration tests.
- Provide example `config/config.toml` template with placeholders for secrets.