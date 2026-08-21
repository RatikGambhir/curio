# Curio frontend

This package is the canonical React application for both product targets.

```bash
npm run dev:web           # Browser SPA
npm run dev:desktop       # Tauri shell + desktop-mode Vite server
npm run build:web         # Static web artifact
npm run build:desktop-ui  # Static UI consumed by Tauri
npm run build:desktop     # Packaged desktop artifact
```

Production builds require `VITE_CURIO_SERVICE_URL` to be the deployed HTTPS
Curio service (`curio-service`). Development defaults to
`http://127.0.0.1:3000`.

Platform differences belong behind `src/platform/contracts.ts`, which defines a
single `ServiceTransport` primitive. Vite selects the concrete platform and
router runtimes at build time, with matching `tsconfig.web.json` and
`tsconfig.desktop.json` aliases; no React platform provider is required. On the
web, the transport uses the browser HTTP stack; on desktop it invokes the
app-owned Rust `service_request` command, which performs the HTTP request in
the Tauri process and streams packets back over a typed channel.

Shared feature code never touches network primitives or the platform runtime
directly. It calls the typed API layer (`src/api`, e.g. `api.post`, `saveUser`)
for JSON endpoints and `src/features/chat/transport.ts` for chat streaming. Run
`npm run check:architecture` to verify those boundaries: Tauri imports stay in
`src/platform/desktop.ts`, browser network APIs stay in `src/platform/web.ts`,
and only `src/api` and `src/features/chat` may import
`@curio/platform-runtime`.

See the root [architecture guide](../ARCHITECTURE.md) and
[deployment notes](../docs/deployment.md).
