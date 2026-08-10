# Curio shared desktop and web implementation plan

Status: proposed  
Prepared: 2026-08-09  
Scope: consolidate the existing React/Vite web and Tauri desktop clients into one frontend codebase while retaining a thin, explicit Rust/native boundary.

## Executive decision

Curio should have **one React/Vite application that is built in two modes**:

1. A normal static web SPA for browsers.
2. The same SPA bundled into a Tauri desktop shell.

The existing `web/curio` application should become the canonical frontend because it is the more complete client: it contains the landing page, Atlas, React Query, the tested browser SSE parser, cancellation support, and the larger UI dependency set. The Tauri `src-tauri` shell should move beside that frontend. Once feature parity is proven, the duplicate React source under `desktop/Curio/src` should be removed.

Rust should remain behind a platform adapter. Browser code cannot call Tauri commands, so the shared React application must depend on a TypeScript interface rather than importing `@tauri-apps/api` throughout the app. The browser implementation uses standard web APIs; the desktop implementation uses Tauri commands/channels where native behavior is useful.

For the current chat flow, preserve the active Cloudflare Worker/D1 architecture. Do **not** combine this consolidation with a migration back to `curio-service`. The optional Axum service is a separate backend decision.

## Why this is the right shape for the current repository

The current worktree already has both product targets, but not a shared application:

- `web/curio/src` and `desktop/Curio/src` are independent React trees.
- They have 62 files at matching relative paths, and most of those files have already drifted.
- The dependency versions have also drifted, including React, React Router, Tailwind, Radix packages, and Lucide.
- The web chat path parses `/v1/chat/stream` in TypeScript and supports `AbortController` cancellation.
- The desktop chat path calls `stream_chat` through Tauri and contains another SSE parser in Rust.
- `@tauri-apps/api` is currently imported only by `desktop/Curio/src/hooks/useChat.ts`, so the native boundary is small and can be isolated cleanly.
- Both clients ultimately talk to the same Cloudflare chat worker and D1 database in the current worktree.
- Authentication is still local mock state and is not production-ready. Consolidation should share its implementation but should not mistake it for real cross-device authentication.

The main problem is therefore code ownership and platform boundaries, not the lack of a second build target.

## Target architecture

Keep the first migration mechanically small by using `web/curio` as the unified app instead of renaming the whole repository at the same time:

```text
curio/
├── web/curio/                         # The one frontend application
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── app/                       # Providers, router, target-aware startup
│   │   ├── features/                  # Chat, auth, vault, atlas, settings
│   │   ├── components/                # Shared UI
│   │   ├── platform/
│   │   │   ├── contracts.ts           # Interfaces consumed by shared code
│   │   │   ├── web.ts                 # Browser implementations
│   │   │   └── desktop.ts             # Tauri implementations only
│   │   └── ...
│   └── src-tauri/                     # Moved from desktop/Curio/src-tauri
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       └── src/
├── curio-workers/                     # Existing active backend
├── curio-service/                     # Optional/reference backend; unchanged here
└── plans/
```

A later cleanup may rename `web/curio` to `apps/curio`, but that rename should not be mixed into the functional consolidation.

The runtime flow becomes:

```text
                         Shared React app
                  UI, routing, auth, chat state
                                │
                         Platform contract
                         /               \
                browser adapter      desktop adapter
                fetch/Web APIs       Tauri IPC/channel
                         \               /
                          Cloudflare Worker
                           Gemini + D1 + queue
```

## Platform contract

Create a small contract in `web/curio/src/platform/contracts.ts`. Shared components and hooks may import this contract, but they must not import a concrete platform module.

The initial surface should be intentionally narrow:

```ts
export type AppTarget = "web" | "desktop"

export interface ChatTransport {
  stream(
    request: ChatStreamRequest,
    options: {
      signal: AbortSignal
      onEvent: (event: ChatStreamEvent) => void
    },
  ): Promise<ChatStreamDoneEvent | ChatStreamErrorEvent>
}

export interface PlatformServices {
  target: AppTarget
  chat: ChatTransport
  openExternalUrl(url: string): Promise<void>
}
```

