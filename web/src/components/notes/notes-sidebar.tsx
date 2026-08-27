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
  SidebarTrigger,
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
      <SidebarHeader className="h-16 shrink-0 justify-center border-b border-sidebar-border p-3">
        <div className="grid grid-cols-[auto_1fr_auto] items-center gap-2 group-data-[collapsible=icon]:grid-cols-1 group-data-[collapsible=icon]:place-items-center">
          <Button
            asChild
            type="button"
            variant="ghost"
            size="icon"
            className="rounded-full text-muted-foreground hover:bg-white/45 hover:text-foreground group-data-[collapsible=icon]:hidden"
          >
            <Link to="/home" aria-label="Go to home">
              <ArrowLeft className="size-5" />
            </Link>
          </Button>

          <Link
            to="/home"
            aria-label="Go to home"
            className="flex items-center justify-center rounded-md px-2 py-1 outline-hidden ring-sidebar-ring focus-visible:ring-2 group-data-[collapsible=icon]:hidden"
          >
            <img
              src={curioLogo}
              alt="Curio"
              className="h-8 w-auto object-contain transition-all group-data-[collapsible=icon]:h-6"
            />
          </Link>

          <SidebarTrigger className="rounded-full text-muted-foreground hover:bg-white/45 hover:text-foreground" />
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
