# Curio agent guide

This file is the repository-wide operating guide for humans and coding agents. It describes how to make changes safely; `docs/ARCHITECTURE.md` is the canonical description of how the system works.

## Instruction order

Follow, in order:

1. The current user request.
2. This repository guide.
3. The relevant project skill under `.agents/skills`.
4. The established source, test, and package conventions nearest the change.

`.claude/settings.json` enables the `actionbook/rust-skills` plugin, which
supplies general Rust guidance. It sits below this guide and the project skills
in that order: where it disagrees with either, they win.

If instructions and code disagree, investigate before editing. Prefer current executable code and tests for behavior, then correct stale documentation as part of an authorized change. Plans under `plans/` are historical or proposed work, not current runtime truth.

## Start every task here

1. Read `docs/ARCHITECTURE.md` at least through the repository map, then read the sections for the affected subsystem.
2. Inspect `git status --short --branch` and the relevant diff. The worktree may contain unrelated user changes; preserve them.
3. Resolve the real package root. There is no root npm workspace, Cargo workspace, or universal task runner.
4. Read the applicable project skill completely:
   - `.agents/skills/react-vite-development/SKILL.md` for `web/src`, frontend tests, Vite, and the TypeScript side of the platform boundary.
   - `.agents/skills/axum-development/SKILL.md` for `curio-service` routers, business logic, SQLx, PostgreSQL, and OpenAI streaming.
   - `.agents/skills/code-verification/SKILL.md` before completing an implementation, regression check, or review.
5. Inspect the package manifest, lockfile, configuration, entrypoint, nearby implementation, and nearby tests. Search for analogous code with `rg` before inventing a pattern.
6. State the observable behavior and contracts that must remain stable. Make the smallest coherent change, then verify it proportionally.

Do not reformat, stage, revert, delete, or “clean up” unrelated work. Never edit generated output to solve a source problem.

## Repository map and authority

| Path | Responsibility | Status |
| --- | --- | --- |
| `web/src` | One shared React application for browser and Tauri desktop targets | Active |
| `web/src-tauri` | Tauri 2 Rust shell, native commands, service HTTP bridge, packaging | Active for desktop |
| `web/tests` | Frontend unit, protocol, architecture, and pure-domain tests | Active |
| `curio-service` | Axum service, OpenAI adapter, SQLx/PostgreSQL persistence, database tools | Active backend for both clients |
| `curio-workers` | Gemini/D1/Queue implementation retained as migration/reference code | Legacy and disconnected from active clients |
| `docs/ARCHITECTURE.md` | Canonical current architecture, contracts, limitations, and extension rules | Authoritative documentation |
| `docs/deployment.md` | Environment, database-role, Railway, SPA, and desktop release guidance | Active operations guide |
| `.github/workflows/shared-clients.yml` | Current CI command matrix | Active, but does not cover legacy Workers or live deployment |
| `plans` | Design history and proposed work | Non-authoritative |
| `.agents/skills` | Reusable repository-specific agent workflows | Active guidance |
| `.claude/settings.json` | Claude Code plugin sources enabled for this checkout | Active guidance |

Package roots and managers:

- `web`: npm using `web/package-lock.json`.
- `web/src-tauri`: Cargo using `web/src-tauri/Cargo.lock`.
- `curio-service`: Cargo using `curio-service/Cargo.lock`.
- `curio-workers/curio-chat-worker`: independent npm package.
- `curio-workers/curio-processor-worker`: independent npm package.

Do not mix package managers, create a root workspace casually, or upgrade dependencies/lockfiles unless the task requires it.

## System model

Curio builds two clients from one React tree:

```text
                         web/src
                  shared React application
                            |
             typed API + raw-byte platform contract
                  /                         \
      browser fetch adapter          Tauri TypeScript adapter
                  |                         | IPC Channel
                  |                  Tauri Rust HTTP bridge
                  \                         /
                         curio-service
                    Axum + SQLx + OpenAI
                         /           \
                  PostgreSQL     Responses API
```

