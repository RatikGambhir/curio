import * as React from "react"
import curioLogo from "../assets/curio-logo.png"
import { Link } from "react-router-dom"
import { NavUser } from "@/components/nav-user"
import { cn } from "@/lib/utils"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar"
import ChatNav, { type ChatListItem } from "./ui/chat-nav"

// This is sample data.
const data = {
  user: {
    name: "shadcn",
    email: "m@example.com",
    avatar: "/avatars/shadcn.jpg",
  },
}

type ChatSidebarProps = React.ComponentProps<typeof Sidebar> & {
  chats: ChatListItem[]
  selectedChatId: string | null
  onSelectChat: (chatId: string) => void
}

export function ChatSidebar({
  chats,
  selectedChatId,
  onSelectChat,
  className,
  ...props
}: ChatSidebarProps) {
  return (
    <Sidebar
      collapsible="icon"
      {...props}
      className={cn(
        "[&_[data-slot=sidebar-inner]]:bg-background",
        className,
      )}
    >
      <SidebarHeader className="border-b border-sidebar-border p-3">
        <Link
          to="/home"
          aria-label="Go to home"
          className="flex items-center justify-center rounded-md px-2 py-1 outline-hidden ring-sidebar-ring focus-visible:ring-2"
        >
          <img
            src={curioLogo}
            alt="Curio"
            className="h-8 w-auto object-contain transition-all group-data-[collapsible=icon]:h-6"
          />
        </Link>
      </SidebarHeader>
      <SidebarContent>
        <ChatNav
          chats={chats}
          selectedChatId={selectedChatId}
          onSelectChat={onSelectChat}
        />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
