# workshop-walker

A Rust service that indexes Steam Workshop items, classifies them, and serves
them through an HTTP API. The frontend is a SvelteKit application in `ui/`.

## Layout

The backend follows a hexagonal layout. Each feature has the same four parts:

- `src/domain/<name>.rs`: the port trait and the error enum. No I/O.
- `src/application/<name>_service.rs`: a service that holds a port and adds the
  use case logic.
- `src/db/<name>_repository.rs`: the SurrealDB adapter that implements the port.
- `src/db/<name>_actor.rs`: a ractor actor that wraps the service and gives the
  rest of the program a message interface.

`src/web` holds the Salvo handlers and the OpenAPI schema. `src/steam` and
`src/processing` hold the background actors that download and classify items.

Workspace crates: `classification` (candle, gated behind the
`ml-classification` feature), `macros` and `proc-macros` (the ID and dual
struct macros), `migrations-tool` (the SurrealDB migration runner), and
`serde-hack`.

## Rust

Edition 2024. Format with `cargo +nightly fmt --all`. Lint with
`cargo clippy --all`. The `pedantic`, `style`, `perf`, `correctness` and
`suspicious` groups are set to deny in `Cargo.toml`, so a clippy warning is a
build failure.

Errors use snafu. Each domain module declares its own `#[non_exhaustive]` error
enum, as in `TagError` in `src/domain/tags.rs`. `main.rs` uses `Whatever` with
`whatever_context` because it has no caller to match on the variant.

Secrets use veil. `Config.steam.api_token` in `src/app_config.rs` carries
`#[redact]`. A new secret field needs the same attribute, and no secret goes
into a log line or a response body.

IDs come in pairs from the `define_id!` macro in `macros/src/lib.rs`. The
internal type holds a `RecordId` and keeps the `I` prefix, such as `IAppID`.
The external type holds the plain value that goes over the wire, such as
`AppID`. An internal ID stays inside `src/db` and `src/application`. A web
handler that returns an internal ID leaks the table name, so report it.

`surrealdb_types::RecordId::new` is a disallowed method in `clippy.toml`. Build
an ID with the newtype instead.

`unwrap` and `expect` are allowed in tests, and `clippy.toml` allows them there.
In a request path or an actor message loop, return an error instead. Do not
report the existing calls; report a new one.

Prefer a crate the workspace already depends on. A new dependency needs a reason
in the pull request body. Where two crates do the same job, avoid the one from
dtolnay.

Do not write the same fully qualified path more than once in a file. Import it.

## Database

SurrealDB 3.1. Use `type::record`, not the `type::thing` of version 2.

Migrations live in `migrations/`, named `<unix milliseconds>_<snake_case>.surql`,
and run up only. A migration that reached master is immutable; a correction is a
new file. `migrations-tool` checksums each file, so an edit to an applied
migration breaks the runner at startup.

## Frontend

SvelteKit 2 with Svelte 5, Skeleton 5, and Tailwind 4.

Shared state uses runes in a `.svelte.ts` module, as in
`ui/src/routes/app/[id]/store.svelte.ts`. The codebase imports nothing from
`svelte/store`; keep it that way.

Format with `npm run format`, lint with `npm run lint`, and typecheck with
`npm run check`, all from `ui/`.

Four components render item text with `{@html}`, such as
`ui/src/routes/app/[id]/itemCard.svelte`. That text comes from Steam and passes
through `BBActor` in `src/processing/bb_actor.rs` first. Flag a change that adds
a new `{@html}` on a value that does not pass through `BBActor`.

## Commit messages

Conventional Commits, as `contributing.md` describes. release-plz reads them to
build the changelog, so the type decides the version bump.

## Review notes

Do not report these:

- Formatting. `cargo fmt` and prettier already decide it.
- Generated output: `ui/src/lib/paraglide/`, `ui/src/paraglide/`,
  `ui/.svelte-kit/`, `ui/build/`, and any lock file.
- A missing test for a UI component. The frontend has no component tests yet.
