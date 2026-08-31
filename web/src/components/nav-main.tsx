"use client"

import { Search, SquarePen, type LucideIcon } from "lucide-react"
import { Link, NavLink, useLocation } from "react-router-dom"

import {
  SidebarGroup,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarTrigger,
} from "@/components/ui/sidebar"

export function PlatformHeader() {
  return (
    <div className="flex h-full w-full items-center justify-between gap-2 px-3 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0">
      <span className="truncate text-xs font-medium text-sidebar-foreground/70 group-data-[collapsible=icon]:hidden">
        Platform
      </span>
      <span className="flex items-center gap-1">
        <Link
          to="/vault"
          aria-label="Search vault"
          className="flex size-6 shrink-0 items-center justify-center rounded-full bg-sidebar-accent text-sidebar-foreground outline-none transition-colors hover:bg-sidebar-accent/80 focus-visible:ring-2 focus-visible:ring-sidebar-ring group-data-[collapsible=icon]:hidden [&_svg]:size-3.5 [&_svg]:shrink-0"
        >
          <Search />
        </Link>
        <Link
          to="/chat"
          aria-label="New chat"
          className="flex size-6 shrink-0 items-center justify-center rounded-full border border-border bg-card text-sidebar-foreground shadow-xs outline-none transition-colors hover:bg-accent focus-visible:ring-2 focus-visible:ring-sidebar-ring [&_svg]:size-3.5 [&_svg]:shrink-0"
        >
          <SquarePen />
        </Link>
        <SidebarTrigger className="md:hidden" />
      </span>
    </div>
  )
}

export function NavMain({
  items,
}: {
  items: {
    title: string
    url: string
    icon?: LucideIcon
  }[]
}) {
  const location = useLocation()

  return (
    <SidebarGroup>
      <SidebarMenu>
        {items.map((item) => {
          const isRoute = item.url.startsWith("/")
          const isActive = isRoute && location.pathname === item.url
          return (
            <SidebarMenuItem key={item.title}>
              <SidebarMenuButton
                tooltip={item.title}
                isActive={isActive}
                asChild
              >
                {isRoute ? (
                  <NavLink to={item.url}>
                    {item.icon && <item.icon />}
                    <span>{item.title}</span>
                  </NavLink>
                ) : (
                  <a href={item.url}>
                    {item.icon && <item.icon />}
                    <span>{item.title}</span>
                  </a>
                )}
              </SidebarMenuButton>
            </SidebarMenuItem>
          )
        })}
      </SidebarMenu>
    </SidebarGroup>
  )
}