The web artifact is a BrowserRouter SPA whose host needs deep-link fallback. The desktop artifact uses HashRouter and delegates HTTP/OS access to app-owned Rust commands. `curio-service` is the active backend. The Cloudflare Workers are not a runtime fallback.

## Cross-system invariants

Preserve these unless the task explicitly changes the architecture and updates tests/documentation with it:

- One `web/src` React implementation serves both web and desktop.
- Vite aliases and matching web/desktop TypeScript configs select router and platform implementations at build time.
- Shared pages/components do not call browser network primitives, import Tauri APIs, construct service origins, or import concrete platform adapters.
- JSON resources go through `web/src/api`; chat uses the typed streaming layer under `web/src/features/chat`.
- Rust and TypeScript wire contracts are mirrored manually; endpoint/event
  changes update both sides and their contract tests together.
- Desktop requests use validated relative paths and remain on the compiled service origin.
- The Tauri bridge transports status and raw bytes; it does not duplicate JSON or Curio SSE parsing.
- OpenAI keys, database URLs, migrator credentials, and other secrets never enter Vite variables, client storage, logs, fixtures, or committed environment files.
- PostgreSQL schema changes are explicit, append-only migrations. Application startup verifies migration state but does not apply DDL.
- The service build excludes SQLite; archived SQLite migrations remain outside
  request handling and there is no active importer.
- Calendar date-only strings and timestamp strings retain distinct public semantics.
- Current auth limitations must be described honestly. Client guards and CORS are not authorization.
- New external links and remote media use an intentional platform capability,
  protocol allowlist, and desktop CSP review; do not copy current direct-anchor
  debt for untrusted values.
- The legacy Worker topology stays explicitly disconnected until a planned migration changes code, tests, deployment, and documentation together.

## Frontend work (`web/src`)

Use `.agents/skills/react-vite-development/SKILL.md` for the full workflow.

### Ownership

- Pages compose routes and page-owned state.
- Feature components own product presentation and adapters.
- Reusable component systems expose controlled data/contracts through deliberate `index.ts` barrels.
- React Query owns server cache and mutation lifecycle.
- Resource modules in `src/api` own JSON paths and wire types.
- `src/features/chat` owns SSE framing, validation, request/event correlation, and streaming transport behavior.
- `src/platform` owns browser/desktop capabilities.
- `src/app/route-manifest.ts` is the route inventory; `src/app/router.tsx` maps IDs to lazy pages and access guards.

Do not introduce a global context, generic repository, new state/form/query library, or shared abstraction without a concrete ownership or reuse need. Keep state at the narrowest valid owner and derive render values rather than mirroring them with Effects.

### Platform boundary

`npm run check:architecture` enforces:

- only `src/platform/web.ts` may use browser networking;
- only `src/platform/desktop.ts` may import Tauri APIs;
- only `src/api` and `src/features/chat` may import `@curio/platform-runtime`;
- shared code may not import concrete platform implementations;
- a second desktop React source tree may not reappear.

Extend `PlatformServices` only for a real target difference. Implement both target behaviors or make target availability explicit. Native validation belongs in Tauri Rust, behind a narrow app-owned command.

### Data and UI truth

Do not infer service persistence from polished UI:

- auth is a local development session;
- Calendar create/list and Account profile save reach the service;
- Calendar task list/kanban are projections of calendar events;
- chat streams/persists new messages but starts from demo UI state;
- Notes, Vault, Atlas, Profile Setup, and much of Home/Settings remain mock, local, or in-memory.

Keep loading, empty, error, disabled, cancellation, and retry states explicit for new async work. Client validation improves UX; the service remains authoritative.

### Styling and interaction

