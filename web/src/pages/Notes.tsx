import { useCallback, useState } from "react";
import { FolderTree, NotebookPen, SquarePen } from "lucide-react";

import {
  ContextPane,
  ContextPaneToggle,
} from "@/components/app-shell/context-pane";
import { NotesNav } from "@/components/notes/notes-nav";
import { mockNoteFolders } from "@/components/notes/notes.mock-data";
import { findNote } from "@/components/notes/notes.types";
import type { NoteFolder, RichTextValue } from "@/components/notes/notes.types";
import { PageHeader } from "@/components/page-header";
import {
  RICH_TEXT_EMPTY_VALUE,
  RichTextEditor,
} from "@/components/rich-text-editor";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { useContextPane } from "@/hooks/useContextPane";

const createEmptyNoteBody = (): RichTextValue =>
  structuredClone(RICH_TEXT_EMPTY_VALUE);

const Notes = () => {
  // Phase 1 is in-memory, exactly as Chat runs on demo-data.ts today.
  const [folders, setFolders] = useState<NoteFolder[]>(() =>
    structuredClone(mockNoteFolders),
  );
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const selectedNote = findNote(folders, selectedNoteId);
  const pane = useContextPane();
  const { dismissOverlay } = pane;

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
    dismissOverlay();
  }, [dismissOverlay]);

  const handleSelectNote = useCallback(
    (noteId: string) => {
      setSelectedNoteId(noteId);
      dismissOverlay();
    },
    [dismissOverlay],
  );

  const createInFirstFolder = () => {
    const folderId = folders[0]?.id;
    if (folderId) {
      handleCreateNote(folderId);
    }
  };

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
    <div className="flex min-h-0 flex-1">
      <ContextPane
        pane={pane}
        title="Notebook"
        description="Your note folders."
        action={
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={createInFirstFolder}
            disabled={folders.length === 0}
            className="-mr-1"
          >
            <SquarePen aria-hidden="true" />
            New
          </Button>
        }
      >
        <NotesNav
          folders={folders}
          selectedNoteId={selectedNoteId}
          onSelectNote={handleSelectNote}
          onCreateNote={handleCreateNote}
        />
      </ContextPane>

      <div className="flex min-w-0 flex-1 flex-col">
        <PageHeader
          title={selectedNote?.title ?? "Notes"}
          meta={selectedNote ? `Edited ${selectedNote.updatedAt}` : undefined}
          leading={<ContextPaneToggle pane={pane} label="notebook" overlayIcon={FolderTree} />}
        />
        {selectedNote ? (
          <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
            <RichTextEditor
              // A fresh editor instance per note: selection, undo history
              // and content stay isolated instead of relying on the
              // controlled-value echo guard alone.
              key={selectedNote.id}
              value={selectedNote.body}
              onChange={handleBodyChange}
              onSave={handleSave}
              autoFocus
              className="flex min-h-0 flex-1 flex-col rounded-none border-0 bg-transparent"
              toolbarClassName="shrink-0"
              containerClassName="max-h-none min-h-0 flex-1 overflow-y-auto px-5 py-10 sm:px-10 lg:py-14"
              contentClassName="mx-auto min-h-full max-w-[42rem]"
            />
          </div>
        ) : (
          <EmptyState
            icon={NotebookPen}
            title="Pick a page to start writing"
            action={
              <Button type="button" variant="outline" onClick={createInFirstFolder}>
                <SquarePen aria-hidden="true" />
                New note
              </Button>
            }
            className="min-h-0 flex-1"
          >
            Choose a note from the notebook, or start a fresh one. Notes are
            kept in memory for now and reset when you reload.
          </EmptyState>
        )}
      </div>
    </div>
  );
};

export default Notes;
