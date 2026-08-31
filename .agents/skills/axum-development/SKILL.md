---
name: axum-development
description: Build, refactor, debug, test, or review Curio's Rust/Axum backend in curio-service/. Use for routers, handlers, domain services, SQLx repositories, PostgreSQL migrations, OpenAI streaming, configuration, auth middleware, database tools, and service tests. Do not use as the primary guide for the React client or Tauri desktop bridge.
---

# Curio Axum development

Fit changes into the service that exists. Before editing, read
[`AGENTS.md`](../../../AGENTS.md), the Axum and persistence sections of the
canonical [`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md),
`curio-service/Cargo.toml`, the relevant module, its tests, and the current
migration. Inspect `git status` and preserve unrelated changes.

If an HTTP/SSE contract is new or changes, also read and apply
`../react-vite-development/SKILL.md` and `../code-verification/SKILL.md` so the
TypeScript consumer and cross-stack checks are designed with the Rust boundary.

## Understand the composition roots

- `src/main.rs` loads `.env`, parses `ServiceConfig`, builds the production router, binds the configured address, and serves it.
- `app_with_config` opens and verifies the PostgreSQL database before composing the application.
- `app_with_database` is the testable full-router composition root for configured feature routes.
- `app()` returns only the small base router. Do not use it as evidence that database-backed or chat routes are present.
- Feature routers are merged under one exact-origin CORS layer. CORS is a browser policy, not authorization.

Add routes at the feature router and merge new top-level feature routers at the full composition root. Apply middleware at the narrowest router that owns the policy. Keep extractors and state types compatible across merged routers.

## Prefer feature-local vertical slices

Calendar is the reference organization for new database-backed behavior:

```text
router
  -> handler: extraction, identity/ownership, HTTP status and envelope
  -> service: validation, defaults, business rules, error classification
  -> repository: parameterized SQL and typed row mapping
  -> Database: pool and migration lifecycle only
```

Use this as a responsibility guide, not a reason to create empty layers. Keep feature SQL in feature repositories. Keep transport DTOs out of the central database implementation. Reuse one cheaply cloned `Database`/`PgPool`; do not open pools per request.

Handlers should be thin and explicit about path/query/body/state/extension extraction. Services should return stable domain errors. Repositories should use bound parameters and typed records. Map internal errors into sanitized client responses at a deliberate boundary and log only non-sensitive operational context.

## Preserve database lifecycle and privilege separation

PostgreSQL is the active runtime store, with version 18 as the current CI and
deployment target rather than a version enforced by connection code.
`Database::connect` validates a conservative schema name, configures UTC and a
schema-scoped search path on every connection, and opens a bounded pool without
running DDL.

Keep these invariants:

- When the schema changes, add an append-only SQLx migration under
  `curio-service/migrations/postgres`; do not create a no-op migration for an
  HTTP-only change or rewrite a migration that may have been applied.
- For a real schema change, update relevant `ops/postgres/verify.sql` catalog or
  version expectations and assess importer compatibility/tests. Do not edit
  archived SQLite migrations merely because an HTTP route changed.
- Run migrations explicitly with the schema-owner credential through `cargo run --bin curio_db -- migrate` before a release.
- The service uses the lower-privilege `DATABASE_URL` and verifies the expected migration at startup.
- `CURIO_DB_SCHEMA` selects an explicit development, production, or disposable test schema; never interpolate an unvalidated identifier.
- Use PostgreSQL `$n` parameters and bind all values.
- Use `INSERT/UPDATE/DELETE ... RETURNING` and deliberate transaction boundaries
  when a response or state transition depends on the committed row.
- Do not expose or log database URLs, credentials, provider keys, raw sensitive payloads, or unrestricted upstream error bodies.

SQLite is archival/import-only. The default dependency graph must remain PostgreSQL-only. `migrations/sqlite` exists for audit/import compatibility, and the `sqlite-import` feature is confined to the one-time importer and its test. Do not introduce SQLite into request paths or default features.

## Authentication and ownership are incomplete

The current bearer middleware is a development placeholder: the nonblank bearer string becomes `CurrentUser.id`; it has no signature, expiry, issuer, or session validation. Client route guards are also not authorization.

- Do not describe the service as production-authenticated.
- Calendar must continue to compare bearer-derived identity with `userId`.
- User upsert currently does not enforce body ID equals bearer ID.
- Chat stream/history routes are currently public, and conversations have no user owner.
- CORS does not repair missing authentication or ownership.

Any production-auth change is cross-stack and data-model work: define token verification, ownership, migrations, route coverage, client token flow, and tests together. Do not silently broaden or narrow existing access while implementing an unrelated feature.

## Maintain public contracts

Use Serde naming and the established JSON envelopes consistently. Add stable
response/error types where a feature owns them; do not leak SQLx, Reqwest,
OpenAI, or internal enum details over HTTP. A route or wire change must update
the matching TypeScript resource/protocol type and both service and frontend
contract tests in the same change. Remember that Axum extractor failures, auth
failures, feature errors, history errors, and SSE terminal errors currently
have different shapes; avoid claiming there is one global error schema until
one is implemented.

For chat:

- Persist the conversation, completed user message, and pending assistant row before calling OpenAI.
- Parse provider SSE incrementally across arbitrary LF/CRLF and byte boundaries.
- Normalize provider events to Curio `token`, `done`, and `error` events.
- Persist completed or failed assistant state before emitting its terminal event.
- On disconnect, serialization failure, or unexpected upstream end, make the best effort to mark the assistant interrupted.
- Keep upstream bodies and sensitive details out of client errors.
- Preserve request/event identifier correlation expected by the shared TypeScript parser.

For calendar:

- Bare `YYYY-MM-DD` values are all-day dates; RFC 3339 values with offsets are timed.
- Start and end must use the same category.
- All-day ends are exclusive; a missing all-day end becomes the next day internally.
- A missing timed end is publicly a milestone but receives a small internal half-open interval for overlap queries.
- Range overlap remains `starts_at < requested_end AND ends_at > requested_start`.
- Preserve bounded view windows and DST slack.
- Return trimmed, validated public date strings; do not expose normalized
  `starts_at`/`ends_at` fields.
- For get/update/delete, treat bearer-derived identity as authoritative and
  scope repository SQL by both resource ID and owner ID. Never authorize only
  by comparing the bearer with a client-supplied `userId`; that permits an IDOR
  when both forged values match. Choose and test a consistent non-enumerating
  `404`/`403` policy.
- Define PATCH omitted-versus-explicit-null semantics in the request type before
  implementation. If `allDay`, `startDate`, or `endDate` changes, merge with the
  stored record, re-run the complete cross-field temporal validation, and
  recompute `starts_at`/`ends_at` in the same owner-scoped database operation or
  transaction.
- For a member PATCH, declare the editable allowlist and immutable fields, empty
  patch behavior, returned status/body, and last-write-wins or optimistic
  concurrency policy. Update `updated_at` without changing `created_at`, return
  the committed record, and test omitted, explicit-null, invalid, missing, and
  foreign-owner cases plus movement between query ranges.

## Configuration and operational behavior

Keep environment parsing centralized and bounded. `ServiceConfig` debug output must redact secrets. Provider credentials belong only in the service. Browser access uses exact configured origins; desktop requests originate in Rust and do not require CORS.

The current server has limited cross-cutting operational middleware. If a task adds tracing, request IDs, graceful shutdown, timeouts, body limits, or rate limiting, specify how it interacts with long-lived SSE and readiness before applying it globally.

`/health` is process liveness. `/ready` checks database access and migration state. Preserve that distinction for deploy probes.

## Test through the appropriate boundary

Keep pure validation/time/protocol behavior in unit tests. Use full-router tests with `tower::ServiceExt::oneshot` for extraction, middleware, status, envelopes, and ownership. Use the PostgreSQL fixture for SQL, constraints, transactions, reconnect durability, migrations, and range behavior. Mock OpenAI with a local HTTP server while exercising the real Curio router and stream.

Integration tests require `CURIO_TEST_DATABASE_URL` for a non-production test-runner role. The fixture must create and drop only disposable `curio_test_*` schemas. Never point automated tests at development or production schemas.

Run from `curio-service`:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features sqlite-import --bin import_sqlite
cargo test --features sqlite-import --test sqlite_import
```

Start with the narrowest relevant test target. Run both importer targets when
feature flags, database dependencies, migrations, import code, or archived
SQLite compatibility changes. When configuration, routing, deployment, or
migrations change, also run the relevant `curio_db verify` or local readiness
check against an explicitly safe database.

Report exact checks, prerequisites, results, skipped coverage, and remaining uncertainty. Never interpret a skipped database test as a pass.

Before handoff, compare the service diff with `docs/ARCHITECTURE.md`. Update
that canonical document in the same change when a feature or fix changes routes,
layering, wire/SSE behavior, persistence, migrations, configuration, auth or
trust boundaries, operations, limitations, or verification. If no claim
changes, report `Architecture impact: none` rather than editing prose only for
ceremony.
