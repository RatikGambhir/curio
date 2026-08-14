import { AnimatePresence, motion } from "framer-motion"
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
import type { ChatListItem } from "@/features/chat/types"

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
        className="mb-2 h-9 w-full justify-start rounded-md border border-transparent bg-transparent px-2 text-sm font-normal text-sidebar-foreground shadow-none hover:translate-y-0 hover:bg-sidebar-foreground/[0.035] hover:text-sidebar-foreground disabled:translate-y-0 disabled:cursor-default disabled:border-sidebar-foreground/35 disabled:bg-transparent disabled:text-sidebar-foreground disabled:opacity-100 group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-0"
      >
        <PencilLine className="size-4" />
        <span className="group-data-[collapsible=icon]:hidden">New chat</span>
      </Button>
      <SidebarGroupLabel>Chats</SidebarGroupLabel>
      <SidebarGroupContent>
        <SidebarMenu className="gap-0.5">
          <AnimatePresence initial={false}>
            {chats.map((chat) => (
              <motion.div
                key={chat.id}
                layout
                initial={{ opacity: 0, x: -14, scale: 0.98 }}
                animate={{ opacity: 1, x: 0, scale: 1 }}
                exit={{ opacity: 0, x: -10, scale: 0.98 }}
                transition={{ duration: 0.22, ease: [0.22, 1, 0.36, 1] }}
              >
                <SidebarMenuItem>
                  <SidebarMenuButton
                    type="button"
                    tooltip={chat.title}
                    isActive={selectedChatId === chat.id}
                    onClick={() => onSelectChat(chat.id)}
                    className="h-auto items-start rounded-md border border-transparent bg-transparent px-2 py-2 text-sidebar-foreground shadow-none hover:translate-y-0 hover:bg-sidebar-foreground/[0.035] hover:text-sidebar-foreground data-[active=true]:border-sidebar-foreground/40 data-[active=true]:!bg-transparent data-[active=true]:!text-sidebar-foreground data-[active=true]:ring-1 data-[active=true]:ring-sidebar-foreground/10 group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:px-2"
                  >
                    <span className="hidden size-6 items-center justify-center rounded-md border border-sidebar-border text-xs font-semibold group-data-[collapsible=icon]:inline-flex">
                      {chat.title.charAt(0).toUpperCase()}
                    </span>
                    <div className="flex w-full min-w-0 flex-col gap-1 group-data-[collapsible=icon]:hidden">
                      <div className="flex items-center justify-between gap-2">
                        <span className="truncate font-normal">{chat.title}</span>
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
              </motion.div>
            ))}
          </AnimatePresence>
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>
  )
}

export default ChatNav