Add capabilities only when a real native/browser difference appears. Likely future additions are file picking, secure credential storage, notifications, auto-update, and filesystem access. Do not create generic wrappers for browser APIs that work identically in both builds.

Select the concrete implementation at **build time**, not by sprinkling `window.__TAURI__` checks around the application. Configure Vite to alias a stable import such as `@curio/platform-runtime` to either `src/platform/web.ts` or `src/platform/desktop.ts` based on the build mode. This has three benefits:

- The regular web bundle does not accidentally execute or depend on Tauri APIs.
- Missing platform methods fail at compile time.
- Platform-specific code stays searchable and reviewable.

Enforce this boundary with an ESLint `no-restricted-imports` rule: only `src/platform/desktop.ts` may import `@tauri-apps/*`.

## Rust sharing policy

There are two different meanings of “sharing Rust,” and they should not be conflated:

1. **Tauri Rust/native logic:** available only in the desktop build through Tauri IPC. Hide it behind the desktop adapter.
2. **Pure Rust business logic that must run in both browser and desktop:** extract it into a platform-neutral crate and compile it both natively and to WebAssembly.

The current Curio code does not yet contain enough pure client-side Rust domain logic to justify WebAssembly. Chat networking, UI state, routing, and local mock auth are better shared in TypeScript. Introducing WASM now would add another build pipeline without removing meaningful duplication.

If a future feature has substantial deterministic logic—such as local ranking, document parsing, or a CPU-heavy transformation—use this optional shape:

```text
crates/curio-core/       # No tauri, reqwest, tokio, sqlx, or OS APIs
  src/lib.rs
  tests/
web/curio/src-tauri/     # Native consumer of curio-core
web/curio/src/wasm/      # Browser wrapper around curio-core's WASM build
```

Treat WASM as a later, evidence-based optimization. It is not required to achieve one shared application.

## Chat transport design

Chat is the only meaningful native boundary today and should be migrated first because it proves the architecture.

### Shared chat code

Make the following code target-independent and keep it in the canonical frontend:

- Request and event types.
- SSE framing/parser logic from `web/curio/src/lib/chat-stream.ts`.
- Correlation validation.
- Message creation and state transitions.
- Error normalization.
- Cancellation behavior.
- The `useChat` hook.

Refactor `useChat` to accept the injected `ChatTransport`. The hook should not know whether bytes arrived from `fetch` or Tauri IPC.

### Web implementation

The web adapter should retain the current tested approach:

- `fetch` `POST /v1/chat/stream` using `VITE_CURIO_CHAT_WORKER_URL`.
- Feed response bytes into the shared SSE parser.
- Use the caller's `AbortSignal` for cancellation.
- Return the normalized terminal `done` or `error` event.

### Desktop implementation

The desktop adapter should call a Tauri command and receive stream packets through a Tauri `Channel`. Current Tauri guidance recommends channels for streaming data; the existing global event names should be removed from this flow.

The Rust command should be a thin HTTP bridge:

- Accept a `requestId`, the chat request, and a typed channel.
- Use `reqwest` to call the same Cloudflare worker endpoint.
- Send a `started` packet with the HTTP status, then raw byte chunks, followed by `end` or a typed transport error.
- Do not parse the SSE domain protocol in Rust. The shared TypeScript parser should parse it for both targets.
- Track in-flight requests in managed Tauri state so `cancel_chat(requestId)` can abort the HTTP request when the frontend's `AbortSignal` fires.
- Remove the in-flight entry on every success, error, cancellation, and channel-close path.

This removes the current TypeScript/Rust SSE-parser duplication while still using Rust where it adds native value. Rust tests then cover HTTP forwarding, status propagation, cancellation, and cleanup; TypeScript tests cover SSE protocol semantics once.

If testing shows that direct WebView `fetch` is fully reliable on every supported desktop platform, the desktop adapter may delegate to the web transport and the Rust bridge can be deleted. The contract allows that simplification without touching feature code.

## Routing and target-specific product behavior

Use one route manifest and choose only the router implementation per target:

- Web: `BrowserRouter`, with the deployed host configured to fall back to `index.html` for SPA routes.
- Desktop: `HashRouter`, avoiding deep-link resolution problems inside the bundled static app.

