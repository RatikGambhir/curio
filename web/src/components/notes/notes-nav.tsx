import { useEffect, useState } from "react"
import { AnimatePresence, motion } from "framer-motion"
import { ChevronRight, FilePlus2, FileText, Folder, PencilLine } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"
import {
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar"

import { findFolderIdForNote } from "./notes.types"
import type { NoteFolder } from "./notes.types"

type NotesNavProps = {
  folders: NoteFolder[]
  selectedNoteId: string | null
  onSelectNote: (noteId: string) => void
  onCreateNote: (folderId: string) => void
}

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
    <SidebarGroup>
      <Button
        type="button"
        onClick={() => {
          const folderId = folders[0]?.id
          if (folderId) {
            onCreateNote(folderId)
          }
        }}
        variant="ghost"
        className="mb-2 h-9 w-full justify-start rounded-md border border-transparent bg-transparent px-2 text-sm font-normal text-sidebar-foreground shadow-none hover:translate-y-0 hover:bg-sidebar-foreground/[0.035] hover:text-sidebar-foreground group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0"
      >
        <PencilLine className="size-4" />
        <span className="group-data-[collapsible=icon]:hidden">New note</span>
      </Button>
      <SidebarGroupLabel>Folders</SidebarGroupLabel>
      <SidebarGroupContent>
        <SidebarMenu className="gap-0.5">
          {folders.map((folder) => {
            const isOpen = openFolderIds.includes(folder.id)

            return (
              <Collapsible
                key={folder.id}
                open={isOpen}
                onOpenChange={() => toggleFolder(folder.id)}
                className="group/folder"
              >
                <SidebarMenuItem>
                  <CollapsibleTrigger asChild>
                    <SidebarMenuButton
                      type="button"
                      tooltip={folder.name}
                      className="rounded-md border border-transparent bg-transparent px-2 text-sidebar-foreground shadow-none hover:translate-y-0 hover:bg-sidebar-foreground/[0.035] hover:text-sidebar-foreground group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-2"
                    >
                      <Folder className="size-4 shrink-0" />
                      <span className="truncate group-data-[collapsible=icon]:hidden">
                        {folder.name}
                      </span>
                      <span className="ml-auto flex items-center gap-1 group-data-[collapsible=icon]:hidden">
                        <span className="text-[10px] text-sidebar-foreground/60">
                          {folder.notes.length}
                        </span>
                        <ChevronRight
                          className={`size-3.5 transition-transform ${
                            isOpen ? "rotate-90" : ""
                          }`}
                        />
                      </span>
                    </SidebarMenuButton>
                  </CollapsibleTrigger>

                  <CollapsibleContent>
                    <SidebarMenuSub className="mt-0.5 gap-0.5">
                      <AnimatePresence initial={false}>
                        {folder.notes.map((note) => (
                          <motion.div
                            key={note.id}
                            layout
                            initial={{ opacity: 0, x: -10 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: -8 }}
                            transition={{ duration: 0.2, ease: [0.22, 1, 0.36, 1] }}
                          >
                            <SidebarMenuSubItem>
                              <SidebarMenuSubButton
                                asChild
                                isActive={selectedNoteId === note.id}
                                className="h-auto w-full items-center rounded-md border border-transparent bg-transparent py-1.5 hover:-translate-y-0 hover:bg-sidebar-foreground/[0.035] data-[active=true]:border-sidebar-foreground/40 data-[active=true]:!bg-transparent data-[active=true]:!text-sidebar-foreground data-[active=true]:ring-1 data-[active=true]:ring-sidebar-foreground/10"
                              >
                                <button
                                  type="button"
                                  onClick={() => onSelectNote(note.id)}
                                >
                                  <FileText className="size-3.5 shrink-0" />
                                  <span className="min-w-0 flex-1 truncate">
                                    {note.title}
                                  </span>
                                  <span className="shrink-0 text-[10px] text-sidebar-foreground/60">
                                    {note.updatedAt}
                                  </span>
                                </button>
                              </SidebarMenuSubButton>
                            </SidebarMenuSubItem>
                          </motion.div>
                        ))}
                      </AnimatePresence>

                      <SidebarMenuSubItem>
                        <SidebarMenuSubButton
                          asChild
                          className="w-full text-sidebar-foreground/70 hover:-translate-y-0 hover:bg-sidebar-foreground/[0.035]"
                        >
                          <button
                            type="button"
                            onClick={() => onCreateNote(folder.id)}
                          >
                            <FilePlus2 className="size-3.5 shrink-0" />
                            <span className="truncate">New note</span>
                          </button>
                        </SidebarMenuSubButton>
                      </SidebarMenuSubItem>
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </SidebarMenuItem>
              </Collapsible>
            )
          })}
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>
  )
}
