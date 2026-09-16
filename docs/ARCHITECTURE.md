# Curio architecture

This is the repository's single canonical architecture document. It describes
the system as it exists today: the active React clients, their browser and Tauri
runtimes, the Axum service, PostgreSQL persistence (with PostgreSQL 18 as the
current CI and deployment target), the OpenAI streaming path, reusable UI
systems, tests and build boundaries, and the disconnected Cloudflare Worker
implementation retained for reference.

It is both a map and a set of constraints. Sections labeled as current
limitations describe real gaps in the present code, not proposed behavior.

## Maintenance contract

Every feature, fix, refactor, migration, or operational change must include an
architecture-impact check. Update this file in the same change whenever the
work changes any described runtime behavior, ownership boundary, route or wire
contract, persistence model, configuration, trust boundary, deployment step,
known limitation, extension rule, or verification command. Remove limitations
when they are actually fixed and add newly discovered constraints that future
work must respect.

Keep this document about current executable behavior. Proposed designs belong
in `plans/` until implemented. A local fix that leaves every architectural claim
accurate does not need prose churn, but its handoff must explicitly say that the
architecture-impact check found no documentation change. Do not create another
`ARCHITECTURE.md`; maintain this canonical file.

## System at a glance

Curio maintains one React application and produces two client artifacts. Both
clients target one Axum service; only the transport between the UI and that
service changes.

```text
                                      build-time target selection
                                             │
                           ┌─────────────────┴─────────────────┐
                           │                                   │
                    Web static SPA                      Tauri desktop app
                    BrowserRouter                        HashRouter
                           │                                   │
                    browser fetch                    TypeScript Tauri adapter
                           │                                   │ IPC channel
                           │                           Rust command/service/repo
                           │                                   │ reqwest
                           └─────────────────┬─────────────────┘
                                             │
                                      curio-service
                                    Axum + Tokio + SQLx
                                      /             \
                              PostgreSQL      OpenAI Responses API

                 curio-workers: legacy Gemini + D1 + Queue pipeline
                 (kept in the repository, not called by either active client)
```

The principal architectural choices are:

1. `web/src` is the only product UI source tree.
2. Browser/desktop differences are selected by Vite aliases, not by runtime
   platform checks spread through components.
3. Shared UI code reaches the service through one transport contract.
4. JSON resources live in `web/src/api`; streaming chat has a dedicated
   protocol layer.
5. The Tauri renderer delegates HTTP and OS integration to app-owned Rust
   commands.
6. `curio-service` is the active backend and owns OpenAI credentials and
   schema-scoped PostgreSQL persistence.
7. The calendar backend is the current reference vertical slice for new
   database-backed features.

## Repository map

| Path | Role | Runtime status |
| --- | --- | --- |
| `web/src` | Shared React application, platform contract, API clients, features, pages, hooks, and UI systems | Active for web and desktop |
| `web/src-tauri` | Tauri 2 Rust shell, native commands, service proxy, cancellation registry, packaging | Active for desktop |
| `curio-service` | Axum HTTP/SSE service, OpenAI adapter, SQLx/PostgreSQL persistence and database tools | Active for both clients |
| `curio-workers` | Previous Gemini chat Worker, D1 storage, queue processor, and embedding pipeline | Legacy; clients do not reference it |
| `web/tests` | Frontend protocol, state, transport, architecture, and pure-domain tests | Active |
| `web/src-tauri/tests` | Unit tests included from private Rust modules | Active |
| `curio-service/tests` | Service unit and full-router API tests | Active |
| `.github/workflows/shared-clients.yml` | Frontend, desktop Rust, and service CI | Active |
| `docs/ARCHITECTURE.md` | Canonical current architecture, contracts, limitations, and extension rules | Documentation only |
| `docs/deployment.md` | Environment, database-role, hosting, and release operations | Documentation only |
| `.agents/skills` | Repository-specific React/Vite, Axum, and verification guidance | Agent support |
| `plans` and ignored `*/tasks` folders | Design/implementation planning artifacts | Not runtime code |

There is no root npm, Cargo, or task-runner workspace coordinating everything.
`web`, `curio-service`, and each legacy Worker are independent package roots
with their own lockfiles and commands. The two Rust crates are also independent;
there is no shared Cargo workspace or shared Rust library.

Generated directories such as `node_modules`, `dist`, Rust `target`, Tauri
`gen`, local `.wrangler-state`, environment files, and local SQLite import
snapshots are not architectural source.

### Technology and toolchain snapshot

| Area | Principal technology |
| --- | --- |
| Shared UI | React 19, TypeScript 5.8, Vite 7, React Router 7, TanStack React Query 5 |
| UI systems | Tailwind CSS 4, Radix/shadcn-style primitives, Plate, dnd-kit, React Flow |
| Desktop | Tauri 2 with a Rust 2021 crate and Reqwest/Tokio bridge |
| Service | Rust 2024 crate, Axum 0.8, Tokio 1, SQLx 0.8, Reqwest 0.12 |
| Active data/provider | PostgreSQL via SQLx; OpenAI Responses API via server-side Reqwest |
| Legacy Workers | Cloudflare Workers/Wrangler, D1, Queues, Gemini |

The npm and Cargo lockfiles are committed. CI pins Node 22 and uses the latest
available stable Rust toolchain, but the repository has no `engines`,
`packageManager`, `.nvmrc`, `rust-toolchain.toml`, or `rust-version` pin. Local
tool versions can therefore drift from CI even when dependency resolution is
locked.

## Active runtime topology

### Web

The web build publishes `web/dist` as a static SPA. `BrowserRouter` owns
client-side navigation, so the host must serve `index.html` for unknown product
routes while continuing to return normal 404 responses for missing assets.

Service requests go directly from the browser to the configured Curio service.
The service therefore controls browser access with an exact CORS allowlist.

### Desktop

The desktop build embeds the same compiled React source in a Tauri WebView and
uses `HashRouter`, avoiding server-style deep-link resolution for bundled
files. The renderer does not perform service HTTP itself:

1. The desktop TypeScript adapter invokes `service_request`.
2. Rust validates the relative path, method, optional JSON/query payload, and
   optional bearer token.
3. Rust performs the request with a shared Reqwest client.
4. Status and raw body bytes return over a typed Tauri `Channel`.
5. TypeScript feeds those bytes into the same JSON or SSE code used by the web
   build.

Because Rust originates the HTTP request, desktop traffic does not depend on
browser CORS.

### Service

`curio-service` is one process containing liveness, readiness, user, calendar,
task, conversation history, and streaming chat routes. At startup it opens the
configured PostgreSQL schema, verifies that the expected migration is already
applied, constructs feature repositories, and merges the routers under one CORS
layer. Schema changes run separately through `curio_db migrate`; the running
Axum service does not perform DDL.

The service calls the OpenAI Responses API for chat. Neither client receives the
OpenAI API key.

## Frontend architecture

### Bootstrap and global providers

`web/src/main.tsx` mounts one React root. Provider order is:

```text
QueryClientProvider
  └─ ThemeProvider
       └─ AuthProvider
            └─ App
                 └─ build-selected AppRouter
```

