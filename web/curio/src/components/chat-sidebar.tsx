import * as React from "react"
import curioLogo from "../assets/curio-logo.png"
import { Link } from "react-router-dom"
import { NavUser } from "@/components/nav-user"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import { ArrowLeft } from "lucide-react"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
  SidebarTrigger,
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
        <div className="grid grid-cols-[auto_1fr_auto] items-center gap-2 group-data-[collapsible=icon]:grid-cols-1">
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
            className="flex items-center justify-center rounded-md px-2 py-1 outline-hidden ring-sidebar-ring focus-visible:ring-2"
          >
            <img
              src={curioLogo}
              alt="Curio"
              className="h-8 w-auto object-contain transition-all group-data-[collapsible=icon]:h-6"
            />
          </Link>

          <SidebarTrigger className="rounded-full text-muted-foreground hover:bg-white/45 hover:text-foreground group-data-[collapsible=icon]:hidden" />
        </div>
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