Expected route behavior:

| Route | Web | Desktop |
|---|---|---|
| `/` | Landing page | Redirect to `/login` or `/home` |
| `/login` | Shared | Shared |
| `/home` | Shared, authenticated | Shared, authenticated |
| `/chat` | Shared, authenticated | Shared, authenticated |
| `/vault` | Shared, authenticated | Shared, authenticated |
| `/atlas` | Shared, authenticated | Shared, authenticated |
| `/profile`, `/settings` | Shared, authenticated | Shared, authenticated |
| `/desktop-construction` | Remove after cutover | Remove after cutover |

Represent availability as route metadata or a platform capability. Do not fork the entire router.

## Build and package changes

Merge the Tauri dependencies and scripts into `web/curio/package.json`. Use separate script names so Tauri hooks do not recurse:

```json
{
  "scripts": {
    "dev:web": "vite --mode web",
    "build:web": "tsc -b && vite build --mode web",
    "dev:desktop-ui": "vite --mode desktop",
    "build:desktop-ui": "tsc -b && vite build --mode desktop",
    "dev:desktop": "tauri dev",
    "build:desktop": "tauri build",
    "test": "vitest run",
    "lint": "eslint ."
  }
}
```

After moving `src-tauri` into `web/curio`, configure Tauri with:

- `beforeDevCommand`: `npm run dev:desktop-ui`
- `beforeBuildCommand`: `npm run build:desktop-ui`
- `devUrl`: the fixed desktop Vite port
- `frontendDist`: `../dist`

Merge the existing `TAURI_DEV_HOST`, strict-port, HMR, and `src-tauri` watch exclusions into the canonical Vite config. Keep the normal web server configuration independent so web development is not forced onto the Tauri port.

The worker URL is public configuration, not a secret. Give both modes an explicit production default and fail clearly when a production build has no valid URL. A packaged desktop app should not depend on the developer launching it with `CURIO_CHAT_WORKER_URL` in the shell.

## Source consolidation rules

Do not bulk-copy one source tree over the other. The worktree contains active, uncommitted changes in both trees. Merge feature by feature and use `web/curio` as the default winner while preserving intentional desktop behavior.

Recommended ownership decisions:

- Keep the web versions of Landing, Atlas, animations, React Query setup, chat cancellation, and SSE tests.
- Preserve desktop loading/disabled behavior where it prevents double submissions.
- Consolidate auth into one provider, one user type, one hook, and one storage schema.
- Consolidate `ChatMessage`, `ChatListItem`, and related types outside mock-data files.
- Keep one `index.css`; manually reconcile token differences rather than accepting formatting-only drift.
- Align on one set of dependency versions before comparing component behavior.
- Move mock data behind feature-local test/demo modules so production state is not coupled to `mocks/chats.ts`.

## Phased implementation

### Phase 0 — Checkpoint and baseline

1. Checkpoint the current dirty worktree before moving files. The present branch is ahead of `origin/main` and contains a large uncommitted worker/auth/UI change set.
2. Record the current commands and results for web build, web tests/lint, desktop frontend build, and Rust tests.
3. Capture a route/feature smoke checklist for login, home, chat, vault, Atlas, settings, and logout.
4. Capture representative web and desktop screenshots so visual drift is detectable.
5. Write down the backend decision: Cloudflare Worker/D1 remains active; `curio-service` is out of scope.

Exit criteria: there is a recoverable checkpoint and a known baseline, including any pre-existing failures.

### Phase 1 — Add target and platform abstractions in the web app

1. Add `AppTarget`, `PlatformServices`, and `ChatTransport` contracts.
2. Add build-time Vite aliases for web and desktop runtime modules.
3. Add a platform provider or a single injected service object at application bootstrap.
4. Add the import-boundary lint rule.
5. Refactor external URL opening and any other existing native usage behind the contract.
6. Add unit tests that run the same contract tests against both adapters where practical.

Exit criteria: shared feature code has no direct Tauri imports and both runtime modules type-check.

### Phase 2 — Unify chat behavior