- React Query owns cached server state and mutations.
- `ThemeProvider` owns the active palette, appearance, and custom themes.
- `AuthProvider` owns the current development user.
- `App` contains no platform logic; it renders the router selected by Vite.
- `chat-provider.tsx` is currently empty and unused. Chat state is page/hook
  state, not a global context.

The root is not wrapped in `React.StrictMode`, the QueryClient uses default
options, and there is no application-level React error boundary.

Most authenticated product pages construct the common shell locally with
`SidebarProvider`, `AppSidebar`, `SidebarInset`, and `PageHeader`.
The shell is reused compositionally rather than mounted once above the router.
Its width cookie is read, but the open/collapsed cookie is currently write-only,
so collapse state can reset when a page-local shell remounts during navigation.

### Build-selected routing

`web/src/app/route-manifest.ts` is the canonical route inventory. Each route
declares an access level and target availability. `router.tsx` maps route IDs
to lazy-loaded pages and applies two guards:

- `RequireAuth` redirects anonymous users to `/login` and records the requested
  path in navigation state. The Login page currently ignores that state and
  always navigates a successful local login to `/home`.
- `RedirectIfAuthenticated` keeps signed-in users out of the login page.

Vite and the target TypeScript configs resolve `@curio/router-runtime` to:

- `router.web.tsx`: `BrowserRouter`, with a public Landing page at `/`.
- `router.desktop.tsx`: `HashRouter`, with `/` redirecting to `/home` or
  `/login` according to the local auth session.

Unknown paths redirect to `/`. Pages are route-level code-split with
`React.lazy` and a shared Suspense loading state.

### Route and feature status

The UI deliberately has mixed persistence maturity. “Authenticated” in this
table means protected by the client route guard; it does not imply production
identity verification.

| Route | Main responsibility | Current data source/persistence |
| --- | --- | --- |
| `/` | Marketing landing page on web; auth-aware redirect on desktop | Static presentation |
| `/login` | Email form and local session creation | Mock user in browser/WebView `localStorage` |
| `/verify-email` | Placeholder | Redirects to `/login` |
| `/profile-setup` | Four-step profile wizard | Component memory only; completion navigates home |
| `/home` | Dashboard, upcoming items, recent cultivations, quick thought | Mostly mock data; thought stored per user in `localStorage` |
| `/chat` | Chat list, composer, streaming response UI | Starts from demo data; new requests persist in service PostgreSQL, but UI state remains in memory |
| `/calendar` | Calendar plus task list/kanban projections | Server-backed create/range-list via React Query and PostgreSQL |
| `/notes` | Folder/sidebar plus Plate rich-text editor | Mock seed cloned into component memory; no reload persistence |
| `/vault` | Search/filter/pagination UI | Static mock records filtered in memory |
| `/atlas` | React Flow knowledge graph | Static initial graph plus component state |
| `/profile`, `/settings` | Shared settings page | Account tab writes the PostgreSQL-backed user profile; themes persist locally; most other tabs are construction/local state |

Important consequences:

- The route guard and the service do not share a real authentication session.
- Saving the Account tab writes the service profile but does not update the
  `AuthProvider` user stored in local state.
- The Profile Setup wizard does not call the profile API.
- Chat history endpoints exist, but the Chat page does not call
  `listConversations` or `conversationMessages`; it initializes from
  `demo-data.ts`.
- Notes, Vault, Atlas, and most Home data are not service resources yet.

### Frontend dependency direction

Shared code follows this intended direction:

```text
pages/components
      │
      ├─ hooks and target-independent feature logic
      │         │
      │         ├─ typed JSON resources in src/api
      │         └─ streaming chat transport/protocol
      │                          │
      └──────────────────────────┴─ ServiceTransport contract
                                      │
                         build-selected platform adapter
```

Pages and components should not:

