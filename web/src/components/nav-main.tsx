"use client"

import { ChevronLeft, ChevronRight, Search, Shapes } from "lucide-react"
import type { ComponentType } from "react"
import { Link, NavLink, useLocation } from "react-router-dom"

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
}

export type AppNavigationSection = {
  label: string
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
        aria-label="Go to Curio home"
        className="flex min-w-0 flex-1 items-center gap-2 rounded-md outline-none focus-visible:ring-2 focus-visible:ring-sidebar-ring group-data-[collapsible=icon]:hidden"
      >
        <span className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground shadow-xs">
          <Shapes className="size-4" aria-hidden="true" />
        </span>
        <span className="grid min-w-0 flex-1 text-left leading-tight">
          <span className="truncate text-sm font-semibold">Curio</span>
          <span className="truncate text-[0.6875rem] text-sidebar-foreground/65">
            Knowledge workspace
          </span>
        </span>
      </Link>
      <SidebarTrigger className="ml-auto shrink-0 text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-foreground group-data-[collapsible=icon]:mx-auto" />
    </div>
  )
}

export function SidebarSearch() {
  return (
    <SidebarGroup className="pb-1 pt-2 group-data-[collapsible=icon]:px-2">
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton
            asChild
            tooltip="Search knowledge"
            className="h-9 border border-sidebar-border bg-card/70 shadow-xs hover:bg-card group-data-[collapsible=icon]:size-8!"
          >
            <Link to="/vault" aria-label="Search knowledge">
              <Search aria-hidden="true" />
              <span className="text-sidebar-foreground/65">Search knowledge…</span>
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

  return sections.map((section) => (
    <SidebarGroup key={section.label}>
      <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
      <SidebarMenu>
        {section.items.map((item) => {
          if (isNavigationGroup(item)) {
            const isOpen = openGroupId === item.id

            return (
              <SidebarMenuItem key={item.id}>
                <SidebarMenuButton
                  type="button"
                  tooltip={item.title}
                  isActive={isOpen}
                  aria-expanded={isOpen}
                  aria-controls={`sidebar-submenu-${item.id}`}
                  onClick={() => onGroupToggle(item.id)}
                >
                  <item.icon aria-hidden="true" size={16} />
                  <span>{item.title}</span>
                  <ChevronRight
                    aria-hidden="true"
                    className={cn(
                      "ml-auto transition-transform group-data-[collapsible=icon]:hidden",
                      isOpen && "rotate-180",
                    )}
                  />
                </SidebarMenuButton>
              </SidebarMenuItem>
            )
          }

          return (
            <SidebarMenuItem key={item.url}>
              <SidebarMenuButton
                tooltip={item.title}
                isActive={location.pathname === item.url}
                asChild
              >
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
      <div className="flex h-(--app-header-height) shrink-0 items-center border-b border-border px-3">
        <button
          type="button"
          onClick={onBack}
          className="flex size-7 shrink-0 items-center justify-center rounded-md text-muted-foreground outline-none transition-colors hover:bg-accent hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        >
          <ChevronLeft className="size-4" aria-hidden="true" />
          <span className="sr-only">Back to main navigation</span>
        </button>
        <h2 className="ml-2 truncate text-sm font-semibold text-foreground">
          {group.title}
        </h2>
      </div>
      <div className="min-h-0 flex-1" />
    </div>
  )
}