1. Move chat protocol types and parsing into a shared feature module.
2. Refactor the existing web `useChat` into target-independent state logic.
3. Implement the web `ChatTransport` using the current fetch path.
4. Replace Tauri global stream events with a channel-based raw-byte bridge.
5. Implement desktop cancellation and in-flight request cleanup.
6. Implement the desktop `ChatTransport` and run the same hook/component tests against it with a mock adapter.
7. Remove the Rust SSE parser only after protocol fixture and end-to-end tests pass.

Exit criteria: the same React chat page and hook stream successfully in browser and Tauri, cancellation works in both, and only one SSE parser remains.

### Phase 3 — Merge the application shell and routes

1. Build a shared route manifest from the current web and desktop routers.
2. Use `BrowserRouter` for web and `HashRouter` for desktop.
3. Consolidate auth provider, authenticated-route guards, and user types.
4. Consolidate shared providers such as React Query.
5. Encode only real product differences as target capabilities.
6. Remove the temporary desktop-construction route once the real desktop build uses the shared app.

Exit criteria: all supported routes render from `web/curio/src` in both targets with the expected access rules.

### Phase 4 — Consolidate UI and feature code

Migrate in small groups so regressions are attributable:

1. UI primitives and design tokens.
2. App shell/sidebar/header/navigation.
3. Login/profile setup/authenticated user state.
4. Chat components and message types.
5. Vault.
6. Settings/profile.
7. Home and Atlas.
8. Web-only landing content.

For each group:

- Compare both versions.
- Select or manually merge behavior.
- Add/update focused tests.
- Run both builds.
- Check the route in web and desktop before moving on.

Exit criteria: no product component is maintained in both source trees.

### Phase 5 — Move the Tauri shell and cut over

1. Move `desktop/Curio/src-tauri` to `web/curio/src-tauri`.
2. Merge Tauri npm dependencies into the canonical package.
3. Update Tauri build hooks, `frontendDist`, dev URL, and Vite settings.
4. Run a development desktop build from `web/curio`.
5. Produce a packaged desktop build and verify it uses the production worker URL.
6. Remove `desktop/Curio/src` and the obsolete desktop package only after parity checks pass.
7. Add temporary CI checks that fail if a second React source tree or unauthorized Tauri import reappears.

Exit criteria: `web/curio` is the only frontend package and is the input to both web and desktop artifacts.

### Phase 6 — Security, deployment, and documentation

1. Replace `app.security.csp: null` with a least-privilege CSP that allows the required Tauri IPC and trusted worker connection only.
2. Replace the worker's production `Access-Control-Allow-Origin: *` with an allowlist for deployed web origins. Keep local development origins configurable.
3. Confirm Tauri capabilities allow only the commands/plugins actually used.
4. Document web hosting's SPA fallback and environment variables.
5. Document desktop signing/notarization/update requirements separately from the shared-code migration.
6. Update the root README so local commands start one shared frontend in either mode.

Exit criteria: both artifacts have repeatable production builds and documented deployment configuration.

## Test strategy

### Shared TypeScript tests

- SSE events split across arbitrary byte boundaries, including multibyte UTF-8.
- Multiple events in one chunk.
- Malformed JSON and unsupported event names.
- Mismatched conversation/message identifiers.
- Missing terminal event and events after a terminal event.
- Abort before response, mid-stream abort, and component unmount.
- Hook state transitions for token, done, error, and cancellation.
- Auth storage parsing and invalid-data cleanup.
- Route guards in both target configurations.

### Adapter contract tests

Run a common `ChatTransport` contract suite against mocked web and desktop adapters:

- Emits ordered tokens.
- Returns exactly one terminal event.
- Propagates transport failures using the same error shape.
- Stops emitting after cancellation.
- Cleans up listeners/channels/controllers.

### Rust tests

- Forwards request body and required headers.
- Reports non-success HTTP status.
- Preserves arbitrary response bytes and ordering.
- Stops the upstream request on cancellation or channel close.
- Removes every in-flight request from managed state.
- Does not leak tokens or worker configuration into logs.

### Build and smoke matrix