- call `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, or
  `sendBeacon`;
- import a concrete `platform/web` or `platform/desktop` module;
- import Tauri APIs;
- construct service URLs.

`npm run check:architecture` scans `web/src` and enforces those rules. ESLint
adds import/network restrictions. Only `src/api` and `src/features/chat` may
import the build-selected `@curio/platform-runtime`; only
`src/platform/desktop.ts` may import `@tauri-apps/*`; only
`src/platform/web.ts` may use browser networking.

The gate also fails if the removed second desktop React tree reappears. This
keeps web and desktop on one UI implementation.

### Platform contract

`web/src/platform/contracts.ts` defines a deliberately small boundary:

- `ServiceTransport.stream(init, handlers)` reports one HTTP status and then
  ordered raw byte chunks.
- `requestService` buffers that stream for ordinary JSON requests.
- `PlatformServices` exposes the selected target, service transport, and
  external-URL opening.
- `AbortSignal` is the common cancellation primitive.
- Transport failures use `ServiceTransportError`; HTTP and protocol failures
  remain higher-layer concerns.

The contract supports `GET`, `POST`, `PATCH`, and `DELETE`. GET payloads
are query objects; other payloads are JSON bodies. The current APIs only need GET
and POST, but the common transport is ready for resource updates/deletes.

#### Web adapter

`platform/web.ts`:

- resolves contract-supplied paths against the build-time service URL;
- serializes GET queries and JSON request bodies;
- attaches the optional bearer token;
- reports the status before reading the body stream;
- forwards raw `ReadableStream` chunks;
- uses the caller's `AbortSignal`;
- opens HTTP/HTTPS external links with `window.open` and
  `noopener,noreferrer`.

Unlike the Rust desktop repository, the web adapter does not independently
reject a protocol-relative or absolute cross-origin path before `new URL`.
Current callers use code-owned relative constants, but runtime origin parity is
a known hardening gap if paths ever become data-driven.

#### Desktop adapter

`platform/desktop.ts`:

- creates a request UUID and typed Tauri Channel;
- invokes `service_request` with the relative path and payload;
- converts base64 channel chunks back to `Uint8Array`;
- maps native error packets to `ServiceTransportError`;
- invokes `cancel_request` on abort or handler failure;
- treats a native invocation that lacks an `end` packet as an incomplete
  transport;
- sends external URLs to the app-owned `open_external_url` command.

The platform service's external-link policy is intentionally stricter on
desktop: browsers accept HTTP or HTTPS, while Rust permits only
credential-free HTTPS links. This is not yet a universal rendering boundary;
some task-card and rich-text link/media elements render direct anchors or URLs
instead of calling `PlatformServices.openExternalUrl`.

### JSON API layer

`web/src/api/client.ts` is the buffered JSON client. It:

1. calls the selected `ServiceTransport`;
2. buffers response bytes;
3. maps non-2xx bodies into `ApiError`;
4. returns `undefined` for an empty successful body;
5. rejects malformed success JSON.

Resource modules own paths and wire types:

| Module | Service surface |
| --- | --- |
| `api/users.ts` | `POST /v1/users` |
| `api/calendar.ts` | `POST/GET /v1/calendar/events` |
| `api/conversations.ts` | `GET /v1/conversations` and messages |

Hooks add UI ownership and cache behavior:

- `useSaveUser` supplies the development bearer ID and runs the user mutation.
- `useCalendarEvents` keys ranges by user/view/start/end, passes an abort
  signal, and maps service records to calendar `TaskItem` values.
- `useCreateCalendarEvent` supplies the current user and invalidates all of
  that user's cached calendar ranges after settlement.

No page should add a direct request when a resource module/hook is the correct
home.

### API contract ownership

Curio has no OpenAPI document, generated SDK, shared Rust/TypeScript DTO package,
or client-generation job. The GitHub workflow named `shared-clients` verifies
the shared web/desktop source; it does not generate clients.

HTTP contracts are mirrored manually between Rust Serde types and TypeScript
wire types. `api.get<T>`/`post<T>` verify status and JSON syntax but currently
cast decoded JSON without runtime schema validation. A contract change must
therefore update the Rust request/response shape, TypeScript wire type, service
API test, and relevant frontend resource/protocol test together. Preserve
camelCase JSON names and calendar date-only-versus-RFC-3339 semantics.

### Streaming chat protocol

Chat is not handled by the buffered JSON client. Its frontend layers are:

- `features/chat/chat-stream.ts`: normalized event types, SSE framing parser,
  UTF-8 streaming decoder, terminal-state checks, and request/event correlation.
- `features/chat/transport.ts`: sends `POST /v1/chat/stream`, separates
  non-2xx error bytes from successful SSE bytes, and feeds a
  `ChatStreamSession`.
- `hooks/useChat.ts`: owns per-chat AbortControllers, optimistic user/assistant
  messages, token appends, terminal errors, and unmount cancellation.
- `pages/Chat.tsx`: owns visible chat/sidebar/message state.

The same TypeScript parser runs for browser HTTP chunks and Tauri channel chunks.
Rust transports bytes and does not parse the Curio SSE protocol.

The TypeScript request includes `userId`, but the current Rust
`ChatStreamRequest` does not. Serde ignores that extra field, and active chat
rows have no user ownership.

`useChat` stores AbortControllers per chat, but exposes one global
`isStreaming`; the current page therefore serializes sends across all chats.
The hook also exposes cancellation/error state that the page does not currently
render or wire to a cancel control.

### Calendar and the current Tasks view

The calendar feature is the most complete frontend-to-database flow:

1. `EventCalendar` reports the visible day/week/month/agenda range.
2. `useCalendarEvents` requests that exact UTC range with the mock bearer ID.
3. The API record adapter preserves public date strings and maps
   `title -> name`, `startDate -> setAt`, and `endDate -> expireAt`.
4. React Query caches each visible range.
5. Root-level calendar creation posts a new event and invalidates cached ranges.

Date format is part of the rendering contract:

- bare `YYYY-MM-DD` means all-day;
- RFC 3339 timestamps mean timed entries;
- a timestamp without an end means a milestone.

Do not normalize date-only values into timestamps in the API client. That would
move an all-day entry across calendar days in some time zones and change its
visual classification.

The Calendar/Tasks switcher currently presents two views over the same
`calendar_events` data:

- Calendar uses the event-calendar component.
- Tasks/List uses the generic data table.
- Tasks/Kanban groups the same records by event status.

The service now has an independent user-owned `tasks` table and authenticated
create/list routes at `/v1/tasks`, but the frontend has no task API client or
hook yet. Existing stored events remain read-only in the UI because the service
has no calendar update or delete endpoints. Calendar editing permissions
disable rename, move, resize, status changes, and delete; root creation remains
available and persisted. The visible Tasks projections still use the most
recently reported calendar range rather than the independent task resource.
Tasks mode renders explicit loading/error UI, while calendar-mode initial
loading or failure currently appears as empty data; create failures are also
not surfaced to the user.

The reusable editing extension can emit `onChange` plus granular edit/reschedule
callbacks for one logical action. This is harmless while persistence is
disabled, but a future service adapter must choose one canonical mutation
boundary or it can duplicate requests; relying only on a granular callback can
also miss edits such as priority changes.

### Reusable UI systems

`web/src/components` contains both application features and package-like
reusable systems:

| Area | Responsibility |
| --- | --- |
| `ui` | shadcn/Radix-style primitives and layout foundations |
| app shell files | Main sidebar, navigation groups, user menu, page header |
| `task-card` | Canonical recursive `TaskItem` model, presentation, editing, permissions, color/time logic, and cross-surface clipboard format |
| `event-calendar` | Month/week/day/agenda projections of `TaskItem[]`, occurrence/classification logic, headless parts, and opt-in editing extension |
| `kanban-board` | Generic columns/items/renderers, drag/drop reducers, permissions, palette, and inline editing |
| `data-table` | Small generic table abstraction used by the task list |
| `rich-text-editor` | Plate-based editor/viewer, plugins, renderers, toolbars, images, links, tables, and code highlighting |
| feature folders | Calendar adapters, Notes, Vault, Home, Settings, profile setup, landing, and auth presentation |

The task component family shares one data and permission language. The
event-calendar re-exports the task types it consumes, while its editing feature
is injected through an extension value so the base calendar does not statically
depend on edit gestures and overlays.

These systems live in the application source tree, not in separate versioned
packages. Treat their public `index.ts` barrels and controlled-data contracts
as local package boundaries; page-specific network or persistence logic should
stay outside them.

### Styling and themes

The frontend uses Tailwind CSS 4, Inter Variable, semantic CSS variables, and
shadcn-compatible aliases in `index.css`.

The color system has three layers:

1. A theme seed (`--theme-hue`, `--theme-chroma`).
2. A generated 50–950 tonal scale.
3. Semantic roles such as background, foreground, primary, border, chart, and
   sidebar tokens.

Light and dark appearance remap semantic roles onto the same scale. Components
should consume semantic roles instead of raw colors so custom themes and dark
mode remain coherent. Fixed syntax-highlight colors are the deliberate
exception.

`ThemeProvider` reads/writes a versioned local preference, follows system
appearance when requested, applies the theme in a layout effect to avoid flash,
and supports a capped list of custom color seeds.

## Axum service architecture

### Process startup and composition

`curio-service/src/main.rs`:

1. loads a local `.env` if present;
2. initializes compact structured diagnostics from `RUST_LOG`;
3. parses required/default configuration;
4. constructs the full router with `app_with_config`;
5. binds `CURIO_SERVICE_ADDR`, otherwise `0.0.0.0:$PORT` when Railway-style
   `PORT` exists, otherwise `127.0.0.1:3000`;
6. calls `axum::serve`.

`app_with_config` is the real composition root. It:

- validates configured CORS origins as HTTP header values;
- opens a schema-scoped PostgreSQL pool with bounded connection settings;
- verifies the latest embedded PostgreSQL migration without applying DDL;
- delegates Calendar, Task, User, and Assistant construction to each domain's
  `route.rs` composition function;
- creates `AssistantService` from OpenAI configuration and
  `AssistantRepository` inside the Assistant route module;
- merges base, readiness, assistant, calendar, task, and user routers;
- applies CORS to the combined application.

The public `app()` helper is materially smaller: it returns only root, health,
and legacy placeholder routes. Tests use it, but production must use
`app_with_config`.

OpenAI configuration is required at process startup even when only health,
calendar, or user functionality is wanted. The server distinguishes process
liveness from database readiness, but currently has no graceful shutdown,
request IDs, rate limit, or explicit project-level timeout/body-size policy
beyond framework and dependency defaults. Targeted structured diagnostics cover
startup, readiness, and the assistant/provider/persistence path, but there is no
request-wide tracing middleware.

### Route inventory and protection

| Method and path | Service auth | Persistence/behavior |
| --- | --- | --- |
| `GET /` | Public | Static text |
| `GET /health` | Public | Process liveness; always returns `{"status":"ok"}` |
| `GET /ready` | Public | `SELECT 1` plus expected migration check; `503` when unavailable |
| `POST /user/conversations` | Bearer middleware | Legacy non-persistent placeholder |
| `GET /user/conversations/{id}` | Bearer middleware | Legacy non-persistent placeholder |
| `POST /user/conversations/{id}` | Bearer middleware | Legacy non-persistent placeholder |
| `POST /v1/users` | Bearer middleware | PostgreSQL user upsert with `RETURNING` |
| `POST /v1/calendar/events` | Bearer plus owner check | PostgreSQL create with `RETURNING` |
| `GET /v1/calendar/events` | Bearer plus owner check | Bounded PostgreSQL overlap query |
| `POST /v1/tasks` | Bearer plus owner check | PostgreSQL create with `RETURNING` |
| `GET /v1/tasks` | Bearer plus owner check | Owner-scoped list ordered by creation time and ID |
| `POST /v1/chat/stream` | No service middleware | PostgreSQL + OpenAI + normalized SSE |
| `GET /v1/conversations` | No service middleware | All conversations, unbounded |
| `GET /v1/conversations/{id}/messages` | No service middleware | Messages or empty list |

CORS allows exact configured origins, the GET/POST/PATCH/DELETE methods, and
`Content-Type` plus `Authorization`. CORS controls browsers; it is not
authentication.

### Development authentication model

The current middleware is explicitly a placeholder:

- It requires `Authorization: Bearer <token>`.
- The raw nonblank, whitespace-free token becomes `CurrentUser.id`.
- It verifies no signature, expiry, issuer, or database record.
- Unauthorized responses are empty `401` responses.

Calendar and Task compare that ID with the client-supplied `userId`. The user
upsert extracts `CurrentUser` but does not compare it with the body ID, so any
syntactically valid bearer can currently upsert any user ID. Assistant
stream/history routes are unprotected and conversations have no owner column.

The client route guard is therefore a UX boundary, not a security boundary.
Production auth requires coordinated frontend tokens, service verification,
route-wide enforcement, and data ownership migrations.

### Backend domain modules

Calendar, Task, Assistant, and User are domain modules under `src/`. Every
domain uses the same five-file spine:

| File | Responsibility |
| --- | --- |
| `model.rs` | Domain records, request/response DTOs, and repository inputs |
| `repository.rs` | Bound PostgreSQL statements and typed row mapping |
| `service.rs` | Validation, business rules, orchestration, and error classification |
| `handler.rs` | Axum extraction plus HTTP/SSE response adaptation |
| `route.rs` | Domain dependency construction, paths, middleware, and Axum state |

`mod.rs` is only the domain boundary and exports the route composition function.
Domains may add narrowly owned files when the common spine is insufficient:
Calendar has `time.rs` and `error.rs`, Task and User have `error.rs`, and
Assistant has `provider.rs` for the OpenAI adapter.

Active persistence follows feature-local typed repositories:

| Area | Current organization |
| --- | --- |
| Calendar | Authenticated event create/range-list plus temporal normalization |
| Task | Authenticated create/list for independent task aggregates |
| Assistant | OpenAI streaming, chat persistence, normalized SSE, and conversation history |
| User | Profile upsert plus legacy non-persistent conversation placeholders |
| Database | PostgreSQL pool configuration, migration execution/verification, readiness, and timestamp serialization; no feature SQL |
| Core | `core::sql`, the shared bind-first statement builder every repository composes its SQL with; no feature SQL and no table knowledge |

For new database-backed features, Calendar is the reference pattern:

```text
feature router
  └─ handler: Axum extraction, auth/ownership, HTTP envelope
       └─ service: validation, defaults, business rules, error classification
            └─ repository: parameterized SQL and row mapping
                 └─ Database: pool and migration lifecycle
```

Keep feature-specific SQL and transport DTO dependencies out of the central
`Database` implementation.

### PostgreSQL lifecycle and SQLite import boundary

`Database::connect`:

- requires a PostgreSQL URL and a conservatively validated schema identifier;
- creates a bounded `PgPool` with a configured acquire timeout and application
  name;
- sets UTC and the validated `<schema>,pg_catalog` search path on every pooled
  connection;
- verifies that the selected schema exists and is accessible;
- opens the pool without applying DDL.

`app_with_config` checks that the newest embedded migration has a successful
row in `_sqlx_migrations` before serving. It does not validate the checksum or
full migration history, and it does not perform a catalog audit. `curio_db
migrate` uses `CURIO_MIGRATOR_DATABASE_URL` to apply `migrations/postgres`;
`curio_db verify` uses the application `DATABASE_URL` and adds an exact expected
application-table catalog check. Both select the target through
`CURIO_DB_SCHEMA`. This keeps the production application role DML-only and
makes a failed pre-deploy migration stop the rollout.

The original three SQLite migrations remain under `migrations/sqlite` solely
for audit compatibility. The default and all-feature builds enable only
`sqlx-postgres`; there is no active SQLite driver, importer binary, or request
path. `sqlite_import_manifests` remains in the PostgreSQL baseline as historical
schema, but the current service does not write it.

Current application tables (in addition to SQLx's `_sqlx_migrations` metadata):

| Table | Ownership and purpose |
| --- | --- |
| `conversations` | Globally keyed chat conversation; no user owner |
| `messages` | User/assistant content, lifecycle status, provider/error IDs, and identity `sort_order`; cascades with conversation |
| `users` | Profile keyed by user ID; email unique |
| `calendar_events` | User-owned event with cascade delete, public date strings, and normalized range instants |
| `tasks` | User-owned task state, optional due time, and stable owner-list ordering |
| `sqlite_import_manifests` | Historical redacted import provenance; currently read-only/unused |

PostgreSQL stores audit and normalized-range instants as `timestamptz(3)` and
serializes public audit timestamps as millisecond RFC 3339 UTC strings.
`calendar_events.all_day` is a native boolean; message role/status and calendar
status/priority constraints live in the database. Task status, priority, and
due-time ordering are also database-constrained. Calendar wire
`start_date`/`end_date` strings remain text so date-only meaning is preserved.

Repositories compose statements with `core::sql`, the shared builder described
below, and map rows into typed `FromRow` records. Assistant start/finish
mutations remain transactional. User upsert, calendar create, and task create
use `INSERT ... RETURNING`. Conversation messages order by `created_at`, then the explicit
identity-backed `sort_order`, rather than an engine-specific implicit row
identifier.

### Shared statement construction

`src/core/sql.rs` owns statement construction for every repository. It knows
nothing about tables or domains; it exists to remove one specific failure mode.

A hand-written statement keeps three parallel lists in sync — the column list,
the `$n` placeholders, and the argument order. Every edit has to touch all
three, a mismatch still compiles, and it fails or binds the wrong column only at
runtime. In the builder, binding a value is what produces its placeholder, so
each column or filter is written once, beside the value it carries, and the
lists cannot drift apart.

Two properties hold by construction:

- Statement structure (`table`, `column`, filter fragments, `ORDER BY`,
  `RETURNING`) is `&'static str`, so only literals and constants reach the SQL
  text. Every caller-supplied value is bound.
- Placeholder numbering follows bind order, so reordering a chain cannot
  misalign arguments.

`Sql::select`, `Sql::insert_into`, and `Sql::update` are separate builder types,
so a clause is only reachable on the statement shape that accepts it.
`upsert_on(key)` refreshes every non-key column from the proposed row and stamps
`updated_at`; `execute_one` requires exactly one affected row and otherwise
returns `sqlx::Error::RowNotFound`. A bind that fails to encode is surfaced as
`sqlx::Error::Encode` rather than sent as an incomplete argument list.

Column lists live as `COLUMNS` constants next to each `FromRow` record, so the
selected set is written once and stays explicit. This is deliberate rather than
`SELECT *`: `calendar_events.starts_at`/`ends_at` and `messages.sort_order` are
internal columns that must not be fetched into public records.

### Assistant service flow

`POST /v1/chat/stream` follows a persistence-before-terminal-event rule:

1. Axum deserializes conversation, user-message, assistant-message IDs and the
   prompt.
2. A transaction inserts/touches the conversation, inserts the completed user
   message, and inserts a pending assistant row.
3. Only after commit does `OpenAiClient` call
   `{OPENAI_BASE_URL}/v1/responses` with `model`, `input`, and
   `stream: true`.
4. The provider SSE parser handles fragmented LF/CRLF event blocks and
   normalizes OpenAI events.
5. Text deltas are appended to an in-memory assistant buffer and emitted as
   Curio `token` events.
6. On provider completion, the full buffer and response ID are committed before
   the Curio `done` event.
7. On provider failure, partial content and a sanitized error code are committed
   before the Curio `error` event.
8. Serialization failure, downstream disconnect, or unexpected upstream end
   attempts to mark the assistant row `interrupted`.

The chat path logs correlation IDs already supplied by the client, provider HTTP
status or transport-error classification, OpenAI's `x-request-id` when present,
and sanitized database operation/error classifications. It deliberately does
not log prompts, message/response bodies, API keys, authorization headers, or
database URLs. A bounded, credential-redacted provider error description may be
forwarded to clients so failures remain diagnosable.

Public normalized events are:

- `token { conversationId, messageId, token }`
- `done { conversationId, messageId, responseId }`
- `error { conversationId, messageId, code, message }`

Raw provider HTTP bodies are not forwarded to clients. Parsed error labels and
messages may appear after credential redaction and length limiting.

Current chat limitations:

- IDs and prompts have no explicit validation or size limits.
- Client IDs are global primary keys; retrying identical message IDs is not
  idempotent.
- Only the latest prompt is sent to OpenAI; stored history is not model context.
- Partial tokens live only in process memory until a terminal/interruption
  update. A crash can leave a pending assistant row.
- Conversation list/history is unbounded, public, and not user-scoped.
- A missing conversation yields an empty message list rather than `404`.
- Reqwest has no explicit provider timeout configuration.

### Calendar vertical slice

`calendar/route.rs` constructs one `CalendarService` from one
`CalendarRepository`, attaches it as Axum state, nests the event routes, and
applies auth once. `calendar/mod.rs` only exposes that composition function.

Layer responsibilities:

- `model.rs`: public records, create/query inputs, repository insert input,
  and allowed status/priority values.
- `handler.rs`: Axum extraction, owner check, response status/envelope.
- `route.rs`: dependency construction, paths, auth middleware, and Axum state.
- `service.rs`: trimming, UUID generation, enum validation, temporal rules,
  bounded-range policy, and SQL error classification.
- `repository.rs`: insert/list SQL and one row mapper.
- `time.rs`: public date parsing and normalized half-open intervals.
- `error.rs`: stable domain errors mapped to JSON HTTP responses.

Calendar temporal invariants:

- all-day values are exact `YYYY-MM-DD`;
- timed values are RFC 3339 with an explicit offset;
- start and end use the same category;
- all-day ends are exclusive;
- missing all-day end becomes the following day;
- a timed event without an end receives a one-minute internal interval while
  keeping public `endDate: null`;
- overlap is `starts_at < requested_end AND ends_at > requested_start`;
- day/week/month/agenda requests are capped at 1/7/42/92 days plus two hours of
  DST slack.

Create returns the trimmed, validated public date representation and never
exposes `starts_at` or `ends_at`. Duplicate IDs map to `409`; missing user
foreign keys map to `422`; unexpected storage failures are logged and
sanitized.

There is no get-one, update, or delete route. Listing an unknown user returns an
empty result because it does not join the users table.

### Task vertical slice

`task/route.rs` constructs an authenticated domain router from
`TaskService`/`TaskRepository`; `task/mod.rs` only exports it. Tasks are
independent aggregates rather than second writes to `calendar_events`.

- `POST /v1/tasks` trims and validates task fields, defaults status/active/start
  time, generates a UUID when needed, and returns the persisted row with `201`.
- `GET /v1/tasks?userId=...` returns only that owner's rows ordered by
  `created_at DESC, id ASC`.
- Both routes compare the development bearer identity with `userId`.
- Status is one of `scheduled`, `in-progress`, `blocked`, `done`, or
  `cancelled`; priority is optional `low`, `medium`, or `high`.
- `setAt`/`dueAt` are RFC 3339 instants stored as `timestamptz(3)`, and a due
  time cannot precede the start time.
- There is no task get-one, update, delete, pagination, calendar projection, or
  frontend API consumer yet.

### User and legacy placeholder routes

`POST /v1/users` trims required values, converts a blank avatar to null,
upserts by ID, returns `200` for create/update, maps duplicate email to
`409`, and otherwise returns a sanitized error.

The `/user/conversations` handlers are unrelated to the real chat database
routes. They echo inputs/path IDs and exist only as authenticated placeholders.
Do not build new conversation behavior on them.

### Service error model

Error behavior is currently boundary-specific:

| Source | Current response |
| --- | --- |
| Auth middleware | Empty `401` |
| Calendar domain/storage | JSON `{ "error": "..." }` |
| Task domain/storage | JSON `{ "error": "..." }` |
| User validation/conflict/storage | JSON `{ "error": "..." }` |
| Axum extractor rejection | Framework-default response |
| Chat/provider/storage after SSE starts | HTTP success stream with terminal `error` event |
| Conversation-history database read | Bare `500` |
| Startup CORS/database/bind failure | Structured error/exit or panic at composition/startup |

There is no common application error type or request-wide logging/correlation
middleware. Structured diagnostics are currently targeted at startup,
readiness, and chat provider/persistence failures.

## Tauri desktop architecture

`web/src-tauri` applies an explicit four-layer split:

| Layer | Responsibility |
| --- | --- |
| `core` | Framework-independent service method/request/packet types and external-link validation |
| `repositories` | Reqwest service access and in-flight cancellation storage |
| `services` | Use-case orchestration for proxying requests and validating links |
| `commands` | Thin Tauri IPC adapters and channel event sink |

`AppServices` constructs one `ServiceRepository`, one `RequestRegistry`,
one `ServiceProxy`, and the external-link service. Tauri manages that state and
registers three app-owned commands:

| Command | Purpose |
| --- | --- |
| `service_request` | Execute and stream an allowed service request |
| `cancel_request` | Cancel/remove an in-flight native request by UUID |
| `open_external_url` | Validate an HTTPS URL and open it through the Rust-side opener plugin |

### Native service proxy

`ServiceRepository` owns one shared Reqwest client with a ten-second connect
timeout. It:

- accepts only GET/POST/PATCH/DELETE;
- requires a path beginning with one slash and rejects protocol-relative paths,
  backslashes, fragments, credentials, and origin changes;
- attaches GET payloads as query values and other payloads as JSON;
- trims/validates bearer headers and marks them sensitive;
- accepts HTTPS service base URLs, with loopback HTTP only in debug builds.

`ServiceProxy` registers each request's `CancellationToken` before executing
it and removes the registry entry on every return path. It selects between
cancellation and connection/body progress, then sends:

- `started { status }`
- zero or more base64 `chunk { bytes }`
- `end`, or
- `error { code, message }`

The proxy deliberately has no overall response timeout so long-lived SSE can
continue. It does not interpret status bodies or SSE semantics.

### Desktop security and packaging

The main Tauri capability declares no direct renderer permissions. The opener
plugin is initialized in Rust, but shared/frontend code is prohibited from
calling frontend plugin APIs; it must use the validated app command.

The CSP permits application assets, Tauri IPC, and local development HMR. It
does not grant the renderer general remote HTTP access. Images are limited to
app/data/blob sources, and media/object/frame embedding is closed down.
Shared task/rich-text components and the web Landing page can contain remote
media URLs, so those assets are unsupported or blocked in the desktop target
until they are routed through an intentional native/cache capability and CSP
review.

Tauri build configuration:

- runs `npm run dev:desktop-ui` before development;
- uses `http://localhost:1420` as the development URL;
- runs `npm run build:desktop-ui` before packaging;
- bundles `web/dist`;
- produces platform bundle targets with the configured icons.

When `TAURI_DEV_HOST` is set, Vite advertises HMR on WebSocket port 1421, while
the current CSP connect list names only localhost/loopback port 1420. Remote-host
HMR is therefore unverified and may require a narrow CSP/config correction.

The Rust bridge reads `VITE_CURIO_SERVICE_URL` at Cargo compile time with
`option_env!` and otherwise uses loopback. The Vite build independently reads
the same name and validates it. Release automation must make the service URL
available to both build processes, not only to client-side Vite substitution.

## End-to-end data flows

### Buffered JSON request

```text
page/component
  → React Query hook
  → resource function in src/api
  → buffered api client
  → selected ServiceTransport
      web: fetch
      desktop: invoke → Rust proxy → reqwest → Channel
  → HTTP status + bytes
  → JSON parse / ApiError
  → query cache or mutation result
```

### Streaming chat

```text
Chat page/useChat
  → streamChat + ChatStreamSession
  → selected raw-byte transport
  → POST /v1/chat/stream
  → PostgreSQL pending records
  → OpenAI Responses API SSE
  → normalized Curio token/done/error SSE
  → raw bytes through browser or Tauri
  → shared TypeScript SSE parser and correlation checks
  → incremental assistant message state
```

### Calendar range and create

```text
EventCalendar visible range
  → useCalendarEvents
  → GET /v1/calendar/events
  → bearer owner check
  → range/date validation
  → half-open PostgreSQL overlap query
  → CalendarEventRecord
  → TaskItem adapter
  → calendar, list, or kanban projection
```

Root-level creation travels back through the same layers with
`POST /v1/calendar/events`, then invalidates every cached range for that user.

### Task create and list

```text
authenticated service request
  → POST/GET /v1/tasks
  → bearer owner check
  → task validation/defaults
  → TaskRepository with bound static SQL
  → tasks
```

The active clients do not currently call these routes; the Calendar Tasks tab
continues to project `calendar_events`.

### External links

- Calls made through `PlatformServices.openExternalUrl` validate HTTP/HTTPS on
  web and invoke Rust's credential-free HTTPS policy on desktop.
- Several current task-card and rich-text anchors bypass that capability and
  rely on ordinary browser/WebView navigation. The native policy is therefore
  not yet enforced for every rendered link.

## Legacy Cloudflare Workers

`curio-workers` is retained because it contains an attachment and embedding
pipeline not yet ported to the active service. It is not referenced by the
frontend or Tauri code.

The legacy topology is:

```text
chat Worker
  → Gemini streaming response
  → D1 conversations/messages/attachment BLOBs
  → optional Cloudflare Queue
  → processor Worker
  → Gemini embeddings
  → D1 embeddings table
```

- `curio-chat-worker` accepts JSON or multipart chat input, calls Gemini,
  streams normalized SSE, persists messages/assets, and publishes asset/message
  identifiers to a queue.
- `curio-processor-worker` consumes batches, loads messages/assets, generates
  embeddings, stores JSON embedding responses, and retries failed queue
  messages.
- Both bind the same D1 database and share `curio-workers/migrations`.
- Each Worker also contains runtime schema-initialization statements used by
  tests; those duplicate migration definitions and must be kept aligned while
  the code remains.
- Each has its own npm scripts, Wrangler configuration, type checks, and
  Cloudflare Vitest tests.

The root CI workflow does not run Worker tests. Treat this tree as migration
source/reference, not a fallback automatically used when `curio-service`
fails.

## Configuration

### Active service

| Variable | Scope | Requirement/default |
| --- | --- | --- |
| `OPENAI_API_KEY` | Service only; secret | Required |
| `OPENAI_MODEL` | Service only | Required; `gpt-5.6` is the default the project ships with |
| `OPENAI_BASE_URL` | Service only | Defaults to `https://api.openai.com` |
| `DATABASE_URL` | Service runtime and `curio_db verify`; secret | Required application-role PostgreSQL URL |
| `CURIO_MIGRATOR_DATABASE_URL` | `curio_db migrate`; secret | Required migrator-role PostgreSQL URL |
| `CURIO_DB_SCHEMA` | Service and database commands | Required validated schema such as `curio_dev` or `curio_prod` |
| `CURIO_DB_MAX_CONNECTIONS` | Service pool | Code default `10`; copied local example sets `5`; bounded from 1 through 50 |
| `CURIO_DB_ACQUIRE_TIMEOUT_SECONDS` | Service pool | Defaults to `10`; bounded from 1 through 60 |
| `CURIO_TEST_DATABASE_URL` | Service integration tests; secret | Test-runner URL; must never target production |
| `CURIO_CORS_ALLOWED_ORIGINS` | Service only | Exact comma-separated origins; local defaults |
| `CURIO_SERVICE_ADDR` | Service process | Explicit bind; otherwise Railway `PORT`, then `127.0.0.1:3000` |
| `RUST_LOG` | Service diagnostics | Optional `tracing_subscriber` filter; defaults to `curio_service=info` |

### Clients and desktop build

| Variable | Scope | Behavior |
| --- | --- | --- |
| `VITE_CURIO_SERVICE_URL` | Web/Vite and desktop compile | Development defaults to loopback; production build must be explicit HTTPS |
| `CURIO_ALLOW_INSECURE_LOCAL_BUILD=1` | Build-time escape hatch | Allows an HTTP service only for deliberate local/debug artifacts |
| `TAURI_DEV_HOST` | Desktop development | Configures Vite host/HMR when developing through another host |

Never put OpenAI or other provider secrets in Vite variables. Vite variables and
the compiled service URL are public artifact configuration.

The Vite development server uses fixed port 1420 and ignores `src-tauri`
changes. Production builds fail if the service URL is absent or non-HTTPS unless
the explicit insecure-local escape hatch is set.

Two current configuration traps matter during development and release:

- `ServiceConfig` defaults include local origins on ports 5173 and 1420, but
  copying the current `curio-service/.env.example` overrides the list with only
  port 5173. Because Vite actually uses 1420, browser development needs that
  exact origin added to `CURIO_CORS_ALLOWED_ORIGINS`.
- `build:web` runs Vite mode `web` and `build:desktop-ui` runs mode `desktop`.
  Vite therefore loads `.env.web*` or `.env.desktop*` (plus common `.env*`),
  not `.env.production`, despite the current `.env.production.example` name.
  A full desktop build also needs `VITE_CURIO_SERVICE_URL` in the shell/CI
  environment seen by Cargo, because Vite env-file loading does not export it
  to Rust's `option_env!` compilation.

### Legacy Workers

The Worker projects use Cloudflare bindings/secrets such as `GEMINI_API_KEY`,
`CURIO_DB`, `CURIO_QUESTION_QUEUE`, and the chat Worker's allowed-origin
configuration. These are unrelated to active client configuration.

## Builds and deployment

### Frontend and desktop commands

Run from `web`:

| Command | Artifact/check |
| --- | --- |
| `npm run dev:web` | Browser-mode Vite server |
| `npm run dev:desktop` | Tauri plus desktop-mode Vite server |
| `npm run build:web` | Strict TS build plus web static bundle |
| `npm run build:desktop-ui` | Strict TS build plus desktop-target UI bundle |
| `npm run build:desktop` | Full Tauri packaging flow |
| `npm run check:architecture` | Source-boundary enforcement |
| `npm run lint` | ESLint |
| `npm test` | Frontend Vitest |

Web and desktop TypeScript configs mirror Vite's router/platform aliases, so
both target implementations are checked independently.

Both UI build commands write `web/dist`; the last build wins. Never publish a
desktop-mode `dist` as the web artifact. The Vite config has no non-root `base`
and `BrowserRouter` has no basename, so the current web artifact assumes
origin-root hosting in addition to requiring SPA fallback for deep links.

Production web hosting must provide SPA fallback. Desktop release distribution
still requires platform signing/notarization and, if introduced later, signed
auto-update metadata.

### Service commands

Run from `curio-service`:

```bash
cargo run --bin curio_db -- migrate
cargo run --bin curio_db -- verify
cargo run --bin curio-service
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Run `curio_db migrate` with the schema-owner credential before starting a new
service release. Railway declares this as its pre-deploy command and probes
`/ready`; ordinary service startup uses the application credential and only
verifies migration state.

`curio-service/ops/postgres/bootstrap_roles.sql` is an operator-reviewed role
and schema bootstrap script; it is not an application migration and must not be
run blindly. `ops/postgres/verify.sql` prints identity, catalog, constraints,
indexes, counts, and possible data violations after migration/import. Its
violation queries are reports rather than automatic failures, so an operator or
wrapper must review/enforce nonzero results.

### CI coverage

`.github/workflows/shared-clients.yml` has two jobs:

- Frontend: Node 22, npm clean install, architecture check, lint, Vitest, web
  build, desktop UI build, and Tauri Rust format/clippy/tests.
- Service: an ephemeral PostgreSQL 18 container, Rust format, all-feature clippy
  with warnings denied, a default-dependency check that rejects `sqlx-sqlite`,
  and PostgreSQL tests.

CI does not currently:

- run the legacy Worker packages;
- exercise Railway networking/roles from pull-request code;
- run browser or packaged-desktop end-to-end tests;
- produce/sign a desktop bundle;
- deploy the service or SPA.

## Test architecture

### Frontend

Vitest covers:

- route-manifest target/root behavior;
- web and desktop transport status/chunks/query/auth/cancellation/error behavior;
- JSON API success/error/empty-body handling;
- Calendar resource requests and date-string preservation;
- chat SSE fragmentation, UTF-8 boundaries, terminal rules, correlation, and
  transport failures;
- local auth migration/normalization;
- theme color math, palette/source consistency, contrast, storage, and custom
  theme rules;
- per-user Home thought storage;
- calendar classification/edit-date invariants and task-list/kanban adapters;
- Notes lookup/data invariants.

These are primarily protocol and pure-domain tests. There is no browser-level
route/page interaction suite. Vitest runs in its default Node environment, with
no DOM component runner, browser runner, or configured coverage command.

### Tauri Rust

Unit tests live under `src-tauri/tests/unit` and are included into private
modules with `#[path]`. They cover service request packet types, URL/path and
bearer validation, external-link policy, and cancellation-registry behavior.
They do not launch a WebView or a real Tauri IPC integration environment.

### Axum service

Service unit tests use the same private-module inclusion pattern. Database-backed
tests read `CURIO_TEST_DATABASE_URL`, reject production-looking targets, create
a unique disposable `curio_test_<run-id>` PostgreSQL schema, apply the PostgreSQL
migration, and close pools/drop the schema during cleanup. Full API tests use
`tower::ServiceExt::oneshot`; a temporary local Axum server still mocks OpenAI
streams.

When `CURIO_TEST_DATABASE_URL` is absent, database-backed helpers return early
and Cargo still reports the enclosing tests as successful. Local verification
must inspect the output and report PostgreSQL coverage as skipped/partial; CI
provides PostgreSQL 18 and exercises those paths.

Coverage includes:

- liveness, database readiness, and placeholder auth;
- user upsert and validation;
- OpenAI provider SSE parsing and sanitized errors;
- normalized chat streaming plus database history;
- chat persistence after closing and reopening a pool on the same disposable
  schema;
- calendar authentication/ownership, create/list, UUIDs, range overlap,
  all-day/timed/milestone semantics, DST slack, and error statuses.
- task authentication/ownership, validation/defaults, create/list ordering,
  injection-shaped values, native PostgreSQL types, and reconnect durability.

Normal database-backed tests and runtime remain PostgreSQL-only. Material gaps
include production auth, user ownership, chat history
authorization/pagination, retries/idempotency, pending-message recovery,
calendar update/delete, production pool/load characterization, operational
middleware, and end-to-end client/service tests.

## Security and trust boundaries

### Enforced boundaries

- Provider secrets stay in `curio-service`.
- Shared frontend code cannot use browser network primitives or Tauri APIs.
- Desktop renderer requests can target only the compiled service origin through
  validated relative paths.
- Desktop bearer tokens are validated as header values and marked sensitive.
- External URLs sent through the desktop Rust command require HTTPS, a host,
  and no embedded credentials.
- Tauri CSP/capabilities keep the renderer from directly acquiring broad native
  or remote-network access.
- Service CORS reflects only exact configured web origins.
- OpenAI errors are sanitized before becoming Curio SSE events.
- Calendar and Task data are checked against the bearer-derived owner ID.
- `ServiceConfig` uses a custom `Debug` implementation that redacts the OpenAI
  key and database URL.

### Current trust gaps

- Client login is mock local storage, not real authentication.
- The service treats bearer contents as the user ID.
- Assistant stream/history routes are public and conversations have no owner.
- User upsert does not enforce bearer ID equals body ID.
- CORS is not an authorization mechanism.
- Frontend route guards protect navigation only.
- Some shared link/media renderers bypass `PlatformServices.openExternalUrl`,
  so the Rust desktop URL policy is not universal; some rendered rich-text/task
  URLs also lack an equivalent HTTP/HTTPS protocol allowlist.
- The web transport does not independently enforce same-origin paths at
  runtime; current API callers use code-owned constants.
- There is no service rate limiting, explicit project-level body-size policy,
  structured audit log, or request correlation.

Do not describe the current system as production-authenticated until those gaps
are addressed.

## Current architectural constraints and debt

1. Authentication and ownership are incomplete and inconsistently applied.
2. Chat UI starts from demo state and does not hydrate persisted history.
3. Chat conversations are global; the frontend `userId` is ignored by the
   backend chat request.
4. Stored conversation history is not sent back to OpenAI as context.
5. Calendar is the most complete handler/service/repository/error vertical
   slice; Assistant and User keep persistence in typed feature repositories.
6. Calendar only supports create and bounded list, so persisted entries must
   remain read-only in edit surfaces.
7. The Calendar “Tasks” views are projections of calendar events even though an
   independent task service resource now exists; there is no frontend task
   client or projection yet.
8. Notes, Vault, Atlas, Profile Setup, and most dashboard/settings data are
   mock, local, or in-memory.
9. JSON/API error shapes are not uniform across auth, Axum extraction, history,
   feature errors, and SSE.
10. Service startup couples every feature to valid OpenAI configuration.
11. Operational server concerns—graceful shutdown, request-wide tracing and
    request IDs, HTTP timeouts, rate limits, and production connection
    monitoring—are minimal.
12. Legacy Worker functionality, especially attachments/embeddings, is not
    active and is outside root CI.
13. There is no cross-stack end-to-end test suite.
14. JSON contracts are mirrored manually and decoded with TypeScript assertions,
    without OpenAPI/code generation or runtime response schemas.
15. `api/calendar.ts` imports status/priority types from a calendar UI module,
    so the current resource layer has an upward type dependency that cuts across
    the intended API-to-UI direction.
16. Calendar-mode fetch/create failures are not surfaced consistently, and
    Tasks projections cover only the most recently visible calendar range.
17. Direct shared link/media rendering bypasses the platform capability, while
    desktop CSP blocks remote media that can appear in shared/web content.
18. Web and desktop UI builds overwrite the same `web/dist`, and current mode
    files/Cargo environment propagation require release discipline.
19. The checked-in local CORS example omits Vite's actual port 1420.

This list is a guide for planning, not permission to mix unrelated refactors
into feature work.

## Extension rules

### Add a page

1. Add the route metadata once in `route-manifest.ts`.
2. Add the lazy page mapping in `router.tsx`.
3. Reuse the common shell where appropriate.
4. Keep target differences in metadata or platform capabilities, not duplicated
   page trees.
5. Add route-manifest tests when access or target behavior changes.

### Add a JSON service resource

1. Prefer a feature-local Axum vertical slice modeled on Calendar.
2. If the persistence schema changes, add an append-only PostgreSQL SQLx
   migration under `migrations/postgres`; an HTTP-only resource change does not
   need a no-op migration.
3. Keep SQL in the repository, validation/defaults in the service, and
   extractors/status/envelopes in handlers.
4. Compose a feature router at `app_with_config`.
5. Define one frontend resource module under `src/api`.
6. Add a React Query hook when cache/ownership/UI adaptation is needed.
7. Test service rules, full-router HTTP behavior, platform-client contract, and
   persistence across reconnect where durability matters.

### Add streaming behavior

1. Keep domain framing/parsing in target-independent TypeScript when both
   clients consume the same byte protocol.
2. Keep platform adapters byte-oriented.
3. Require one explicit terminal event and validate request/event correlation.
4. Define cancellation and partial-persistence behavior before implementation.

### Add native functionality

1. Extend `PlatformServices` only for a real browser/desktop difference.
2. Implement both target behaviors or make target support explicit.
3. Put OS/security validation in Tauri `core/services`.
4. Expose a narrow app-owned command.
5. Review CSP, Tauri capabilities, architecture gates, and unit tests.

### Add a reusable UI system

1. Keep its data model and controlled state independent of API clients.
2. Export a deliberate public barrel.
3. Put product adapters in a feature folder outside the reusable system.
4. Consume semantic theme tokens.
5. Preserve keyboard/accessibility and permission contracts in tests.

### Change persisted calendar/task-like dates

Treat wire format as domain data. Preserve date-only versus timestamp values,
normalize separate query instants on the service, and test all-day, timed,
milestone, range-boundary, and timezone behavior.

For future get/update/delete operations, derive ownership from authenticated
identity and scope SQL by both owner and resource ID. Define PATCH
omitted-versus-null semantics explicitly. When any date-category field changes,
merge it with stored state, re-run cross-field validation, and recompute the
normalized interval atomically.

Also define the mutable-field allowlist, immutable identifiers/ownership/audit
fields, empty-patch behavior, missing-versus-not-owned response, and concurrency
policy. Update `updated_at` while preserving `created_at`, and return the
committed owner-scoped row rather than reconstructing a response from input.

For frontend calendar persistence, keep React Query as the controlled source of
truth, choose and test an explicit optimistic or pessimistic reconciliation
strategy, and reconcile/invalidate the full user calendar query prefix because
an edit can move a record between cached visible ranges. Enable only UI
permissions covered by the implemented service operations, and prove one
logical edit emits one request.

## Invariants to preserve

- One React source tree serves web and desktop.
- Vite aliases and matching TS configs select platform/router implementations.
- Shared components and pages do not access network or Tauri APIs.
- Resource URLs live in `src/api`; chat streaming remains behind its typed
  protocol transport.
- Desktop service requests remain relative and same-origin.
- Rust desktop code transports raw response bytes and does not duplicate the
  frontend SSE parser.
- Provider secrets never enter client builds.
- PostgreSQL migrations run explicitly before deployment, startup verifies the
  expected version, and the default service build contains no SQLite driver.
- Archived SQLite migrations remain isolated from active request handling; no
  SQLite importer or driver is active.
- New service features follow Calendar's feature-local layering.
- Calendar public date strings retain their all-day/timed meaning.
- The legacy Workers remain explicitly disconnected unless a planned migration
  changes the active topology and all documentation/tests together.

## Related documentation

- [`../README.md`](../README.md): short project overview and local commands.
- [`../web/README.md`](../web/README.md): frontend target and transport summary.
- [`deployment.md`](deployment.md): environment, hosting, signing, and release
  notes.
- [`../AGENTS.md`](../AGENTS.md): repository-wide implementation and
  verification rules.
- [`../.agents/skills`](../.agents/skills): project-specific React/Vite, Axum,
  and verification workflows.
- [`../plans`](../plans): historical or proposed implementation plans, not
  authoritative descriptions of active runtime behavior.
