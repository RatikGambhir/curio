# Calendar page plan (ilinxa event-calendar)

Linear: [RAT-16](https://linear.app/ratik-gambhir/issue/RAT-16/calendar-page-ilinxa-event-calendar)
Component: <https://ui.ilinxa.com/components/event-calendar>

A new **Calendar** page at `web/src/pages/Calendar.tsx`, built on the ilinxa
event-calendar shadcn registry component.

## Scope

A route-guarded `/calendar` page rendering month/week/day/agenda views over Curio
event data, in the existing app shell (sidebar + `PageHeader`) used by
`Atlas.tsx` and `Vault.tsx`.

**In scope:** registry install, missing shadcn primitives, domain types + mock
data, the page and its route wiring, theming, opt-in editing, and passing the
repo's lint/boundary/type gates.

**Out of scope:** backend persistence. `curio-service` has no events table —
migrations are `0001_chat.sql` and `0002_users.sql` only. Phase 1 ships against
mock data behind a typed seam, exactly as `Vault.tsx` does with
`vault.mock-data.ts`. Phase 7 sketches what persistence would take, but it is not
part of this plan's definition of done.

## What the component gives us

Registry: `@ilinxa/event-calendar` (plus optional `@ilinxa/event-calendar-fixtures`
and `@ilinxa/event-calendar-editing`).

- Exports `EventCalendar` (main assembly), `EventCalendarRoot` (headless),
  `CalendarMonthView` / `CalendarWeekView` / `CalendarDayView` /
  `CalendarAgendaView`, `CalendarToolbar`, `CalendarSkeleton`, `useCalendar`.
- Consumes a canonical `TaskItem[]` via `data` — no adapter layer required.
- Key props: `data`, `statusOptions`, `priorityOptions`, `labelOptions`,
  `statusColors`, `defaultView`, `now`, `showMiniNav`, `onTaskClick`,
  `onRangeChange`, `editable`, `editing`, `onChange`, `permissions`,
  `renderTooltip`.
- Event typing is inferred: a `classifyEvent()` predicate, else date-only strings
  (`"2026-06-22"`) = all-day vs. full timestamps = timed, else a span heuristic.
  Milestones render as markers when a timestamp has no end date.
- Keyboard: `M/W/D/A` switch views, arrows/PageUp/PageDown step the period, `T`
  jumps to today; on a focused event arrows move, Shift+arrows resize, Enter/F2
  edit, Delete removes.

## Gap analysis against this repo

| Requirement | Repo state | Action |
| --- | --- | --- |
| `@ilinxa` registry | Not in `web/components.json` (`registries` has only `@shadcnblocks`, `@magicui`, `@moleculeui`) | Add the registry entry |
| shadcn `avatar`, `button`, `input`, `skeleton` | Present in `web/src/components/ui/` | — |
| shadcn `badge`, `calendar`, `context-menu`, `popover` | **Missing** | Install; `calendar` pulls `react-day-picker` |
| `date-fns@^4.1.0` | Not installed | Add |
| `@dnd-kit/core@^6.3.1`, `@dnd-kit/utilities@^3.2.2` | Not installed | Add (needed for the editing slice) |
| `lucide-react@^1.11.0` peer | Repo pins `^0.525.0` | **Risk — see below** |
| `task-card` internal dep | Not present | Pulled transitively by the registry |
| Package manager | Repo uses **npm** (`package-lock.json`); docs say `pnpm dlx` | Use `npx shadcn@latest add ...` |

## Risks

**1. `lucide-react` peer mismatch (highest).** The component declares
`lucide-react@^1.11.0`; the repo is on `^0.525.0`, and lucide's 0.x line is not
semver-compatible with a 1.x range. Resolve this *before* any other work — either
the peer range is advisory and 0.525 satisfies the actual imports, or a major
upgrade is required, which would touch every icon import across
`app-sidebar.tsx`, `nav-*.tsx`, the vault components, and the landing page.

**2. Vendored code vs. the architecture gates.** Registry code lands inside
`web/src/`, which is scanned by both gates:

- `web/scripts/check-frontend-boundaries.mjs` hard-fails on any `fetch(`,
  `XMLHttpRequest`, `WebSocket`, `EventSource`, or `sendBeacon(` outside
  `src/platform/web.ts`.
- `web/eslint.config.js` applies `no-restricted-globals: fetch` to
  `**/*.{ts,tsx}`.

The calendar is presentational and should be clean, but `onRangeChange`
lazy-fetch examples in the docs suggest data-fetching demo code may ship with the
fixtures. Verify immediately after install.

**3. Strict TypeScript.** `tsconfig.app.json` sets `strict`, `noUnusedLocals`,
`noUnusedParameters`, `noFallthroughCasesInSwitch`, and `verbatimModuleSyntax`.
Vendored third-party code frequently trips `noUnusedParameters` and
`verbatimModuleSyntax` (type imports must be `import type`).

**4. Tailwind v4.** The repo is CSS-first — `components.json` has
`"tailwind.config": ""` and all tokens live as CSS custom properties in
`web/src/index.css`. Registry components written against a v3
`tailwind.config.js` may reference tokens that do not exist here.

**5. Both build targets.** Web and desktop share `src/`, so the page must
type-check under `tsconfig.web.json` *and* `tsconfig.desktop.json`, and work
under `HashRouter` (desktop) as well as `BrowserRouter` (web).

## Proposed file layout

```
web/src/pages/Calendar.tsx                      # page shell + state
web/src/components/calendar/calendar.types.ts   # TaskItem re-export, Curio event type
web/src/components/calendar/calendar.config.ts  # statusOptions/priorityOptions/statusColors
web/src/components/calendar/calendar.mock-data.ts
web/src/components/event-calendar/**            # vendored registry output — do not hand-edit
```

Mirrors the `src/components/vault/` convention (`vault.types.ts`,
`vault.mock-data.ts`, feature components) with the page owning state.

---

## Phase 1 — Install and resolve the lucide peer

Gating. Nothing else starts until this is clean.

**Registry.** Add alongside the existing entries in `web/components.json`:

```json
"@ilinxa": "https://ui.ilinxa.com/r/{name}.json"
```

Confirm the URL template against the docs — it is inferred from the sibling
entries, not copied from ilinxa's instructions.

**Install** (npm, not pnpm), from `web/`:

```bash
npx shadcn@latest add @ilinxa/event-calendar
```

**Resolve the lucide peer — the real work.** Determine which is true, in order:

1. *The peer range is advisory.* Grep the installed calendar source for its actual
   `lucide-react` imports and check each icon name exists in 0.525. If they all
   resolve, pin an override and move on — cheapest outcome.
2. *A real upgrade is needed.* Scope the blast radius first: every
   `lucide-react` import across `app-sidebar.tsx`, `nav-main.tsx`,
   `nav-projects.tsx`, `nav-user.tsx`, `src/components/vault/**`,
   `src/components/landing/**`, `src/components/ui/**`. Icon renames between
   majors are the usual breakage. Do the upgrade as its own commit, separate from
   any calendar code.

Do not paper over this with `--force`. A silently mismatched icon package
produces missing-icon runtime failures that only show up in views nobody clicked
during review.

**Verify immediately:** run `npm run check:architecture` against the vendored
output. This is the cheapest point to discover a boundary violation.

**Exit criteria:** registry resolves, install completes, `lucide-react` is on one
version satisfying both app and component, `npm install` needs no
`--legacy-peer-deps` or `--force`.

## Phase 2 — Missing primitives and dependencies

Already present in `web/src/components/ui/`: `avatar.tsx`, `button.tsx`,
`input.tsx`, `skeleton.tsx`.

```bash
npx shadcn@latest add badge calendar context-menu popover
npm install date-fns@^4.1.0 @dnd-kit/core@^6.3.1 @dnd-kit/utilities@^3.2.2
```

- `calendar` pulls `react-day-picker`. Check which major shadcn installs — v8 and
  v9 have incompatible APIs, and the mini-nav (`showMiniNav`) sits on top of it.
- `context-menu` and `popover` add `@radix-ui/react-context-menu` and
  `@radix-ui/react-popover`. The repo has both the `radix-ui@^1.4.3` umbrella and
  individual `@radix-ui/react-*` entries — match whichever convention
  `dropdown-menu.tsx` and `hover-card.tsx` already use rather than mixing.
- Existing `ui/` files are shadcn **new-york** style, `baseColor: neutral`,
  `cssVariables: true`. Diff CLI output against `ui/dialog.tsx` before committing.
- `@dnd-kit/*` is only exercised by Phase 6 but is a declared peer of the base
  component — install now so resolution settles in one pass.
- The registry may also pull an internal `task-card` component (shared across
  ilinxa's task-family). Let it install; note where it lands, since it is vendored
  code subject to the same gates.

## Phase 3 — Domain types, config, and mock data

Three files under `web/src/components/calendar/`:

**`calendar.types.ts`** — re-export `TaskItem` from the vendored component as the
calendar's event type, so page code never imports out of
`src/components/event-calendar/**` directly. That indirection is what lets the
shadcn CLI regenerate the vendored tree without touching app code. Define a
`CurioCalendarEvent` alias (or narrow extension) and point every app-side
reference at it.

**Event classification** — the component infers type in three layers, and the
data model must cooperate: a `classifyEvent()` predicate; else date format
(date-only = all-day, full timestamp = timed); else a span heuristic.

Layer 2 is a **string-format contract, not a `Date` contract**. All-day events
must serialize as bare `YYYY-MM-DD`. If mock data (or later an API) returns a
full ISO timestamp for an all-day event, it silently renders as a timed block at
midnight. Pin this in the type layer with a comment and cover both shapes in
fixtures. A timestamp with no end date renders as a **milestone marker** — decide
whether Curio wants that shape and represent it explicitly.

**`calendar.config.ts`** — `statusOptions`, `priorityOptions`, `labelOptions`,
`statusColors`. Map `statusColors` onto the existing theme tokens in
`web/src/index.css`; `--chart-1` through `--chart-5` are already a coherent oklch
green/neutral palette. No raw hex — it will not respond to the dark-theme block.

**`calendar.mock-data.ts`** — cover what each view stresses: all-day single and
multi-day spans (month's spanning bars), timed and overlapping events (week/day
lane packing), a milestone, enough events on one day to trigger `+N more`
overflow, and events near the current week so day/week are not empty on load.

Optionally start from `@ilinxa/event-calendar-fixtures` to see the expected
shape, then replace with Curio-flavoured data. Do not ship vendor fixtures as the
app's mock data — demo content, and another vendored file on the lint surface.

## Phase 4 — Page and route wiring

**The page.** Follow `Atlas.tsx`, not `Vault.tsx` — the week/day time-grid needs
a real bounded height, and `Vault`'s scrolling `max-w-[1200px]` container would
collapse it:

```tsx
<SidebarProvider>
  <AppSidebar />
  <SidebarInset className="bg-background">
    <div className="flex h-screen w-full flex-col bg-background">
      <PageHeader />
      <main className="min-h-0 flex-1 px-4 pb-4">
        {/* EventCalendar in a rounded bordered container, as Atlas does for ReactFlow */}
      </main>
    </div>
  </SidebarInset>
</SidebarProvider>
```

Default export — the router lazy-imports every page.

**Props:** `data` from Phase 3; the option sets from `calendar.config.ts`;
`defaultView="month"`; `showMiniNav`.

`now` must come from a **`useState(() => new Date())` initializer**, never an
inline `new Date()` — an inline value gets fresh identity every render, and if
the component treats `now` as a dependency that can drive `onRangeChange` into a
loop. The docs frame `now` as an SSR-stability prop; the same hazard applies in
an SPA.

`onTaskClick` — stub wired to local selection state so the callback is proven to
fire (a detail panel is out of scope). `onRangeChange` — no-op with a comment
pointing at Phase 7. It must **not** call `fetch` from page code; ESLint bans the
global and the boundary script hard-fails on it. Real fetching goes through
`src/api/`.

Leave `editable` off — that is Phase 6.

**Route wiring — five coordinated edits.** `RouteId` is a closed union consumed
by an exhaustive `switch`, so a missing edit is a type error rather than a silent
404, but all five are required:

1. `web/src/app/route-manifest.ts:12` — add `| "calendar"` to `RouteId`
2. `web/src/app/route-manifest.ts:43` — add
   `{ id: "calendar", path: "/calendar", access: "authenticated", targets: allTargets }`
3. `web/src/app/router.tsx:18` —
   `const Calendar = lazy(() => import("@/pages/Calendar"))`
4. `web/src/app/router.tsx:64` — `case "calendar": return <Calendar />` in
   `pageForRoute`
5. `web/src/components/app-sidebar.tsx:27` — a `navMain` entry:
   `{ title: "Calendar", url: "/calendar", icon: CalendarDays }`

`targets: allTargets` is not optional — `route-manifest.test.ts` asserts every
authenticated path reaches both targets. `access: "authenticated"` routes through
`RequireAuth`, which redirects to `/login` preserving `state.from`.

Nothing in the page may import `@tauri-apps/*` or `@curio/platform-runtime` —
ESLint `no-restricted-imports` blocks both outside their designated modules.

## Phase 5 — Theming and gate conformance

**Theming.** `web/src/index.css` is Tailwind v4 CSS-first — every token is a CSS
custom property on `:root` with a dark block, an oklch green/neutral palette with
`--chart-1` … `--chart-5`, custom `--sidebar-*`, `--radius: 0.35rem`, and shadow
tokens. Registry components authored against a v3 config may reference utilities
or tokens that do not resolve. Check for: color classes pointing at absent
tokens; hardcoded hex/rgb that will not flip in dark mode; radius/shadow values
ignoring `--radius` and `--shadow-*`; and the now-indicator line, today
highlight, and event chips, which are the highest-signal surfaces for a mismatch.
Verify all four views in **both** themes.

**Gates.** Four run over `web/src/`, and vendored code sits inside that tree:

1. `npm run check:architecture` — hard-fails on `fetch(`, `XMLHttpRequest`,
   `WebSocket`, `EventSource`, `sendBeacon(` outside `src/platform/web.ts`. Strip
   any such code; the calendar should be presentational and Curio's data access
   belongs in `src/api/`.
2. `npm run lint` — `no-restricted-globals: fetch` across `**/*.{ts,tsx}`, no
   vendor exemption. `reactRefresh.configs.vite` also flags files exporting both
   components and non-components, common in registry barrels.
3. `npm run build:web` / `npm run build:desktop-ui` — `tsc -b` under the strict
   flags above.
4. `npm test` — vitest.

**On fixing vendored code.** Prefer the smallest edit that clears the gate, and
record every change. Vendored files are regenerated by `npx shadcn@latest add`,
so hand-edits are silently lost on update. If they pile up, that is the signal to
either adopt the tree as app-owned or narrow the gates with an explicit, reviewed
exemption.

Do not add a blanket vendor exemption to `check-frontend-boundaries.mjs` — it is
a security boundary described in `docs/ARCHITECTURE.md`, and a wildcard under
`src/components/` would neutralize it for far more than the calendar.

## Phase 6 — Editing slice (opt-in)

Only after the read-only page is green.

```bash
npx shadcn@latest add @ilinxa/event-calendar-editing
```

```tsx
import { calendarEditing } from "@/components/event-calendar/features/editing"

<EventCalendar
  editable
  editing={calendarEditing}
  data={events}
  onChange={setEvents}
  statusOptions={statusOptions}
  permissions={permissions}
/>
```

`onChange` makes the component **controlled** — the page now owns the event
array. With no backend, mutations live in React state only and vanish on reload.
Decide explicitly whether that is acceptable or whether editing stays behind a
flag until persistence lands; a calendar that silently discards edits on refresh
is worse than one that is visibly read-only.

`permissions` takes a `TaskPermissions` matrix gating which actions appear.
Define it in `calendar.config.ts`. Start restrictive.

Turns on: drag-to-reschedule, edge-resize, create by draw/double-click,
right-click Copy/Cut/Edit/Delete, keyboard mutations on a focused event,
cross-surface clipboard.

**Verify:**

- **Delete has no confirmation step** in the documented keybindings — a stray
  `Delete` on a focused event removes it. With no persistence and no undo that is
  unrecoverable. Confirm whether the slice ships undo; if not, gate delete through
  `permissions` or add a confirm.
- Keyboard handlers are global (`M/W/D/A`, `T`, arrows). Check they do not fire
  while focus is in the sidebar search, `PageHeader` controls, or any input —
  single-letter global shortcuts are a common typing-hijack bug.
- `@dnd-kit` sensors vs. the `SidebarProvider` layout can interact badly at
  container edges.
- Editing must preserve the date-only vs. timestamp distinction from Phase 3.
  Dragging an all-day event must not silently convert it to a timed one at
  midnight.

## Phase 7 — Persistence (not in this plan)

Sketched so the seam is documented; **not** part of the definition of done.

`curio-service` (Axum + SQLite) has no events concept. Persistence would need
`migrations/0003_events.sql` (events owned by user id, with the all-day vs. timed
distinction in the schema rather than inferred at render time), an `src/events/`
module following the `src/user/` shape, and bearer-protected
`GET /v1/events?start=&end=` plus create/update/delete — the range query is what
makes `onRangeChange` lazy-fetch worthwhile.

Frontend: `web/src/api/events.ts` following `src/api/conversations.ts` (typed
records over `api.get/post/patch/delete`, no URL construction or network
primitives outside that layer), plus a React Query hook keyed by the visible
range.

**The critical contract:** the component classifies by string format. A backend
that normalizes everything to ISO-8601 timestamps will silently turn every
all-day event into a midnight-anchored timed block. Decide this at schema-design
time, not at the serializer.

Blocked on a product decision about what a Curio calendar event represents —
user-authored, derived from vault items or chat, or both.

## Definition of done (Phases 1–6)

- `/calendar` renders for authenticated users on both targets, reachable from the
  sidebar
- All four views render; period navigation, `showMiniNav`, and keyboard shortcuts
  work
- Light and dark themes read correctly against `src/index.css` tokens
- `npm run lint`, `npm run check:architecture`, `npm run build:web`,
  `npm run build:desktop-ui`, and `npm test` all pass
