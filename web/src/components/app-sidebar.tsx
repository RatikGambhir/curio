import { useCallback, useEffect, useState, type ComponentProps } from "react"
import {
  BookOpen,
  CalendarDays,
  Globe,
  House,
  Inbox,
  LibraryBig,
  MessageSquare,
  NotebookPen,
} from "lucide-react"
import { useLocation } from "react-router-dom"

import {
  NavMain,
  NavSubmenu,
  PlatformHeader,
  SidebarSearch,
  type AppNavigationGroup,
  type AppNavigationSection,
} from "@/components/nav-main"
import { NavUser } from "@/components/nav-user"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"
import { useSidebar } from "@/components/ui/sidebar-context"

const appNavigationSections: AppNavigationSection[] = [
  {
    items: [
      {
        title: "Home",
        url: "/home",
        icon: House,
        description: "Your daily Curio overview.",
      },
    ],
  },
  {
    label: "Workspace",
    items: [
      {
        id: "inbox",
        title: "Inbox",
        icon: Inbox,
        items: [],
        emptyMessage:
          "Messages and shared items will collect here once Inbox is connected.",
      },
      {
        title: "Notes",
        url: "/notes",
        icon: NotebookPen,
        description: "Capture and shape what you learn.",
      },
      {
        title: "Calendar",
        url: "/calendar",
        icon: CalendarDays,
        description: "Plan events and track tasks.",
      },
    ],
  },
  {
    label: "Knowledge",
    items: [
      {
        id: "explore",
        title: "Explore",
        icon: LibraryBig,
        items: [],
        emptyMessage:
          "Collections and suggested reading will appear here as your vault grows.",
      },
      {
        title: "Vault",
        url: "/vault",
        icon: BookOpen,
        description: "Browse your saved knowledge.",
      },
    ],
  },
  {
    label: "Ask AI",
    items: [
      {
        title: "Chat",
        url: "/chat",
        icon: MessageSquare,
        description: "Ask questions and explore ideas.",
      },
      {
        title: "Atlas",
        url: "/atlas",
        icon: Globe,
        description: "See how your ideas connect.",
      },
    ],
  },
]

const navigationGroups = appNavigationSections
  .flatMap((section) => section.items)
  .filter((item): item is AppNavigationGroup => "items" in item)

export function AppSidebar({ ...props }: ComponentProps<typeof Sidebar>) {
  const location = useLocation()
  const { isMobile, state } = useSidebar()
  const [openGroupId, setOpenGroupId] = useState<string | null>(null)
  const openGroup =
    navigationGroups.find((group) => group.id === openGroupId) ?? null

  useEffect(() => {
    setOpenGroupId(null)
  }, [location.pathname])

  useEffect(() => {
    if (!openGroup) {
      return
    }

    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpenGroupId(null)
      }
    }

    window.addEventListener("keydown", closeOnEscape)
    return () => window.removeEventListener("keydown", closeOnEscape)
  }, [openGroup])

  const closeSubmenu = useCallback(() => setOpenGroupId(null), [])
  const toggleGroup = useCallback((groupId: string) => {
    setOpenGroupId((current) => (current === groupId ? null : groupId))
  }, [])

  return (
    <>
      <Sidebar collapsible="icon" {...props}>
        <SidebarHeader className="h-(--app-header-height) shrink-0 justify-center border-b border-sidebar-border p-0">
          <PlatformHeader />
        </SidebarHeader>
        <SidebarContent className="gap-0 pt-0">
          {isMobile && openGroup ? (
            <div className="min-h-0 flex-1 bg-card text-card-foreground">
              <NavSubmenu group={openGroup} onBack={closeSubmenu} />
            </div>
          ) : (
            <>
              <SidebarSearch />
              <NavMain
                sections={appNavigationSections}
                openGroupId={openGroupId}
                onGroupToggle={toggleGroup}
                onNavigate={closeSubmenu}
              />
            </>
          )}
        </SidebarContent>
        <SidebarFooter className="border-t border-sidebar-border p-2">
          <NavUser />
        </SidebarFooter>
        <SidebarRail />
      </Sidebar>

      {!isMobile && openGroup ? (
        <aside
          aria-label={`${openGroup.title} submenu`}
          className="fixed inset-y-0 z-30 w-[19rem] animate-in overflow-hidden rounded-r-xl border-r border-border bg-card text-card-foreground shadow-xl duration-200 fade-in-0 slide-in-from-left-2"
          style={{
            left:
              state === "collapsed"
                ? "var(--sidebar-width-icon)"
                : "var(--sidebar-width)",
          }}
        >
          <NavSubmenu group={openGroup} onBack={closeSubmenu} />
        </aside>
      ) : null}
    </>
  )
}