Follow Tailwind CSS 4, shadcn/Radix primitives, Inter Variable, and semantic CSS theme tokens. Preserve keyboard behavior, focus, accessible names, reduced motion, contrast, text zoom/reflow, and responsive reachability. Avoid hard-coded brand/palette values where a semantic role exists.

Never put secrets in `VITE_*`. `VITE_CURIO_SERVICE_URL` is public build-time configuration. Production builds require HTTPS unless the documented local debug opt-in is deliberately used.

## Desktop Rust work (`web/src-tauri`)

Keep the current split:

- `core`: framework-independent request/packet types and validation.
- `repositories`: Reqwest service access and in-flight cancellation storage.
- `services`: proxy and external-link use cases.
- `commands`: thin Tauri IPC adapters.

The service proxy accepts only the allowed methods, validates relative paths/origin and bearer header values, streams raw body chunks over a typed channel, and cleans request registry entries on every return path. Preserve cancellation on abort, handler failure, and native interruption.

Desktop external links go through the Rust command and require the existing secure policy. Review CSP, capabilities, tests, and both TypeScript/Rust packet shapes whenever adding native access. Avoid broad Tauri permissions or frontend plugins as shortcuts.

## Axum service work (`curio-service`)

Use `.agents/skills/axum-development/SKILL.md` for the full workflow.

### Composition and layering

Production uses `app_with_config`; tests that need the complete application generally use `app_with_database`. The public `app()` contains only base routes and is not the full service.

Model new database-backed features after the Calendar vertical slice when the responsibilities are real:

```text
handler (Axum/HTTP/auth)
  -> service (validation/business rules/error classification)
     -> repository (typed SQL/row mapping)
        -> Database (pool/migration lifecycle)
```

Keep each backend domain under `curio-service/src/<domain>/` with a consistent
spine: `model.rs`, `repository.rs`, `service.rs`, `handler.rs`, and `route.rs`.
`mod.rs` is the domain boundary and should primarily expose route composition.
Add narrowly owned modules such as `error.rs`, `time.rs`, or `provider.rs` only
when that domain has a concrete need.

Keep feature SQL out of `Database`. Bind every SQL value. Reuse the pool. Sanitize storage/provider failures before returning them and never log credentials or sensitive payloads.

### Authentication and errors

The bearer token is currently treated as a development user ID. Calendar checks ownership, user upsert has an ownership gap, and Assistant stream/history routes are unprotected and globally scoped. Do not present this as production auth. A real auth change must coordinate tokens, middleware, all routes, ownership schema, client behavior, and tests.

There is no single global error envelope today. Preserve the owning feature's established contract unless the task is an intentional cross-service normalization.

### PostgreSQL and migrations

- Runtime/CI target PostgreSQL; the service does not enforce the server major version at connection time.
- Application and migrator credentials are separate.
- `CURIO_DB_SCHEMA` is validated and used for the connection search path.
- Add a migration under `curio-service/migrations/postgres` only when the schema
  changes; never create no-op migrations or edit an applied migration.
- Schema changes also require an impact check for `ops/postgres/verify.sql`,
  archived SQLite compatibility, deployment notes, and database-backed tests.
- Run DDL through `curio_db migrate`, not service startup.
- Readiness means database access plus expected migration-row verification.
- `curio_db verify` performs additional catalog checks.
- Database tests must use a safe `CURIO_TEST_DATABASE_URL` and disposable `curio_test_*` schemas.

Do not run migrations, imports, or destructive database tools against a persistent target merely to “see if they work.” Resolve and verify the exact target first; follow `docs/deployment.md`.

### Contract-sensitive domains

The Assistant domain must persist initial records before the provider call and persist terminal/interrupted state before the corresponding terminal event where possible. Keep provider parsing incremental and errors sanitized.

Calendar preserves trimmed public wire values while separately normalizing half-open UTC intervals for overlap queries. Maintain date-only/timed/milestone rules, exclusive all-day ends, bounded views, and DST slack.

