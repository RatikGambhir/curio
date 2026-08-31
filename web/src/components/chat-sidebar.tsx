import * as React from "react"
import curioLogo from "../assets/curio-logo.png"
import { Link } from "react-router-dom"
import { NavUser } from "@/components/nav-user"
import { Button } from "@/components/ui/button"
import { ArrowLeft } from "lucide-react"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import type { ChatListItem } from "@/features/chat/types"
import ChatNav from "./ui/chat-nav"

type ChatSidebarProps = React.ComponentProps<typeof Sidebar> & {
  chats: ChatListItem[]
  selectedChatId: string | null
  isNewChat: boolean
  onSelectChat: (chatId: string) => void
  onStartNewChat: () => void
}

export function ChatSidebar({
  chats,
  selectedChatId,
  isNewChat,
  onSelectChat,
  onStartNewChat,
  className,
  ...props
}: ChatSidebarProps) {
  return (
    <Sidebar collapsible="icon" {...props} className={className}>
      <SidebarHeader className="h-(--app-header-height) shrink-0 justify-center p-2">
        <div className="flex items-center gap-1 group-data-[collapsible=icon]:justify-center">
          <Button
            asChild
            type="button"
            variant="ghost"
            size="icon"
            className="text-muted-foreground hover:bg-sidebar-accent hover:text-foreground group-data-[collapsible=icon]:hidden"
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
              className="h-7 w-auto object-contain"
            />
          </Link>
          <SidebarTrigger className="ml-auto md:hidden" />
        </div>
      </SidebarHeader>
      <SidebarContent>
        <ChatNav
          chats={chats}
          selectedChatId={selectedChatId}
          isNewChat={isNewChat}
          onSelectChat={onSelectChat}
          onStartNewChat={onStartNewChat}
        />
      </SidebarContent>
      <SidebarFooter>
        <NavUser />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
