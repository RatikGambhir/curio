# Curio application architecture

## One frontend, two artifacts

`web/src` is the only maintained product UI. Vite resolves the stable
`@curio/platform-runtime` import to the target platform adapter and
`@curio/router-runtime` to the target router entry point. Web mode selects
`src/platform/web.ts` with `src/app/router.web.tsx`; desktop mode selects
`src/platform/desktop.ts` with `src/app/router.desktop.tsx`. Shared hooks and
components never import Tauri APIs directly; they call the typed API layer,
which uses the build-selected platform runtime. There is no runtime platform
context or provider.

| Layer | Responsibility |
| --- | --- |
| `src/app` | Shared route manifest, target router choice, and access guards |
| `src/api` | Typed JSON client and per-resource endpoints over the platform transport |
| `src/features` | Target-independent auth and chat types/protocol logic |
| `src/components`, `src/pages`, `src/hooks` | Shared product UI and state |
| `src/platform/contracts.ts` | The `ServiceTransport` interface seen by shared code |
| `src/platform/web.ts` | Browser HTTP transport and browser URL opening |
| `src/platform/desktop.ts` | App-owned Tauri command/channel transport and cancellation IPC |
| `src-tauri` | Rust HTTP bridge, validated native integrations, and packaging |

`tsconfig.web.json` and `tsconfig.desktop.json` mirror those aliases during
target-specific type checking. `BrowserRouter` is selected for web builds.
`HashRouter` is selected for desktop builds so bundled static assets do not need
a deep-link server. Both targets use the same route manifest and guards. The web
root renders the landing page; the desktop root redirects to login or home.

## One service transport

Every request to the Curio service—streaming chat or plain JSON—flows through
the single `ServiceTransport` primitive in `src/platform/contracts.ts`:

- The web adapter executes requests with the browser HTTP stack against the
  compiled `VITE_CURIO_SERVICE_URL`, reporting the status before streaming raw
  body chunks.
- The desktop adapter invokes the app-owned Rust `service_request` command with
  a validated method (`GET`, `POST`, `PATCH`, `DELETE`), a relative path, a JSON
  payload, and an optional bearer token. Rust performs the HTTP request with a
  shared connection pool and forwards `started`, base64 `chunk`, `end`, and
  transport `error` packets over a typed Tauri `Channel`. Rust does not
  understand SSE; it only moves bytes.
- `cancel_request` cancels the Rust request through managed in-flight state, and
  every completion/error/channel-close path removes that state. On web,
  cancellation is the caller's `AbortSignal`.

On top of the transport, shared code provides two clients:

- `src/api/client.ts` buffers responses and exposes typed JSON helpers
  (`api.get/post/patch/delete`), mapping error bodies to `ApiError`. Resource
  modules such as `src/api/users.ts` define the endpoint surface; features and
  hooks (for example `useSaveUser`, a React Query mutation) never construct URLs
  or touch network primitives.
- `src/features/chat/transport.ts` streams `/v1/chat/stream` responses into the
  shared `ChatStreamSession`, so SSE parsing, correlation checks, UTF-8 framing,
  terminal-event handling, and protocol errors have one TypeScript
  implementation for both targets. The shared `useChat` hook owns request IDs,
  cancellation, and UI state.

## Backend

`curio-service` (Axum + SQLite) is the backend for both targets. It serves the
chat stream (`POST /v1/chat/stream`, normalized SSE `token`/`done`/`error`
events backed by the OpenAI Responses API), conversation history
(`GET /v1/conversations`, `GET /v1/conversations/{id}/messages`), and the
authenticated user API (`POST /v1/users`, bearer-protected upsert). CORS allows
only configured web origins; the desktop client needs no CORS because requests
originate from the Rust process.

The Cloudflare workers under `curio-workers` are legacy: the clients no longer
reference them.

## Security boundaries

ESLint and `npm run check:architecture` enforce that `@tauri-apps/*` imports are
limited to the desktop adapter, frontend Tauri plugins cannot bypass app-owned
commands, browser network APIs stay in the web adapter, concrete platform
modules are selected only through the build alias, the platform runtime is only
consumed by `src/api` and `src/features/chat`, and a second desktop React tree
does not reappear. The desktop `service_request` command accepts relative paths
only, so renderer input cannot select another origin, and the bearer token is
restricted to the Authorization header. The renderer has no direct opener
permission; Rust validates HTTPS external URLs before asking the operating
system to open them. The service reflects CORS only for configured origins;
local Vite origins are the development default.

Rust remains native-only because there is currently no substantial pure Rust
client domain logic worth compiling to WebAssembly. A future deterministic,
CPU-heavy feature can be extracted into a platform-neutral crate when there is
a measured use case.