For owned-resource reads or mutations, the authenticated identity is
authoritative. Scope persistence by both owner and resource ID; do not authorize
only against a body/query `userId`. Define PATCH omitted/null semantics and
revalidate/recompute dependent stored fields atomically.

## Legacy Worker work (`curio-workers`)

Treat this code as isolated migration/reference material. Each Worker has its own dependencies, Wrangler configuration, bindings, and tests. Root CI does not run them. Do not route active clients to Workers, share their Gemini/D1 assumptions with `curio-service`, or deploy them unless the task explicitly requests a legacy migration/deployment change.

If a Worker changes, keep its runtime schema initialization and `curio-workers/migrations` aligned and test the changed package independently.

## Verification matrix

Read `.agents/skills/code-verification/SKILL.md` before final verification. Start narrow, then expand to every affected boundary.

### Frontend (`web`)

```bash
npm run check:architecture
npm run lint
npm test
VITE_CURIO_SERVICE_URL=https://service.example.com npm run build:web
VITE_CURIO_SERVICE_URL=https://service.example.com npm run build:desktop-ui
```

Use the example URL only for offline build validation; it proves no deployed connectivity.

### Tauri (`web/src-tauri`)

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

### Service (`curio-service`)

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Database-backed tests can return early when `CURIO_TEST_DATABASE_URL` is absent. A green command without that safe variable is partial evidence, not a PostgreSQL pass.

### Legacy Worker package

```bash
npm run typecheck
npm test -- --run
```

For docs-only changes, verify Markdown structure, paths, commands, links, skill YAML, and `git diff --check`. Do not run expensive unrelated suites as ceremony; do run all consumers of any changed contract.

## Security and operational rules

- Never commit `.env` files or real keys. Examples contain empty/public placeholders only.
- Do not print secret-bearing environment values, DSNs, authorization headers, upstream bodies, or user-sensitive content.
- Keep provider and database access server-side.
- Treat CORS, client route guards, hidden UI, and TypeScript types as non-security boundaries.
- Validate identifiers, URL origins/protocols, ownership, and untrusted decoded data at their authoritative boundary.
- Use narrowly scoped, recoverable file/database operations. Confirm exact targets before destructive actions.
- Distinguish `/health` liveness from `/ready` dependency/migration readiness.
- Consider long-lived SSE before applying global HTTP timeouts, buffering, compression, or middleware.

## Documentation maintenance

`docs/ARCHITECTURE.md` is the single architecture source of truth. Update it when a change alters topology, package responsibilities, route/API contracts, persistence, trust boundaries, configuration, deployment, or verification. Update `docs/deployment.md` when operator steps or secrets/roles change. Keep the README short and link to the canonical docs.

Architecture review is mandatory for every feature, fix, refactor, and
migration—not only for work labeled “architecture.” Before handoff, compare the
implemented diff with every relevant architecture section. Update the canonical
document in the same change when current behavior, ownership, flows, contracts,
configuration, limitations, security posture, or commands changed. Remove a
documented limitation only when code and verification prove it is resolved. If
the document remains fully accurate, report `Architecture impact: none` rather
than editing prose cosmetically.

When source behavior and a plan disagree, update the canonical current-state document and leave plan history intact unless the task explicitly asks to revise it. Do not create another competing `ARCHITECTURE.md`.

## Completion standard

Before handing off:

- inspect the final diff and status;
- ensure no unrelated or generated files were changed;
- run `git diff --check`;
- run the smallest trustworthy test matrix for every affected contract;
- complete the architecture-impact check and update
  `docs/ARCHITECTURE.md` when required;
- state exact commands and manual checks with pass/fail/skip results;
- disclose missing prerequisites and residual risk without exposing secrets;
- distinguish pre-existing failures only when evidence proves they predate the work.

Do not claim a skipped, truncated, environment-gated, or unrun check passed. Do not claim deployment, production authentication, browser accessibility, or database compatibility from compilation alone.
