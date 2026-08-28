import { useState } from "react"
import { NotebookPen } from "lucide-react"

import { NotesSidebar } from "@/components/notes/notes-sidebar"
import { mockNoteFolders } from "@/components/notes/notes.mock-data"
import { findNote } from "@/components/notes/notes.types"
import type { NoteFolder, RichTextValue } from "@/components/notes/notes.types"
import { PageHeader } from "@/components/page-header"
import {
  RichTextEditor,
  emptyRichTextValue,
} from "@/components/rich-text-editor"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"

const Notes = () => {
  // Phase 1 is in-memory, exactly as Chat runs on demo-data.ts today.
  const [folders, setFolders] = useState<NoteFolder[]>(mockNoteFolders)
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null)
  const selectedNote = findNote(folders, selectedNoteId)

  const patchNote = (
    noteId: string,
    patch: { body?: RichTextValue; updatedAt?: string },
  ) => {
    setFolders((currentFolders) =>
      currentFolders.map((folder) => ({
        ...folder,
        notes: folder.notes.map((note) =>
          note.id === noteId ? { ...note, ...patch } : note,
        ),
      })),
    )
  }

  const handleCreateNote = (folderId: string) => {
    const noteId = crypto.randomUUID()

    setFolders((currentFolders) =>
      currentFolders.map((folder) =>
        folder.id === folderId
          ? {
              ...folder,
              notes: [
                {
                  id: noteId,
                  title: "Untitled",
                  updatedAt: "Just now",
                  body: emptyRichTextValue(),
                },
                ...folder.notes,
              ],
            }
          : folder,
      ),
    )
    setSelectedNoteId(noteId)
  }

  return (
    <SidebarProvider className="h-screen w-full font-sans">
      <NotesSidebar
        folders={folders}
        selectedNoteId={selectedNoteId}
        onSelectNote={setSelectedNoteId}
        onCreateNote={handleCreateNote}
      />
      <SidebarInset className="bg-background">
        <div className="flex h-full w-full flex-col bg-background">
          {selectedNote ? (
            // The editor owns the whole inset: its toolbar stands in for the
            // page header rather than sitting in a card below one.
            <RichTextEditor
              // A fresh editor instance per note: selection, undo history
              // and content stay isolated instead of relying on the
              // controlled-value echo guard alone.
              key={selectedNote.id}
              flush
              header={
                <h1 className="max-w-[16rem] truncate text-sm font-semibold text-foreground">
                  {selectedNote.title}
                </h1>
              }
              headerTrailing={
                <span className="text-xs whitespace-nowrap text-muted-foreground">
                  Edited {selectedNote.updatedAt}
                </span>
              }
              value={selectedNote.body}
              onChange={(body) => patchNote(selectedNote.id, { body })}
              onSave={(body) =>
                patchNote(selectedNote.id, { body, updatedAt: "Just now" })
              }
            />
          ) : (
            <>
              <PageHeader />
              <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden bg-background px-4 py-5 md:px-8 md:py-6">
                <div className="flex h-full w-full flex-col items-center justify-center gap-3 text-center">
                  <div className="flex size-12 items-center justify-center rounded-full border border-border bg-card text-muted-foreground">
                    <NotebookPen className="size-5" />
                  </div>
                  <h1 className="text-lg font-semibold text-foreground">
                    Pick a note to start writing
                  </h1>
                  <p className="max-w-md text-sm text-muted-foreground">
                    Choose a note from a folder on the left, or create a new
                    one. Notes are kept in memory for now and reset on reload.
                  </p>
                </div>
              </div>
            </>
          )}
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}

export default Notes
