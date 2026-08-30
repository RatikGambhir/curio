# Notes page plan (ilinxa rich-text-editor)

Linear: [RAT-24](https://linear.app/ratik-gambhir/issue/RAT-24/notes-page-ilinxa-rich-text-editor)
Component: <https://ui.ilinxa.com/components/rich-text-editor>

A new **Notes** page at `web/src/pages/Notes.tsx`: a folder/note sidebar on the
left, the ilinxa rich-text-editor filling the inset on the right.

## Scope

- `web/src/pages/Notes.tsx` at `/notes`, authenticated, both targets
- A page-specific `NotesSidebar` listing the user's folders with notes nested
  inside
- `RichTextEditor` as the main inset surface, editing the selected note
- Mock folders and notes — no backend

**Out of scope:** persistence. Notes live in React state and reset on reload for
Phase 1, exactly as `Chat.tsx` runs on `demo-data.ts` today. Phase 6 sketches
what persistence would take, but it is not part of this plan's definition of
done.

## The shell already exists in this codebase

`web/src/pages/Chat.tsx` is the precedent, not `Atlas.tsx`. It swaps the global
`AppSidebar` for a page-specific `ChatSidebar` in the same `SidebarProvider` slot:

```tsx
<SidebarProvider className="h-screen w-full font-sans">
  <ChatSidebar chats={...} selectedChatId={...} onSelectChat={...} />
  <SidebarInset className="bg-background">
    <div className="flex h-full w-full flex-col bg-background">
      <PageHeader />
      {/* main surface */}
    </div>
  </SidebarInset>
</SidebarProvider>
```

`ChatSidebar` (`src/components/chat-sidebar.tsx`) is the template for
`NotesSidebar`: `Sidebar collapsible="icon"`, a `SidebarHeader` with a
back-to-`/home` arrow and the Curio logo, a `SidebarContent` holding the list nav,
`NavUser` in the footer, `SidebarRail`. Copy that structure — Notes should feel
like Chat, not like a second app.

The folder→notes nesting is already available: `src/components/ui/sidebar.tsx`
exports `SidebarMenuSub`, `SidebarMenuSubItem`, and `SidebarMenuSubButton`, and
`src/components/ui/collapsible.tsx` exists for expand/collapse.

## What the component gives us

Registry: `@ilinxa/rich-text-editor`, plus optional
`@ilinxa/rich-text-editor-fixtures`.

Two exports from one folder:

- `<RichTextEditor>` — client WYSIWYG, ~165KB gzip
- `<RichTextViewer>` — read-only, ~32KB gzip, via `platejs/static`

```typescript
interface RichTextEditorProps {
  value?: RichTextValue
  defaultValue?: RichTextValue
  onChange?: (value: RichTextValue) => void
  onSave?: (value: RichTextValue) => void      // Cmd/Ctrl+S
  onImageUpload?: ImageUploader
  autoFocus?: boolean
}

type RichTextValue = Element[]  // Plate node array
```

Also exports `serializeRichTextToHtml(value, options?)`, documented as async and
**server-only**.

Toolbar: fixed top bar (marks, H1–H3, blockquote, code block, hr, lists,
link/image/table, font family/size/color) plus a selection-anchored floating
toolbar via `@floating-ui/react`. Code blocks highlight 15 languages with token
colors mapped to `chart-1..5`. Images resize inline (width as a percentage) with
editable captions.

**Data model:** Plate's `Value` — an array of element nodes as JSON, not HTML.
The docs explicitly warn against HTML round-trips; JSON stays authoritative and
HTML is an export-boundary format only.

## Gap analysis

| Requirement | Repo state | Action |
| --- | --- | --- |
| `@ilinxa` registry | Not in `web/components.json` | Add (shared with RAT-16 — may already be done) |
| shadcn `button` | Present | — |
| 10 × `@platejs/*` + `platejs@^53` | Not installed | Add, all in lockstep |
| `@floating-ui/react@^0.27.19` | Not installed | Add |
| `lowlight@^3.3.0`, `highlight.js@^11.11.1` | Not installed | Add |
| `@tailwindcss/typography` | **Not installed, no `@plugin` line in `index.css`** | See risk 2 |
| `lucide-react@^1.11.0` peer | Repo pins `^0.525.0` | Same conflict as RAT-16 Phase 1 |
| `Collapsible`, `SidebarMenuSub*` | Present | — |
| Package manager | npm; docs say `pnpm dlx` | `npx shadcn@latest add @ilinxa/rich-text-editor` |

## Risks

**1. The documented usage example will break the build.** Both code samples on
the component page call `fetch()` directly inside `onSave` and `onImageUpload`.
In this repo that fails two gates: `web/eslint.config.js` bans the `fetch` global
across `**/*.{ts,tsx}`, and `web/scripts/check-frontend-boundaries.mjs`
hard-fails on `fetch(` anywhere under `src/` outside `src/platform/web.ts`. Do
not copy the example. `onSave` updates local state in Phase 1; any real network
call goes through `src/api/`.

**2. `@tailwindcss/typography` is missing entirely.** The docs note prose styling
depends on it. It is not in `package.json`, and `src/index.css` has no `@plugin`
directive. Under Tailwind v4 this is registered in CSS
(`@plugin "@tailwindcss/typography";`), not in a config file — the repo is
CSS-first with `"tailwind.config": ""`. Without it, editor content renders
unstyled. Verify whether the component ships its own prose styles or genuinely
needs the plugin before adding it, since the plugin's `prose` defaults may fight
the existing oklch token palette.

**3. Thirteen `@platejs/*` packages pinned to `^53.x`.** They must move together —
a single mismatched minor across `platejs`, `@platejs/basic-nodes`,
`@platejs/table`, etc. produces confusing plugin-registration failures. Install
in one command and commit the lockfile change on its own.

**4. `lucide-react` peer, again.** Same `^1.11.0` vs. `^0.525.0` conflict as the
calendar plan. Whichever page lands first sets the resolution; the other
inherits it. Do not resolve it twice in different directions.

**5. Bundle weight.** ~165KB gzip for the editor. `web/src/app/router.tsx`
already lazy-imports every page, so this is code-split behind `/notes` by
default — preserve that. `NotesSidebar` must not import from the editor folder,
or the sidebar drags the editor chunk in with it.

**6. `"use client"` and server-only APIs.** This is a Vite SPA, not Next.js. The
`"use client"` directive is inert here (`nav-main.tsx` already carries one
harmlessly). But `RichTextViewer` renders via `platejs/static` and
`serializeRichTextToHtml` is documented server-only — there is no server in this
app. Verify both actually run in-browser before building anything on them;
neither is needed for the core editing flow.

**7. Both build targets.** Web (`BrowserRouter`) and desktop (`HashRouter`) share
`src/`. The page must type-check under `tsconfig.web.json` and
`tsconfig.desktop.json`, and nothing in it may import `@tauri-apps/*` or
`@curio/platform-runtime`.

## Editor state across note switching

The sharpest correctness trap. The editor is controlled via `value` + `onChange`
and carries an internal "echo-guard" that suppresses `onChange` emissions caused
by external `value` updates.

Selecting a different note changes `value` externally. If the editor is not
remounted, that guard plus Plate's internal editor instance can leave the
previous note's selection, undo history, or content bleeding into the new one —
and in the worst case write note A's content into note B.

**Mitigation:** `key={selectedNoteId}` on `<RichTextEditor>` so each note gets a
fresh instance. Cheap, and it makes the isolation structural rather than
dependent on the guard's behaviour. Explicitly test switching between two notes
with unrelated content, including mid-edit.

## Proposed file layout

```
web/src/pages/Notes.tsx                        # page shell + selection state
web/src/components/notes/notes-sidebar.tsx     # mirrors chat-sidebar.tsx
web/src/components/notes/notes-nav.tsx         # folder tree, mirrors ui/chat-nav.tsx
web/src/components/notes/notes.types.ts        # NoteFolder, NoteItem, RichTextValue re-export
web/src/components/notes/notes.mock-data.ts    # mirrors features/chat/demo-data.ts
web/src/components/rich-text-editor/**         # vendored registry output — do not hand-edit
```

Chat splits its types/data into `src/features/chat/` because that feature has
protocol logic shared with the transport layer. Notes has no such logic, so the
`src/components/vault/` convention (types and mock data colocated with the
components) is the right fit.

---

## Phase 1 — Install the editor and the Plate stack

Gating. Everything else depends on this landing cleanly.

**Registry.** If the calendar plan has already added the `@ilinxa` entry to
`web/components.json`, skip this. Otherwise:

```json
"@ilinxa": "https://ui.ilinxa.com/r/{name}.json"
```

The rich-text-editor docs give this template explicitly (unlike the calendar
page), so it can be copied as-is.

**Install.** Repo is npm, not pnpm. From `web/`:

```bash
npx shadcn@latest add @ilinxa/rich-text-editor
```

Only one shadcn primitive is required — `button` — and it is already present.

**Plate stack.** Thirteen npm peers, all in one command so resolution is settled
in a single pass:

```bash
npm install platejs@^53.0.3 @platejs/basic-nodes@^53.0.0 @platejs/basic-styles@^53.0.0 @platejs/caption@^53.0.0 @platejs/code-block@^53.0.0 @platejs/indent@^53.0.0 @platejs/link@^53.0.3 @platejs/list@^53.0.2 @platejs/media@^53.0.1 @platejs/table@^53.0.0 @floating-ui/react@^0.27.19 lowlight@^3.3.0 highlight.js@^11.11.1
```

The `@platejs/*` packages are versioned together at `^53.x`. A single package
drifting to a different minor causes plugin-registration failures that surface as
missing toolbar features rather than clean errors. Commit the lockfile change on
its own so a future bisect can isolate it.

**lucide-react peer.** Declares `lucide-react@^1.11.0`; repo pins `^0.525.0`.
Same conflict as the calendar plan's Phase 1. If that is already resolved, adopt
its resolution unchanged; if Notes lands first, resolve it here and record the
decision on RAT-16 so both converge. One resolution across the repo. No
`--force`.

**`@tailwindcss/typography`.** Not installed, and `src/index.css` has no
`@plugin` directive. Before adding it, check whether the vendored editor ships
its own content styles — if it does, the plugin's `prose` defaults may fight the
existing oklch palette rather than help. If genuinely needed, register it the
Tailwind v4 way, in CSS:

```css
@plugin "@tailwindcss/typography";
```

Not in a config file — `components.json` has `"tailwind.config": ""` and this
repo is CSS-first.

**Verify before moving on:**

```bash
npm run check:architecture
```

The editor takes upload/save as callback props, so vendored code should be
clean — the fixtures package is the likelier offender if it was installed.

## Phase 2 — Notes types and mock data

Two files under `web/src/components/notes/`.

**`notes.types.ts`** — model the two-level structure the sidebar renders:

```typescript
export type NoteItem = {
  id: string
  title: string
  updatedAt: string      // display string, as ChatListItem does
  body: RichTextValue
}

export type NoteFolder = {
  id: string
  name: string
  notes: NoteItem[]
}
```

`ChatListItem` in `src/features/chat/types.ts` is the reference — note that it
stores `updatedAt` as a pre-formatted display string (`"2m ago"`, `"Yesterday"`),
not a `Date`. Match that for Phase 1; real timestamps become the persistence
layer's problem.

Re-export `RichTextValue` from the vendored editor here, so no other app file
imports out of `src/components/rich-text-editor/**` directly. That indirection is
what lets the shadcn CLI regenerate the vendored tree without touching app code.

**The body format.** `RichTextValue` is Plate's `Value` — an **array of element
nodes as JSON**, not an HTML string and not markdown. Mock note bodies must be
authored in that node shape, so a hand-written `"<p>hello</p>"` will not render.
The docs warn explicitly against HTML round-trips: JSON is authoritative, HTML is
an export-boundary format only.

Get the exact node shape from `npx shadcn@latest add @ilinxa/rich-text-editor-fixtures`,
or by typing into the editor and logging `onChange`. The second route is more
reliable — it produces exactly what this version emits. Do not ship the vendor
fixtures as the app's mock data; they are demo content and add another vendored
file to the lint surface.

**`notes.mock-data.ts`** — three or four folders, two to five notes each. Include
at least one note exercising the richer node types the toolbar produces — a
heading, a list, a code block, a blockquote — so the prose theming work in Phase 5
has something real to check against. One deliberately long note is useful for
testing editor scroll inside the inset.

## Phase 3 — NotesSidebar, folder tree with nested notes

Two files: `notes-sidebar.tsx` (mirrors `chat-sidebar.tsx`) and `notes-nav.tsx`
(folder tree, mirrors `ui/chat-nav.tsx`).

**Structure.** Copy `chat-sidebar.tsx`'s shape so Notes reads as the same app:

- `<Sidebar collapsible="icon">`
- `SidebarHeader` — `h-16 shrink-0 justify-center border-b border-sidebar-border`,
  holding a ghost-button back arrow to `/home`, the Curio logo, and a
  `SidebarTrigger`, with the `group-data-[collapsible=icon]:hidden` treatment on
  the pieces that collapse away
- `SidebarContent` — `<NotesNav>`
- `SidebarFooter` — `<NavUser />`
- `SidebarRail`

Props follow `ChatSidebarProps`: extend `React.ComponentProps<typeof Sidebar>`
and take the data plus selection callbacks (`folders`, `selectedNoteId`,
`onSelectNote`, and whatever create affordances you add).

**The folder tree.** Everything needed is already in the repo — no new
primitives. `Collapsible` wrapping a `SidebarMenuItem` whose `SidebarMenuButton`
is the folder row (`CollapsibleTrigger asChild`), with `CollapsibleContent`
holding a `SidebarMenuSub` of note rows. `ui/chat-nav.tsx` shows the styling
vocabulary for active/hover states — `data-[active=true]` borders and the
`hover:bg-sidebar-foreground/[0.035]` treatment.

**Details worth getting right:**

- **Collapsed (icon) mode.** `collapsible="icon"` means the rail shrinks to
  icons. Nested note rows have no sensible icon representation — decide whether
  folders collapse to icon buttons with tooltips or the tree simply hides.
  `chat-nav.tsx` handles this with `group-data-[collapsible=icon]:hidden` on the
  text and a fallback initial-letter badge.
- **Which folder starts open.** Open the folder containing the selected note on
  mount, rather than all-open or all-closed.
- **Long titles.** `truncate` on note titles, as `chat-nav.tsx` does — folder
  names and note titles both overflow easily at sidebar width.
- **Do not import the editor.** `NotesSidebar` and `NotesNav` must not import
  anything from `src/components/rich-text-editor/**`, or the ~165KB editor chunk
  gets pulled into the sidebar's module graph and the route-level code splitting
  stops helping.
- Framer Motion list animations are optional — `chat-nav.tsx` uses
  `AnimatePresence` for chat rows, and `framer-motion` is already a dependency.

## Phase 4 — Notes.tsx, editor wiring, and route registration

**The page shell.** `Chat.tsx` is the model:

```tsx
<SidebarProvider className="h-screen w-full font-sans">
  <NotesSidebar
    folders={folders}
    selectedNoteId={selectedNoteId}
    onSelectNote={setSelectedNoteId}
  />
  <SidebarInset className="bg-background">
    <div className="flex h-full w-full flex-col bg-background">
      <PageHeader />
      <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden bg-background px-4 py-5 md:px-8 md:py-6">
        {/* RichTextEditor, or an empty state when nothing is selected */}
      </div>
    </div>
  </SidebarInset>
</SidebarProvider>
```

Default export — the router lazy-imports every page.

`min-h-0 flex-1 overflow-hidden` on the editor's container is load-bearing.
Without `min-h-0`, a flex child with long content refuses to shrink and the
editor pushes the page into a full-document scroll instead of scrolling
internally.

**Editor wiring:**

```tsx
<RichTextEditor
  key={selectedNoteId}
  value={note.body}
  onChange={(next) => updateNoteBody(selectedNoteId, next)}
  onSave={(next) => updateNoteBody(selectedNoteId, next)}
/>
```

`key={selectedNoteId}` is not optional — see "Editor state across note
switching" above. Test it explicitly: type into note A, switch to note B
mid-edit, switch back. Content must be intact and unmixed in both.

**Do not copy the docs' `onSave` example.** It calls
`fetch("/api/articles/save", ...)` directly, which fails ESLint's
`no-restricted-globals: fetch` and `check-frontend-boundaries.mjs`. In Phase 1
`onSave` writes to local state; real persistence goes through `src/api/`.

Same for `onImageUpload` — the documented example posts a `FormData` via `fetch`.
Either leave the prop off for Phase 1, or hand back a local object URL. Object
URLs do not survive a reload, which is consistent with everything else being
in-memory for now.

`onSave` fires on `Cmd/Ctrl+S`. Confirm the editor calls `preventDefault` so the
browser's save dialog does not open on top of it.

**Empty state.** Decide what renders before a note is selected. `Chat.tsx` has a
real precedent in `ChatEmptyState`, swapped in via `AnimatePresence` on
`isNewChat`. A simple centered prompt is fine — mounting the 165KB editor with no
document is not.

**Route wiring — five coordinated edits.** `RouteId` is a closed union with an
exhaustive `switch`, so a missed edit is a compile error rather than a silent 404:

1. `web/src/app/route-manifest.ts:12` — add `| "notes"` to `RouteId`
2. `web/src/app/route-manifest.ts:43` — add
   `{ id: "notes", path: "/notes", access: "authenticated", targets: allTargets }`
3. `web/src/app/router.tsx:18` — `const Notes = lazy(() => import("@/pages/Notes"))`
4. `web/src/app/router.tsx:64` — `case "notes": return <Notes />` in `pageForRoute`
5. `web/src/components/app-sidebar.tsx:27` — a `navMain` entry:
   `{ title: "Notes", url: "/notes", icon: NotebookPen }`

`targets: allTargets` is required — `route-manifest.test.ts` asserts every
authenticated path reaches both targets. `access: "authenticated"` routes through
`RequireAuth`, which redirects to `/login` preserving `state.from`.

Keep the `lazy()` import — it is what keeps the editor bundle off every other
route.

## Phase 5 — Prose theming and gate conformance

**Prose theming.** `web/src/index.css` is Tailwind v4 CSS-first: every token is a
CSS custom property on `:root` with a dark-mode block, an oklch green/neutral
palette with `--chart-1` … `--chart-5`, `--radius: 0.35rem`, and `--shadow-*`
tokens.

Two integration points from the docs worth checking:

- **Code block token colors map to `chart-1..5`.** Those tokens exist here, but
  they were chosen as a *chart* palette — five muted greens and neutrals. As
  syntax highlighting they may be near-indistinguishable. Look at a real code
  block before assuming it works, and consider overriding just the highlight
  tokens if contrast is poor.
- **"signal-lime token integration"** — the component's own accent naming. Check
  whether it references tokens absent from `index.css` and remap them onto the
  existing palette rather than adding a parallel one.

Also check the surfaces most likely to be off: the fixed toolbar's borders and
hover states against `--sidebar` and `--border`; the floating selection toolbar's
popover background against `--popover`; table borders; blockquote rules; and
image caption text.

If `@tailwindcss/typography` was added in Phase 1, verify its `prose` defaults do
not override the app's font stack (`--font-sans` is Inter Variable) or foreground
colors. `prose` ships opinionated color and spacing that frequently fights an
existing design system — scope it tightly or override the relevant
`--tw-prose-*` variables.

Verify light **and** dark.

**Gates.** Four run over `web/src/`, and the vendored editor sits inside that
tree:

1. `npm run check:architecture` — hard-fails on `fetch(`, `XMLHttpRequest`,
   `WebSocket`, `EventSource`, `sendBeacon(` outside `src/platform/web.ts`.
2. `npm run lint` — `no-restricted-globals: fetch` across `**/*.{ts,tsx}` with no
   vendor exemption. `reactRefresh.configs.vite` also flags files exporting both
   components and non-components, common in Plate plugin/barrel files.
3. `npm run build:web` / `npm run build:desktop-ui` — `tsc -b` under `strict`,
   `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`,
   `verbatimModuleSyntax`. A vendored tree this size (13 Plate packages' worth of
   glue) is the most likely thing in the repo to trip `noUnusedParameters` and
   `verbatimModuleSyntax`.
4. `npm test` — vitest.

**Bundle check.** Confirm the editor actually landed in its own chunk. Run
`npm run build:web` and inspect the output: the ~165KB gzip editor plus the Plate
stack should sit behind the `/notes` route's lazy chunk, not in the main bundle.
If it leaked into the entry chunk, something outside `pages/Notes.tsx` is
importing from the editor folder — most likely `NotesSidebar` or a type import
that was not written as `import type`.

**On fixing vendored code.** Keep edits minimal and log every one.
`npx shadcn@latest add` regenerates these files, so hand-edits are silently lost
on update. If they accumulate, that is the signal to either adopt the tree as
app-owned or negotiate a narrow, reviewed gate exemption.

Do not add a blanket vendor carve-out to `check-frontend-boundaries.mjs`. That
script is a security boundary documented in `ARCHITECTURE.md`, and a wildcard
under `src/components/` would disable it for far more than the editor.

## Phase 6 — Persistence (not in this plan)

Sketched so the seam is documented; **not** part of the definition of done.

`curio-service` (Axum + SQLite) has no notes concept. Persistence would need
`migrations/0003_notes.sql` (folders and notes tables owned by user id, with note
bodies stored as JSON text), an `src/notes/` module following the `src/user/`
shape, and bearer-protected CRUD for both folders and notes.

Importantly: a list endpoint returning folder/note metadata *without* bodies, and
a separate fetch for a single note's body. Note bodies are unbounded rich
documents; returning every body to render a sidebar tree would not scale past a
handful of notes.

Frontend: `web/src/api/notes.ts` following `src/api/conversations.ts` (typed
records over `api.get/post/patch/delete`, no URL construction or network
primitives outside that layer), plus React Query hooks alongside
`usePromptQuery` / `useSaveUser`.

**The critical contract:** note bodies are Plate `Value` — a JSON array of
element nodes, not HTML. A backend that stores rendered HTML, or normalizes
bodies through an HTML pass, will silently lose node-level structure — captions,
image width percentages, table cell metadata, code block language tags. Store the
JSON verbatim.

Also worth settling: `serializeRichTextToHtml` is documented as server-only, and
this is an SPA with a Rust backend. If HTML export is ever wanted, it does not
obviously have a home — the Rust service cannot run a JS serializer, and the
browser may not be able to run a function marked server-only. Verify before
promising export as a feature.

**Autosave.** Phase 4 wires `onSave` (Cmd/Ctrl+S) to local state. With a backend,
decide between explicit save and debounced autosave on `onChange`. `onChange`
fires on every keystroke, so autosave needs debouncing plus in-flight request
coalescing, or it will flood the service. Chat's streaming transport already
handles cancellation via `AbortSignal` / `cancel_request` and is worth reading as
precedent.

Blocked on a product decision about what Notes is for — standalone user
documents, or something linked to vault items and chat threads.

## Definition of done (Phases 1–5)

- `/notes` renders behind the auth guard on both targets, reachable from the app
  sidebar
- Folders expand/collapse; selecting a note loads it into the editor
- Editing works, `Cmd/Ctrl+S` fires `onSave`, and content survives switching
  notes and switching back
- No content bleed between notes
- Editor content is styled correctly in light and dark
- The editor bundle is confirmed code-split behind `/notes`
- `npm run lint`, `npm run check:architecture`, `npm run build:web`,
  `npm run build:desktop-ui`, and `npm test` all pass
