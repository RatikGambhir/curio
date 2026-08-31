import { useCallback, useState } from "react";
import { NotebookPen } from "lucide-react";

import { NotesSidebar } from "@/components/notes/notes-sidebar";
import { mockNoteFolders } from "@/components/notes/notes.mock-data";
import { findNote } from "@/components/notes/notes.types";
import type { NoteFolder, RichTextValue } from "@/components/notes/notes.types";
import { PageHeader } from "@/components/page-header";
import {
  RICH_TEXT_EMPTY_VALUE,
  RichTextEditor,
} from "@/components/rich-text-editor";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";

const createEmptyNoteBody = (): RichTextValue =>
  structuredClone(RICH_TEXT_EMPTY_VALUE);

const Notes = () => {
  // Phase 1 is in-memory, exactly as Chat runs on demo-data.ts today.
  const [folders, setFolders] = useState<NoteFolder[]>(() =>
    structuredClone(mockNoteFolders),
  );
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const selectedNote = findNote(folders, selectedNoteId);

  const patchNote = useCallback(
    (noteId: string, patch: { body?: RichTextValue; updatedAt?: string }) => {
      setFolders((currentFolders) =>
        currentFolders.map((folder) => ({
          ...folder,
          notes: folder.notes.map((note) =>
            note.id === noteId ? { ...note, ...patch } : note,
          ),
        })),
      );
    },
    [],
  );

  const handleCreateNote = useCallback((folderId: string) => {
    const noteId = crypto.randomUUID();

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
                  body: createEmptyNoteBody(),
                },
                ...folder.notes,
              ],
            }
          : folder,
      ),
    );
    setSelectedNoteId(noteId);
  }, []);

  const handleBodyChange = useCallback(
    (body: RichTextValue) => {
      if (selectedNoteId) {
        patchNote(selectedNoteId, { body });
      }
    },
    [patchNote, selectedNoteId],
  );

  const handleSave = useCallback(
    (body: RichTextValue) => {
      if (selectedNoteId) {
        patchNote(selectedNoteId, { body, updatedAt: "Just now" });
      }
    },
    [patchNote, selectedNoteId],
  );

  return (
    <SidebarProvider className="h-screen w-full font-sans">
      <NotesSidebar
        folders={folders}
        selectedNoteId={selectedNoteId}
        onSelectNote={setSelectedNoteId}
        onCreateNote={handleCreateNote}
      />
      <SidebarInset>
        <div className="flex h-full w-full flex-col">
          <PageHeader>
            <div className="min-w-0">
              <h1 className="truncate text-sm font-semibold text-foreground">
                {selectedNote?.title ?? "Notes"}
              </h1>
              {selectedNote ? (
                <p className="text-xs text-muted-foreground">
                  Edited {selectedNote.updatedAt}
                </p>
              ) : null}
            </div>
          </PageHeader>
          {selectedNote ? (
            <main className="flex min-h-0 flex-1 flex-col overflow-hidden">
              <RichTextEditor
                // A fresh editor instance per note: selection, undo history
                // and content stay isolated instead of relying on the
                // controlled-value echo guard alone.
                key={selectedNote.id}
                value={selectedNote.body}
                onChange={handleBodyChange}
                onSave={handleSave}
                autoFocus
                className="flex min-h-0 flex-1 flex-col rounded-none border-0"
                toolbarClassName="shrink-0"
                containerClassName="max-h-none min-h-0 flex-1 overflow-y-auto px-6 py-6 md:px-10"
                contentClassName="mx-auto min-h-full max-w-4xl"
              />
            </main>
          ) : (
            <main className="relative flex min-h-0 flex-1 flex-col overflow-hidden px-4 py-5 md:px-8 md:py-6">
              <div className="flex h-full w-full flex-col items-center justify-center gap-3 text-center">
                <div className="flex size-12 items-center justify-center rounded-full border border-border bg-card text-muted-foreground">
                  <NotebookPen className="size-5" />
                </div>
                <h1 className="text-lg font-semibold text-foreground">
                  Pick a note to start writing
                </h1>
                <p className="max-w-md text-sm text-muted-foreground">
                  Choose a note from a folder on the left, or create a new one.
                  Notes are kept in memory for now and reset on reload.
                </p>
              </div>
            </main>
          )}
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Notes;
