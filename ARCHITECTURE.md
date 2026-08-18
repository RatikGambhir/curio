# Curio application architecture

## One frontend, two artifacts

`web/src` is the only maintained product UI. Vite resolves the stable
`@curio/platform-runtime` import to `src/platform/web.ts` in web mode and to
`src/platform/desktop.ts` in desktop mode. Shared hooks and components never
import Tauri APIs directly; they consume `PlatformServices` through the platform
provider.

| Layer | Responsibility |
| --- | --- |
| `src/app` | Shared route manifest, target router choice, and access guards |
| `src/features` | Target-independent auth and chat types/protocol logic |
| `src/components`, `src/pages`, `src/hooks` | Shared product UI and state |
| `src/platform/contracts.ts` | Narrow interface seen by shared code |
| `src/platform/web.ts` | Browser fetch and browser URL opening |
| `src/platform/desktop.ts` | Tauri channels, cancellation IPC, and native URL opening |
| `src-tauri` | Thin Rust HTTP bridge and native packaging |

`BrowserRouter` is selected for web builds. `HashRouter` is selected for desktop
builds so bundled static assets do not need a deep-link server. Both targets use
the same route manifest and guards. The web root renders the landing page; the
desktop root redirects to login or home.

## Shared chat streaming

The shared `useChat` hook creates request/message IDs, owns cancellation and UI
state transitions, and calls the injected `ChatTransport`.

- The web adapter posts directly to `/v1/chat/stream` with `fetch` and the
  caller's `AbortSignal`.
- The desktop adapter invokes `stream_chat` with a typed Tauri `Channel`.
- Rust forwards the request and emits only `started`, raw `chunk`, `end`, or
  transport `error` packets. It does not understand SSE events.
- Both paths feed bytes into `ChatStreamSession`, so correlation checks, UTF-8
  framing, terminal-event handling, and protocol errors have one TypeScript
  implementation.
- `cancel_chat` cancels the Rust request through managed in-flight state, and
  every completion/error/channel-close path removes that state.

The Cloudflare chat worker streams Gemini output and persists messages in D1.
The processor worker consumes the queue and writes embeddings to the same D1
database. `curio-service` is deliberately outside this runtime path.

## Security boundaries

ESLint and `npm run check:architecture` enforce that `@tauri-apps/*` imports are
limited to the desktop adapter and that a second desktop React tree does not
reappear. The Tauri shell uses a restrictive CSP and exposes only scoped HTTPS
URL opening in its capability file. The worker reflects CORS only for configured
origins; local Vite origins are the development default.

Rust remains native-only because there is currently no substantial pure Rust
client domain logic worth compiling to WebAssembly. A future deterministic,
CPU-heavy feature can be extracted into a platform-neutral crate when there is
a measured use case.
