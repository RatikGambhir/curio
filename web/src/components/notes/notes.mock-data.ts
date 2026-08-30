import type { NoteFolder, RichTextValue } from "./notes.types"

// Phase 1 has no backend: these bodies are Plate node arrays, the same shape
// the editor emits from `onChange`, and they reset on reload.

const welcomeNote: RichTextValue = [
  { type: "h1", children: [{ text: "Welcome to Notes" }] },
  {
    type: "p",
    children: [
      { text: "Notes is a " },
      { text: "rich-text", bold: true },
      {
        text: " workspace. Pick a note in the sidebar, then edit it here. Press ",
      },
      { text: "Cmd/Ctrl+S", code: true },
      { text: " to save." },
    ],
  },
  { type: "h2", children: [{ text: "What the toolbar covers" }] },
  {
    type: "p",
    listStyleType: "disc",
    indent: 1,
    children: [{ text: "Marks, headings, quotes, dividers and lists" }],
  },
  {
    type: "p",
    listStyleType: "disc",
    indent: 1,
    children: [{ text: "Links, images with captions, and tables" }],
  },
  {
    type: "p",
    listStyleType: "disc",
    indent: 1,
    children: [{ text: "Code blocks highlighted across fifteen languages" }],
  },
  {
    type: "blockquote",
    children: [
      {
        text: "Nothing here is persisted yet — bodies live in React state until the notes API exists.",
      },
    ],
  },
]

const formattingNote: RichTextValue = [
  { type: "h1", children: [{ text: "Formatting reference" }] },
  {
    type: "p",
    children: [
      { text: "Every node type below round-trips as JSON, never as HTML." },
    ],
  },
  { type: "h3", children: [{ text: "Marks" }] },
  {
    type: "p",
    children: [
      { text: "Bold", bold: true },
      { text: ", " },
      { text: "italic", italic: true },
      { text: ", " },
      { text: "underline", underline: true },
      { text: ", " },
      { text: "strikethrough", strikethrough: true },
      { text: ", and " },
      { text: "inline code", code: true },
      { text: "." },
    ],
  },
  { type: "h3", children: [{ text: "A code block" }] },
  {
    type: "code_block",
    lang: "typescript",
    children: [
      {
        type: "code_line",
        children: [{ text: "export type NoteItem = {" }],
      },
      { type: "code_line", children: [{ text: "  id: string" }] },
      { type: "code_line", children: [{ text: "  title: string" }] },
      { type: "code_line", children: [{ text: "  body: RichTextValue" }] },
      { type: "code_line", children: [{ text: "}" }] },
    ],
  },
  { type: "h3", children: [{ text: "An ordered list" }] },
  // `listStart` is what the ordered-list normalizer would otherwise have to
  // derive on the first edit; setting it keeps the first paint numbered 1-2-3.
  {
    type: "p",
    listStyleType: "decimal",
    indent: 1,
    listStart: 1,
    children: [{ text: "Select a note" }],
  },
  {
    type: "p",
    listStyleType: "decimal",
    indent: 1,
    listStart: 2,
    children: [{ text: "Edit it" }],
  },
  {
    type: "p",
    listStyleType: "decimal",
    indent: 1,
    listStart: 3,
    children: [{ text: "Switch away and back — the content stays put" }],
  },
  { type: "hr", children: [{ text: "" }] },
  {
    type: "p",
    children: [
      { text: "The editor docs live at " },
      {
        type: "a",
        url: "https://ui.ilinxa.com/components/rich-text-editor",
        children: [{ text: "ui.ilinxa.com" }],
      },
      { text: "." },
    ],
  },
]

const longNote: RichTextValue = [
  { type: "h1", children: [{ text: "Architecture read-through" }] },
  ...Array.from({ length: 18 }, (_, index) => ({
    type: "p",
    children: [
      {
        text: `Section ${
          index + 1
        }. The frontend is one React tree built as either a web SPA or the Tauri renderer, so every page has to type-check under both targets and keep its network access behind the typed API layer. This note is deliberately long so the editor's internal scrolling can be checked against the page inset instead of pushing the whole document into a full-page scroll.`,
      },
    ],
  })),
]

const paragraph = (text: string): RichTextValue => [
  { type: "p", children: [{ text }] },
]

export const mockNoteFolders: NoteFolder[] = [
  {
    id: "folder-getting-started",
    name: "Getting started",
    notes: [
      {
        id: "note-welcome",
        title: "Welcome to Notes",
        updatedAt: "2m ago",
        body: welcomeNote,
      },
      {
        id: "note-formatting",
        title: "Formatting reference",
        updatedAt: "1h ago",
        body: formattingNote,
      },
    ],
  },
  {
    id: "folder-product",
    name: "Product",
    notes: [
      {
        id: "note-roadmap",
        title: "Roadmap themes for the next cycle",
        updatedAt: "Yesterday",
        body: paragraph(
          "Vault ingestion, Atlas graph performance, and a persistence story for Notes.",
        ),
      },
      {
        id: "note-interviews",
        title: "User interview notes",
        updatedAt: "2d ago",
        body: paragraph(
          "Researchers keep their reading notes outside the product today, then paste fragments into chat.",
        ),
      },
      {
        id: "note-pricing",
        title: "Pricing questions",
        updatedAt: "5d ago",
        body: paragraph("Open: per-seat or per-vault? Nothing decided yet."),
      },
    ],
  },
  {
    id: "folder-engineering",
    name: "Engineering",
    notes: [
      {
        id: "note-architecture",
        title: "Architecture read-through",
        updatedAt: "3d ago",
        body: longNote,
      },
      {
        id: "note-service",
        title: "curio-service migrations",
        updatedAt: "1w ago",
        body: paragraph(
          "Migrations apply at startup. A notes schema would land as 0003_notes.sql.",
        ),
      },
    ],
  },
  {
    id: "folder-scratch",
    name: "Scratch",
    notes: [
      {
        id: "note-scratch",
        title: "Untitled",
        updatedAt: "Just now",
        body: paragraph(""),
      },
    ],
  },
]
