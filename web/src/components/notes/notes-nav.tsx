import { useEffect, useState } from "react"
import { ChevronRight, FilePlus2, FileText } from "lucide-react"

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"
import { cn } from "@/lib/utils"

import { findFolderIdForNote } from "./notes.types"
import type { NoteFolder } from "./notes.types"

// Deliberately free of any runtime import from `src/components/rich-text-editor/**`:
// the folder list must not drag the editor chunk into its module graph.
type NotesNavProps = {
  folders: NoteFolder[]
  selectedNoteId: string | null
  onSelectNote: (noteId: string) => void
  onCreateNote: (folderId: string) => void
}

const rowClassName =
  "focus-ring relative flex w-full min-w-0 items-center gap-2 rounded-md px-2 text-left transition-colors duration-150 hover:bg-accent"

export function NotesNav({
  folders,
  selectedNoteId,
  onSelectNote,
  onCreateNote,
}: NotesNavProps) {
  // Open the folder holding the selected note, rather than all or nothing.
  const [openFolderIds, setOpenFolderIds] = useState<string[]>(() => {
    const initial = findFolderIdForNote(folders, selectedNoteId)
    return initial ? [initial] : folders.slice(0, 1).map((folder) => folder.id)
  })

  useEffect(() => {
    const folderId = findFolderIdForNote(folders, selectedNoteId)
    if (!folderId) {
      return
    }

    setOpenFolderIds((current) =>
      current.includes(folderId) ? current : [...current, folderId],
    )
  }, [folders, selectedNoteId])

  const toggleFolder = (folderId: string) => {
    setOpenFolderIds((current) =>
      current.includes(folderId)
        ? current.filter((id) => id !== folderId)
        : [...current, folderId],
    )
  }

  return (
    <ul className="flex flex-col gap-0.5 p-2">
      {folders.map((folder) => {
        const isOpen = openFolderIds.includes(folder.id)

        return (
          <li key={folder.id}>
            <Collapsible open={isOpen} onOpenChange={() => toggleFolder(folder.id)}>
              <CollapsibleTrigger className={cn(rowClassName, "h-8 text-sm font-medium text-foreground")}>
                <ChevronRight
                  className={cn(
                    "size-3.5 shrink-0 text-muted-foreground transition-transform duration-200",
                    isOpen && "rotate-90",
                  )}
                  aria-hidden="true"
                />
                <span className="min-w-0 flex-1 truncate">{folder.name}</span>
                <span className="font-mono text-2xs tabular-nums text-muted-foreground">
                  {folder.notes.length}
                </span>
              </CollapsibleTrigger>

              <CollapsibleContent>
                <ul className="mb-1 ml-[0.9375rem] flex flex-col gap-0.5 border-l border-border pl-2 pt-0.5">
                  {folder.notes.map((note) => {
                    const isActive = selectedNoteId === note.id

                    return (
                      <li key={note.id}>
                        <button
                          type="button"
                          onClick={() => onSelectNote(note.id)}
                          aria-current={isActive ? "true" : undefined}
                          className={cn(
                            rowClassName,
                            "h-8 text-[0.8125rem] text-secondary-foreground",
                            "before:absolute before:inset-y-1.5 before:-left-[calc(0.5rem+1px)] before:w-0.5 before:rounded-full before:bg-primary before:opacity-0 before:transition-opacity",
                            isActive &&
                              "bg-accent-subtle/70 text-foreground before:opacity-100 hover:bg-accent-subtle",
                          )}
                        >
                          <FileText
                            className="size-3.5 shrink-0 text-muted-foreground"
                            aria-hidden="true"
                          />
                          <span className="min-w-0 flex-1 truncate">{note.title}</span>
                          <span className="shrink-0 font-mono text-2xs tabular-nums text-muted-foreground">
                            {note.updatedAt}
                          </span>
                        </button>
                      </li>
                    )
                  })}
                  <li>
                    <button
                      type="button"
                      onClick={() => onCreateNote(folder.id)}
                      className={cn(rowClassName, "h-8 text-[0.8125rem] text-muted-foreground hover:text-foreground")}
                    >
                      <FilePlus2 className="size-3.5 shrink-0" aria-hidden="true" />
                      <span className="truncate">New note in {folder.name}</span>
                    </button>
                  </li>
                </ul>
              </CollapsibleContent>
            </Collapsible>
          </li>
        )
      })}
    </ul>
  )
}
