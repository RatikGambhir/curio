import { useMemo } from "react"
import { useNavigate } from "react-router-dom"
import { ChevronsUpDown, LogOut, Palette, Settings2, UserRound } from "lucide-react"

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { useSidebar } from "@/components/ui/sidebar-context"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"
import { initialsFor } from "@/lib/initials"

export function NavUser() {
  const { isMobile } = useSidebar()
  const navigate = useNavigate()
  const { user, logoutUser } = useAuthenticatedUser()
  const initials = useMemo(() => initialsFor(user?.name), [user?.name])

  if (!user) {
    return null
  }

  const handleSignOut = () => {
    logoutUser()
    navigate("/login", { replace: true })
  }

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton
              size="lg"
              tooltip={user.name}
              className="h-11 gap-2.5 px-1.5 data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-foreground group-data-[collapsible=icon]:p-1!"
            >
              <Avatar className="size-7 rounded-md">
                <AvatarImage src={user.avatar} alt="" />
                <AvatarFallback className="rounded-md bg-sidebar-accent font-mono text-2xs font-medium text-sidebar-foreground">
                  {initials}
                </AvatarFallback>
              </Avatar>
              <span className="grid min-w-0 flex-1 text-left leading-tight">
                <span className="truncate text-sm font-medium text-sidebar-foreground">
                  {user.name}
                </span>
                <span className="truncate text-xs text-sidebar-muted-foreground">
                  {user.email}
                </span>
              </span>
              <ChevronsUpDown className="ml-auto size-3.5!" aria-hidden="true" />
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            className="w-(--radix-dropdown-menu-trigger-width) min-w-60"
            side={isMobile ? "top" : "right"}
            align="end"
            sideOffset={8}
          >
            <DropdownMenuLabel className="px-2 py-2 font-normal">
              <span className="block truncate text-sm font-medium">{user.name}</span>
              <span className="block truncate text-xs text-muted-foreground">
                {user.email}
              </span>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuItem onSelect={() => navigate("/settings?tab=account")}>
                <UserRound />
                Account
              </DropdownMenuItem>
              <DropdownMenuItem
                onSelect={() => navigate("/settings?tab=customization")}
              >
                <Palette />
                Appearance
              </DropdownMenuItem>
              <DropdownMenuItem onSelect={() => navigate("/settings")}>
                <Settings2 />
                All settings
              </DropdownMenuItem>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuItem onSelect={handleSignOut}>
              <LogOut />
              Sign out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}
