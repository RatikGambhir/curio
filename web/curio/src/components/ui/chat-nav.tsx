import {
  SidebarGroupContent,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "./sidebar"
import { Button } from "./button"
import { PencilLine } from "lucide-react"

export type ChatListItem = {
  id: string
  title: string
  updatedAt: string
  preview?: string
}

type ChatNavProps = {
  chats: ChatListItem[]
  selectedChatId: string | null
  isNewChat: boolean
  onSelectChat: (chatId: string) => void
  onStartNewChat: () => void
}

const ChatNav = ({
  chats,
  selectedChatId,
  isNewChat,
  onSelectChat,
  onStartNewChat,
}: ChatNavProps) => {
  return (
    <SidebarGroup>
      <Button
        type="button"
        variant="ghost"
        disabled={isNewChat}
        onClick={onStartNewChat}
        className="mb-3 h-11 w-full justify-start rounded-lg border border-sidebar-border bg-sidebar px-3 text-sm font-medium text-sidebar-foreground shadow-none hover:bg-sidebar-accent hover:text-sidebar-accent-foreground disabled:translate-y-0 disabled:cursor-default disabled:bg-sidebar-accent disabled:text-sidebar-accent-foreground disabled:opacity-100 group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0"
      >
        <PencilLine className="size-4" />
        <span className="group-data-[collapsible=icon]:hidden">New chat</span>
      </Button>
      <SidebarGroupLabel>Chats</SidebarGroupLabel>
      <SidebarGroupContent>
        <SidebarMenu className="gap-2">
          {chats.map((chat) => (
            <SidebarMenuItem key={chat.id}>
              <SidebarMenuButton
                type="button"
                tooltip={chat.title}
                isActive={selectedChatId === chat.id}
                onClick={() => onSelectChat(chat.id)}
                className="h-auto items-start rounded-lg border border-sidebar-border bg-sidebar px-3 py-2 text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground data-[active=true]:border-sidebar-ring data-[active=true]:bg-sidebar-accent data-[active=true]:text-sidebar-accent-foreground group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-2"
              >
                <span className="hidden size-6 items-center justify-center rounded-md border border-sidebar-border text-xs font-semibold group-data-[collapsible=icon]:inline-flex">
                  {chat.title.charAt(0).toUpperCase()}
                </span>
                <div className="flex w-full min-w-0 flex-col gap-1 group-data-[collapsible=icon]:hidden">
                  <div className="flex items-center justify-between gap-2">
                    <span className="truncate font-medium">{chat.title}</span>
                    <span className="shrink-0 text-[10px] text-sidebar-foreground/70">
                      {chat.updatedAt}
                    </span>
                  </div>
                  {chat.preview ? (
                    <span className="truncate text-xs text-sidebar-foreground/70">
                      {chat.preview}
                    </span>
                  ) : null}
                </div>
              </SidebarMenuButton>
            </SidebarMenuItem>
          ))}
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>
  )
}

export default ChatNav
