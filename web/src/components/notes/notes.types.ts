import type { RichTextValue } from "@/components/rich-text-editor"

/**
 * Re-exported so no other app file reaches into
 * `src/components/rich-text-editor/**` directly — the vendored editor tree can
 * then change shape without touching Notes.
 */
export type { RichTextValue }

export type NoteItem = {
  id: string
  title: string
  /** Pre-formatted for display, as `ChatListItem` does. */
  updatedAt: string
  /** Plate node array. Never an HTML string. */
  body: RichTextValue
}

export type NoteFolder = {
  id: string
  name: string
  notes: NoteItem[]
}

export function findNote(
  folders: readonly NoteFolder[],
  noteId: string | null,
): NoteItem | undefined {
  if (!noteId) {
    return undefined
  }

  for (const folder of folders) {
    const note = folder.notes.find((candidate) => candidate.id === noteId)
    if (note) {
      return note
    }
  }

  return undefined
}

export function findFolderIdForNote(
  folders: readonly NoteFolder[],
  noteId: string | null,
): string | undefined {
  if (!noteId) {
    return undefined
  }

  return folders.find((folder) =>
    folder.notes.some((note) => note.id === noteId),
  )?.id
}
