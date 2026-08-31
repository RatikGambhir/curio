import * as React from "react"
import { Link } from "react-router-dom"
import { ArrowLeft } from "lucide-react"

import curioLogo from "../../assets/curio-logo.png"
import { NavUser } from "@/components/nav-user"
import { Button } from "@/components/ui/button"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"

import { NotesNav } from "./notes-nav"
import type { NoteFolder } from "./notes.types"

// Deliberately free of any import from `src/components/rich-text-editor/**`:
// the sidebar must not drag the editor chunk into its module graph.
type NotesSidebarProps = React.ComponentProps<typeof Sidebar> & {
  folders: NoteFolder[]
  selectedNoteId: string | null
  onSelectNote: (noteId: string) => void
  onCreateNote: (folderId: string) => void
}

export function NotesSidebar({
  folders,
  selectedNoteId,
  onSelectNote,
  onCreateNote,
  className,
  ...props
}: NotesSidebarProps) {
  return (
    <Sidebar collapsible="icon" {...props} className={className}>
      <SidebarHeader className="h-(--app-header-height) shrink-0 justify-center p-2">
        <div className="flex items-center gap-1 group-data-[collapsible=icon]:justify-center">
          <Button
            asChild
            type="button"
            variant="ghost"
            size="icon"
            className="text-muted-foreground hover:bg-sidebar-accent hover:text-foreground group-data-[collapsible=icon]:hidden"
          >
            <Link to="/home" aria-label="Go to home">
              <ArrowLeft className="size-5" />
            </Link>
          </Button>

          <Link
            to="/home"
            aria-label="Go to home"
            className="flex items-center justify-center rounded-md px-2 py-1 outline-hidden ring-sidebar-ring focus-visible:ring-2"
          >
            <img
              src={curioLogo}
              alt="Curio"
              className="h-7 w-auto object-contain"
            />
          </Link>
        </div>
      </SidebarHeader>
      <SidebarContent>
        <NotesNav
          folders={folders}
          selectedNoteId={selectedNoteId}
          onSelectNote={onSelectNote}
          onCreateNote={onCreateNote}
        />
      </SidebarContent>
      <SidebarFooter>
        <NavUser />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
