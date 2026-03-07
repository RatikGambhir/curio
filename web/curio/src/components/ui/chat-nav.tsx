import {
  SidebarGroupContent,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "./sidebar"

export type ChatListItem = {
  id: string
  title: string
  updatedAt: string
  preview?: string
}

type ChatNavProps = {
  chats: ChatListItem[]
  selectedChatId: string | null
  onSelectChat: (chatId: string) => void
}

const ChatNav = ({ chats, selectedChatId, onSelectChat }: ChatNavProps) => {
  return (
    <SidebarGroup>
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
