"use client"

import { type LucideIcon } from "lucide-react"
import { NavLink, useLocation } from "react-router-dom"

import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"

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
      <SidebarGroupLabel>Platform</SidebarGroupLabel>
      <SidebarMenu>
        {items.map((item) => {
          const isRoute = item.url.startsWith("/")
          const isActive = isRoute && location.pathname === item.url
          return (
            <SidebarMenuItem key={item.title}>
              <SidebarMenuButton tooltip={item.title} isActive={isActive} asChild>
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
