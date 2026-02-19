import * as React from "react"
import curioLogo from "../assets/curio-logo.png"
import { NavUser } from "@/components/nav-user"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"
import ChatNav from "./ui/chat-nav"

// This is sample data.
const data = {
  user: {
    name: "shadcn",
    email: "m@example.com",
    avatar: "/avatars/shadcn.jpg",
  },
}

export function ChatSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        {/* TODO: Fix logo background color, for some reaso its not transparent */}
      <img
        src={curioLogo}
        alt="Curio Logo"
        className="mb-8 w-64 h-auto"
        style={{ mixBlendMode: 'lighten' }}
      />
      </SidebarHeader>
      <SidebarContent>
        <ChatNav />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