| Check | Web | Desktop |
|---|---:|---:|
| TypeScript compile | Required | Required |
| ESLint | Required | Same source, required once |
| Vitest | Required | Shared tests plus adapter tests |
| Production frontend build | `build:web` | `build:desktop-ui` |
| Rust fmt/test/clippy | N/A | Required |
| Packaged artifact | Static `dist` | Tauri bundle |
| Login/chat/vault/Atlas/settings smoke | Required | Required |
| Chat cancellation and retry | Required | Required |

## Acceptance criteria

The consolidation is complete when all of the following are true:

- There is exactly one maintained React source tree.
- The web and desktop artifacts are built from that same source tree.
- Shared components/hooks contain no direct `@tauri-apps/*` imports.
- All Tauri imports are isolated to the desktop platform adapter/bootstrap.
- Browser navigation and refresh work on deployed SPA routes.
- Desktop navigation works from a packaged build.
- Web and desktop chat use the same request/event types, parser, hook, and UI state transitions.
- Chat tokens, terminal errors, cancellation, and retry behave consistently in both targets.
- The Rust bridge uses channels rather than global events if the bridge is retained.
- Both clients persist to the same active worker/D1 backend without duplicate messages.
- The desktop package does not require a shell environment variable to find the production backend.
- CSP, Tauri capabilities, and production CORS are no longer permissive defaults.
- Root documentation and CI cover both outputs.

## Risks and mitigations

### Active uncommitted work is overwritten

Mitigation: checkpoint before moving files and merge by feature. Never replace `desktop/Curio/src` wholesale with `web/curio/src` or vice versa.

### The web UI silently becomes the desktop UI without preserving desktop fixes

Mitigation: use a route/feature checklist and visual baselines; explicitly carry over disabled/loading states and native behavior.

### Browser and desktop routing diverge

Mitigation: share one route manifest and vary only `BrowserRouter` versus `HashRouter` plus route availability metadata.

### Streaming cancellation leaks a Rust task

Mitigation: use request IDs, managed cancellation handles, cleanup guards, and tests for every terminal path.

### Platform abstraction becomes a dumping ground

Mitigation: wrap only actual differences. Standard React, fetch, localStorage, and DOM code remains ordinary shared code unless a target proves it needs another implementation.

### Backend migration expands the scope

Mitigation: keep Cloudflare Worker/D1 as the active backend in this plan. Evaluate `curio-service` versus Workers in a separate architecture decision record.

### Adding WASM too early increases complexity

Mitigation: require a measured use case before creating `curio-core` WASM output. Tauri IPC and a shared TypeScript frontend are sufficient for the current product.

## Suggested commit sequence

1. `chore: checkpoint current web desktop and worker state`
2. `refactor(frontend): add build target and platform contracts`
3. `refactor(chat): share chat protocol and state logic`
4. `refactor(desktop): stream chat over tauri channel`
5. `refactor(app): share routes auth and providers`
6. `refactor(ui): consolidate shared feature components`
7. `build(tauri): move desktop shell into canonical app`
8. `chore: remove duplicate desktop frontend`
9. `security: tighten tauri csp capabilities and worker cors`
10. `ci: verify web and desktop artifacts`

Each commit should leave at least one target buildable; after the chat adapter lands, both targets should remain buildable at every step.

## Deliberate non-goals

- Rewriting the Cloudflare Workers in Rust.
- Switching from Gemini to another model provider.
- Replacing D1 or changing the embedding pipeline.
- Implementing production authentication.
- Adding offline-first synchronization.
- Renaming the whole repository layout while behavior is still moving.
- Compiling Rust to WASM without a concrete shared computation.

These may be valuable later, but mixing them into the consolidation would make failures harder to isolate.

## Reference basis

- Tauri treats the bundled frontend as a static host and recommends SPA/SSG/MPA rather than server-rendered frontends: [Frontend Configuration](https://v2.tauri.app/start/frontend/).
- Tauri documents the normal Vite `devUrl`/`frontendDist` integration used by this plan: [Vite](https://v2.tauri.app/start/frontend/vite/).
- Tauri commands are the JavaScript-to-Rust boundary, and channels are the recommended mechanism for streamed HTTP-style data: [Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/).
- Tauri recommends a tailored, restrictive CSP rather than leaving it disabled: [Content Security Policy](https://v2.tauri.app/security/csp/).

