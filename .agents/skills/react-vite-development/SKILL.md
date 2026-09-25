---
name: react-vite-development
description: Build, refactor, debug, test, or review Curio's shared React/Vite application in web/. Use for pages, components, routes, hooks, API and chat clients, React Query flows, platform adapters, styling, accessibility, Vitest, and web/desktop UI builds. Do not use as the primary guide for Rust code in web/src-tauri or curio-service.
---

# Curio React + Vite development

Work in the existing shared client architecture. Before editing, read the
relevant frontend and Tauri sections of the canonical
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md), the repository
[`AGENTS.md`](../../../AGENTS.md), `web/package.json`, `web/vite.config.ts`, the
target TypeScript configs, and nearby tests. Inspect `git status` and preserve
unrelated work.

If a client feature requires a new or changed `curio-service` endpoint, also
read and apply `../axum-development/SKILL.md` and
`../code-verification/SKILL.md`; do not build presentation against a
hypothetical backend contract.

## Locate the owner

`web/src` is the only React source tree for both product targets. Choose the narrowest existing owner:

- `src/pages`: route composition and page-owned state.
- `src/components/<feature>`: product UI and feature adapters.
- `src/components/ui`: shared primitives.
- `src/components/task-card`, `event-calendar`, `kanban-board`, `data-table`, and `rich-text-editor`: reusable controlled systems with deliberate public barrels.
- `src/hooks`: reusable application lifecycles and React Query ownership.
- `src/api`: buffered JSON resources and their wire types.
- `src/features/chat`: target-independent SSE protocol and streaming transport.
- `src/platform`: the browser/desktop capability boundary.
- `src/app`: route inventory, guards, lazy page mapping, and target router shells.

Search for an analogous implementation and test before creating a new pattern. Keep page-specific persistence and API adaptation outside package-like reusable component systems.

## Preserve the dual-target boundary

Vite aliases select implementations at build time:

- `@curio/platform-runtime` resolves to `platform/web.ts` or `platform/desktop.ts`.
- `@curio/router-runtime` resolves to a `BrowserRouter` or `HashRouter` shell.
- `tsconfig.web.json` and `tsconfig.desktop.json` must mirror those aliases.

Keep these invariants:

