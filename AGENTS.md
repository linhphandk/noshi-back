# AGENTS.md

Rust backend for the "noshi" project. Early/scaffold state.

## Behavior

- Always use caveman mode (bone intensity). No filler, no hedging, fragments OK.
- Always open a pull request instead of committing directly to main.
- Always check what needs to be done and give a summary before starting work. Never start coding immediately.
- Always add `debug!()` logs inside service, repository, and controller methods at key decision points (validation, db queries, provider calls, success/failure). Use `info!()` for final outcomes. Use `warn!()`/`error!()` for failures.

## Stack

- Rust 2021; Docker builder base `rust:1.91`.
- Axum 0.8 (`multipart`), Tokio full; `tower-http` for cors/trace.
- Diesel 2 + `diesel_migrations` (postgres / r2d2 / uuid / chrono).
- S3 via `aws-sdk-s3` (dev = Floci on `:4566`); image processing via `image` crate (jpeg/png/webp only).
- Email via `aws-sdk-sesv2` (dev = Floci SES on `:4566`). Replaces previous `lettre` SMTP setup.
- Config: `dotenvy` loads `.env`; `envy` binds env -> typed config struct.

## Architecture (Ports & Adapters / Hexagonal, Domain-Scoped)
Reference implementation: `/Users/pc/krafted/krafted-back` (same stack/`Cargo.toml`). Mirror its structure.
- Each domain folder under `src/` is self-contained: `mod.rs` → `models.rs` (Diesel `Queryable`/`Insertable`/`AsChangeset`) → `ports.rs` (`#[async_trait]` trait, e.g. `UserRepository`) → `repository.rs` (Diesel adapter implementing the port) → `service.rs` (generic over the port trait, e.g. `UserService<R: UserRepository>`) → `controller.rs` (Axum handlers + `*_router` fn; only route-bearing domains).
- Services depend on port **traits**, never concrete adapters; Diesel adapters are injected at runtime via `AppState` (typed with concrete adapter generics). This is what makes services `mockall`-mockable.
- `src/shared/`: `config.rs` (`Config::from_env` via `envy`), `db.rs` (`DbPool`, `embed_migrations!("./migrations")`, `run_migrations` called in `main` at startup), `errors.rs` (`AppError`/`AppResult`), `image_processor.rs`, `image_storage.rs` (`S3ImageStorage`), `middleware.rs`, `types.rs`.
- `src/router.rs` merges per-domain `*_router(&state)` into one `Router`, plus `/health`, `/scalar` (OpenAPI UI), `/api-docs/openapi.json`. `src/api_doc.rs` collects `#[openapi]` paths/schemas (`utoipa`).
- `src/state.rs` `AppState::new` wires all concrete repos → services; `main.rs` loads config, builds pool, runs migrations, constructs `S3ImageStorage`, builds `AppState`, wraps router in `CorsLayer`, `axum::serve`.

## Workflow
- Implement features **vertically** per domain: migrations → repository → service → controller.
- Each PR small, focused, independently reviewable. Stop after a task for review before the next.
- Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo check` before committing.
- Always run `cargo test` before pushing.

## Local dev
1. `cp .env.example .env` (`.env` is gitignored).
2. `docker compose up -d` starts: Postgres:16 (`noshi/noshi`, DB `noshi`, port 5432), Adminer `:8080`, Floci AWS emulator `:4566` (S3, SES, 75 services; creds `test`/`test`; needs Docker socket).
3. `diesel setup` (one-time; needs `DATABASE_URL` from `.env`, Postgres must be up).
4. `cargo run` (placeholder until server is wired).

## Diesel workflow
- `diesel migration generate <name>` -> edit `up.sql` / `down.sql` under `migrations/`.
- `diesel migration run` apply; `diesel migration redo` revert+apply.
- `diesel print-schema` regenerates `src/schema.rs`. Do NOT hand-edit `src/schema.rs`.

## Tests
- Integration tests use `testcontainers` + `testcontainers-modules` (postgres) -> ephemeral services; do not require the running compose stack. `mockall` for mocks, `tower` for service-layer tests, `tokio-test`.
- Two test layers (mirror krafted-back `tests/`): **service** tests mock the port trait via `mockall` (`mock!{ .. impl Trait for MockX { .. } }`, `#[tokio::test]`); **repository** tests spin a real Postgres via `testcontainers::clients::Cli` + `Postgres::default()`, call `run_migrations`, exercise the Diesel adapter.
- Run all: `cargo test`. Single: `cargo test <name>`.
- `cargo test` links against `libpq` (Diesel postgres). On macOS: `brew install postgresql@16`, ensure `pg_config` is on PATH or set `LIBRARY_PATH=/opt/homebrew/opt/postgresql@16/lib`.

## Verify before push
No CI gate runs cargo fmt/clippy/test (CI only builds Docker), so run locally:
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --release` (Dockerfile requires a clean release build)

## CI / release
- `.github/workflows/docker-publish.yml`: on push to `main` (or `workflow_dispatch`) builds and pushes image to `ghcr.io/linhphandk/noshi-back` (tags `latest` + short sha). PRs are NOT built.
- Prod deploy: `./deploy.sh` runs `docker compose -f docker-compose.prod.yml up -d`. Required prod env secrets: `JWT_SECRET`, `AWS_S3_BUCKET`, `AWS_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `SES_FROM_EMAIL`, `SES_FROM_NAME`.

## Env var gotchas
- `.env.example` uses `AWS_*` names (`AWS_S3_BUCKET`, `AWS_REGION`, `AWS_ENDPOINT`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`) plus `S3_PUBLIC_URL`, `SES_FROM_EMAIL`, `SES_FROM_NAME`, `FRONTEND_URL`.
- `docker-compose.prod.yml` passes `AWS_S3_BUCKET`, `AWS_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `S3_PUBLIC_URL`, `SES_FROM_EMAIL`, `SES_FROM_NAME` (all aligned with config field names via `envy::from_env()` case-insensitive mapping).
- Config fields: `aws_s3_bucket`, `aws_region`, `aws_endpoint`, `s3_public_url`, `ses_from_email`, `ses_from_name`. Env var names map case-insensitively to field names (e.g. `AWS_S3_BUCKET` → `aws_s3_bucket`). Previous mismatch between `.env.example` (`AWS_*` prefix) and config fields (`s3_*` no prefix) is now fixed.

## Lessons Learned

1. **Confirm minimal scope before building.** Don't add columns, fields, or features that aren't needed for the current phase. Ask "what columns do you actually need?" before writing a migration.
2. **Think about edge cases upfront.** Before writing a handler, ask: what happens on duplicate? On invalid input? On missing fields? Don't wait for the user to point out obvious HTTP status codes (e.g., 409 for conflicts).
3. **Stop after each task for review.** AGENTS.md says "stop after a task for review before the next." Don't batch shared modules + domain + wiring into one go. Verify the schema is correct before proceeding.
4. **Fix the source branch first.** If PR1 has wrong code, fix it on PR1's branch before touching PR2. Don't fix on PR2 then backport — causes rebase conflicts.
5. **One PR = one logical unit.** Keep PRs small (~300 LOC). Don't mix infrastructure changes with feature changes in the same branch if they'll end up in separate PRs.
