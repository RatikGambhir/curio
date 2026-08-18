# Curio frontend

This package is the canonical React application for both product targets.

```bash
npm run dev:web           # Browser SPA
npm run dev:desktop       # Tauri shell + desktop-mode Vite server
npm run build:web         # Static web artifact
npm run build:desktop-ui  # Static UI consumed by Tauri
npm run build:desktop     # Packaged desktop artifact
```

Production builds require `VITE_CURIO_CHAT_WORKER_URL` to be the deployed HTTPS
worker. Development uses localhost by default.

Platform differences belong behind `src/platform/contracts.ts`. Vite selects
the concrete runtime at build time; shared feature code must not inspect Tauri
globals or import `@tauri-apps/*`. Run `npm run check:architecture` to verify
that boundary.

See the root [architecture guide](../ARCHITECTURE.md) and
[deployment notes](../docs/deployment.md).