- Components, pages, and ordinary hooks do not call `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, or `sendBeacon`.
- Only `src/platform/web.ts` uses browser networking.
- Only `src/platform/desktop.ts` imports `@tauri-apps/*`.
- Only `src/api` and `src/features/chat` import `@curio/platform-runtime`.
- Shared code never imports a concrete web or desktop adapter.
- Desktop OS access goes through a narrow app-owned Rust command, not a frontend Tauri plugin.
- One React implementation must continue to serve web and desktop; do not add a second desktop UI tree.

Run `npm run check:architecture` whenever a change can affect these boundaries.

## Routes, state, and data

Add or change routes through both `src/app/route-manifest.ts` and the lazy page mapping in `src/app/router.tsx`. Preserve target availability and `public`, `anonymous`, or `authenticated` access metadata. Route guards are currently local UX behavior, not server authorization.

Use existing state owners:

- React Query owns cached service state and mutations.
- `AuthProvider` owns the development-only local user session.
- `ThemeProvider` owns appearance and custom theme persistence.
- URL state owns shareable navigation state when applicable.
- Local component state owns transient presentation and edit drafts.

Do not copy query-cache data into local state unless it is an intentional draft. Derive render values rather than synchronizing them with Effects. Use Effects only at external-system boundaries and make cancellation and cleanup safe under repeated setup.

For a JSON resource, put paths, request/response types, and transport calls in
`src/api`. Keep service paths code-owned, single-leading-slash, and relative;
encode dynamic segments. Add a feature hook when cache keys, invalidation,
cancellation, authenticated ownership, or UI mapping need an owner. Treat
parsed JSON as an untrusted boundary. When no runtime-schema convention exists,
either add narrow validation for the new/changed high-risk boundary or
explicitly retain and report the known assertion-only debt; never imply that a
TypeScript cast validated a response.

Chat streaming stays separate from the buffered JSON client. Keep raw-byte transport in the platform adapter, SSE framing/correlation/terminal rules in `features/chat`, and message state/cancellation in `useChat`. A successful chat stream must have exactly one valid terminal event.

## Product-state honesty

Do not accidentally document or code against planned persistence as though it already exists:

- Authentication is mock local storage and bearer contents are not cryptographically verified.
- Calendar create/range-list and account-profile save are service-backed.
- Calendar task list and kanban are projections of calendar events, not a separate task resource.
- Existing calendar records are read-only until update/delete endpoints exist.
- Chat writes to the service, but the page starts from demo state and does not hydrate persisted history.
- Notes, Vault, Atlas, Profile Setup, and much of Home/Settings are mock, local, or in-memory.

When replacing mock data, define the service contract and failure states before changing presentation code.

## Persisted calendar editing

Apply these rules when adding update behavior to the currently read-only stored
events:

- Define the PATCH path, editable-field allowlist, omitted-versus-null meaning,
  authoritative returned record, validation/not-found/forbidden/conflict
  statuses, and concurrency choice before wiring UI callbacks.
- The service must authorize bearer-derived identity against the stored row in
  a race-safe owner-scoped operation. Prior list visibility, client permissions,
  an event ID, or a body `userId` is not authorization.
- The reusable calendar editing extension can emit `onChange` plus granular
  callbacks for one logical edit. Choose one canonical persistence boundary and
  test that one gesture produces one mutation; persisting every callback
  duplicates requests, while listening only to a granular callback can miss
  fields such as priority.
- Keep React Query as the controlled source of truth. Choose an explicit
  optimistic flow (snapshot, exact cache update, rollback, authoritative
  response reconciliation) or a pessimistic flow with visible pending/error
  feedback. Do not let controlled data snap back or appear saved after failure.
- An edit can move an event between cached visible ranges. Reconcile or
  invalidate the full `['calendar', userId]` query prefix, not only the current
  range.
- Enable only calendar permissions backed by the implemented contract. PATCH
  does not imply delete, child creation, or support for every UI-editable field.
- Keep update wire/domain types independent of presentation modules; do not
  deepen the current `api/calendar.ts` dependency on calendar UI types.
- Test the exact path/body/bearer, all-day/timed/milestone transitions, one
  request per logical edit, success/error reconciliation, range movement, Axum
  ownership/persistence, and both active transports as required by the feature.

## UI and domain invariants

- Match the Tailwind 4, shadcn/Radix, semantic-token, Lucide, and type-system conventions already in use: Schibsted Grotesk (`font-sans`) for interface text, Newsreader (`font-serif`/`font-display`) for long-form reading and display headings, JetBrains Mono (`font-mono`/`eyebrow`) for index marks and code.
- Authenticated pages render inside the persistent `AppShell` layout route and start with `PageHeader`; page-owned secondary lists use `ContextPane`. Reuse `SegmentedControl`, `Notice`, and `EmptyState` before adding another variant of those patterns.
- Consume semantic theme roles instead of hard-coded palette values, except for deliberate fixed domains such as syntax highlighting.
- Prefer semantic HTML and native controls. Preserve labels, accessible names, keyboard behavior, focus visibility, reduced motion, and narrow/wide layout behavior.
- Keep destructive and asynchronous actions guarded against duplicate submission and expose useful loading, empty, error, disabled, and retry states.
- Preserve controlled component APIs and public `index.ts` barrels in reusable systems.
- Route new external-link behavior through `PlatformServices`; do not copy the
  current direct-anchor debt for untrusted URLs. Validate allowed protocols and
  verify desktop CSP/native behavior before adding remote images or media.
- Calendar wire dates are domain data: bare `YYYY-MM-DD` means all-day, RFC 3339 means timed, and a missing timed end means milestone. Do not normalize date-only strings into timestamps in the client.
- The reusable task/card/calendar/kanban permission and clipboard contracts are shared across surfaces; update adapters and focused tests together when those contracts change.

## Vite and configuration

Treat every value bundled into the client as public. Never place OpenAI keys, database URLs, private tokens, or other secrets in `VITE_*` variables.

`VITE_CURIO_SERVICE_URL` is build-time configuration. Development defaults to `http://127.0.0.1:3000`; production builds require an explicit HTTPS URL unless the documented local debug escape hatch is intentionally used. The Vite server runs on strict port `1420`. A Vite development setting or proxy would not configure production hosting, service CORS, or SPA fallback.

Do not edit `dist`, `.vite`, generated Tauri files, dependency trees, or build artifacts. Keep aliases synchronized across Vite and both target TypeScript configs.

## Verify proportionally

Run commands from `web` and use npm, matching `package-lock.json`:

```bash
npm run check:architecture
npm run lint
npm test
VITE_CURIO_SERVICE_URL=https://service.example.com npm run build:web
VITE_CURIO_SERVICE_URL=https://service.example.com npm run build:desktop-ui
```

Start with a focused Vitest file when possible, then run the relevant broader checks. Both build commands matter when shared code, aliases, platform types, routing, or configuration changes. A Vite build is not a substitute for the explicit TypeScript build already embedded in these scripts.

For meaningful UI behavior, also exercise the affected route in the appropriate runtime. Check the primary flow, the most relevant alternate state, console/network failures, keyboard interaction, and narrow and wide layouts. If Rust under `web/src-tauri` changes, also follow the Tauri verification matrix in `AGENTS.md` and the `code-verification` skill.

Report exact commands and outcomes, skipped checks, remaining uncertainty, and any failure proven to predate the change.

Before handoff, compare the frontend diff with `docs/ARCHITECTURE.md`. Update
that canonical document in the same change when a feature or fix changes routes,
state/persistence maturity, platform boundaries, API or SSE contracts, reusable
systems, configuration, build behavior, limitations, or verification. If no
claim changes, report `Architecture impact: none` rather than adding cosmetic
documentation churn.
