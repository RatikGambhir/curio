import { ChevronRight, Search, X } from "lucide-react"
import type { ComponentType } from "react"
import { Link, NavLink, useLocation } from "react-router-dom"

import { CurioMark } from "@/components/brand/curio-mark"
import { EmptyState } from "@/components/ui/empty-state"
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { cn } from "@/lib/utils"

export type AppNavigationIcon = ComponentType<{
  className?: string
  size?: number
  "aria-hidden"?: boolean | "true" | "false"
}>

export type AppNavigationLink = {
  title: string
  url: string
  icon: AppNavigationIcon
  description: string
}

export type AppNavigationGroup = {
  id: string
  title: string
  icon: AppNavigationIcon
  items: AppNavigationLink[]
  /** Shown in the submenu while the group has no entries. */
  emptyMessage: string
}

export type AppNavigationSection = {
  /** Omitted for a lead section that needs no heading. */
  label?: string
  items: Array<AppNavigationLink | AppNavigationGroup>
}

function isNavigationGroup(
  item: AppNavigationLink | AppNavigationGroup,
): item is AppNavigationGroup {
  return "items" in item
}

export function PlatformHeader() {
  return (
    <div className="flex h-full w-full min-w-0 items-center gap-2 px-3 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0">
      <Link
        to="/home"
        aria-label="Curio home"
        className="focus-ring flex min-w-0 flex-1 items-center gap-2.5 rounded-md py-1 pl-1 text-sidebar-foreground group-data-[collapsible=icon]:hidden"
      >
        <CurioMark className="size-[1.375rem] text-sidebar-foreground" />
        <span className="font-display text-[1.375rem] leading-none tracking-[-0.01em]">
          Curio
        </span>
      </Link>
      <SidebarTrigger className="ml-auto shrink-0 text-sidebar-muted-foreground hover:bg-sidebar-hover hover:text-sidebar-foreground group-data-[collapsible=icon]:mx-auto" />
    </div>
  )
}

export function SidebarSearch() {
  return (
    <SidebarGroup className="pb-2 pt-3 group-data-[collapsible=icon]:px-2">
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton
            asChild
            variant="outline"
            tooltip="Search knowledge"
            className="h-9 hover:bg-sidebar-accent"
          >
            <Link to="/vault">
              <Search aria-hidden="true" />
              <span>Search knowledge</span>
            </Link>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup>
  )
}

export function NavMain({
  sections,
  openGroupId,
  onGroupToggle,
  onNavigate,
}: {
  sections: AppNavigationSection[]
  openGroupId: string | null
  onGroupToggle: (groupId: string) => void
  onNavigate: () => void
}) {
  const location = useLocation()

  return sections.map((section, index) => (
    <SidebarGroup key={section.label ?? index} className="py-1.5">
      {section.label ? (
        <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
      ) : null}
      <SidebarMenu>
        {section.items.map((item) => {
          if (isNavigationGroup(item)) {
            const isOpen = openGroupId === item.id

            return (
              <SidebarMenuItem key={item.id}>
                <SidebarMenuButton
                  type="button"
                  tooltip={item.title}
                  aria-expanded={isOpen}
                  aria-controls={`sidebar-submenu-${item.id}`}
                  onClick={() => onGroupToggle(item.id)}
                >
                  <item.icon aria-hidden="true" size={16} />
                  <span>{item.title}</span>
                  <ChevronRight
                    aria-hidden="true"
                    className={cn(
                      "ml-auto size-3.5! transition-transform duration-200 group-data-[collapsible=icon]:hidden",
                      isOpen && "rotate-180",
                    )}
                  />
                </SidebarMenuButton>
              </SidebarMenuItem>
            )
          }

          const isActive =
            location.pathname === item.url ||
            location.pathname.startsWith(`${item.url}/`)

          return (
            <SidebarMenuItem key={item.url}>
              <SidebarMenuButton tooltip={item.title} isActive={isActive} asChild>
                <NavLink to={item.url} onClick={onNavigate}>
                  <item.icon aria-hidden="true" size={16} />
                  <span>{item.title}</span>
                </NavLink>
              </SidebarMenuButton>
            </SidebarMenuItem>
          )
        })}
      </SidebarMenu>
    </SidebarGroup>
  ))
}

export function NavSubmenu({
  group,
  onBack,
  className,
}: {
  group: AppNavigationGroup
  onBack: () => void
  className?: string
}) {
  return (
    <div
      id={`sidebar-submenu-${group.id}`}
      className={cn("flex h-full min-h-0 flex-col", className)}
    >
      <div className="flex h-(--app-header-height) shrink-0 items-center gap-2 border-b border-border pl-4 pr-3">
        <h2 className="min-w-0 flex-1 truncate font-display text-[1.25rem] leading-none text-foreground">
          {group.title}
        </h2>
        <button
          type="button"
          onClick={onBack}
          className="focus-ring flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        >
          <X className="size-4" aria-hidden="true" />
          <span className="sr-only">Close {group.title}</span>
        </button>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto p-2">
        {group.items.length > 0 ? (
          <ul className="flex flex-col gap-0.5">
            {group.items.map((item) => (
              <li key={item.url}>
                <Link
                  to={item.url}
                  onClick={onBack}
                  className="focus-ring flex items-start gap-3 rounded-md px-2.5 py-2 transition-colors hover:bg-accent"
                >
                  <item.icon aria-hidden="true" size={16} />
                  <span className="min-w-0">
                    <span className="block text-sm font-medium">{item.title}</span>
                    <span className="block text-xs text-muted-foreground">
                      {item.description}
                    </span>
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState title={`Nothing in ${group.title} yet`} className="py-16">
            {group.emptyMessage}
          </EmptyState>
        )}
      </div>
    </div>
  )
}
