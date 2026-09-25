import { ContextPaneItem } from "@/components/app-shell/context-pane"
import type { ChatListItem } from "@/features/chat/types"

type ChatHistoryProps = {
  chats: ChatListItem[]
  selectedChatId: string | null
  onSelectChat: (chatId: string) => void
}

export function ChatHistory({
  chats,
  selectedChatId,
  onSelectChat,
}: ChatHistoryProps) {
  if (chats.length === 0) {
    return (
      <p className="px-4 py-6 text-sm text-muted-foreground">
        Conversations you start will be listed here.
      </p>
    )
  }

  return (
    <ul className="flex flex-col gap-0.5 p-2">
      {chats.map((chat) => (
        <li key={chat.id}>
          <ContextPaneItem
            isActive={selectedChatId === chat.id}
            onSelect={() => onSelectChat(chat.id)}
          >
            <span className="flex w-full items-baseline justify-between gap-3">
              <span className="truncate text-sm font-medium text-foreground">
                {chat.title}
              </span>
              <span className="shrink-0 font-mono text-2xs tabular-nums text-muted-foreground">
                {chat.updatedAt}
              </span>
            </span>
            {chat.preview ? (
              <span className="line-clamp-1 text-[0.8125rem] text-muted-foreground">
                {chat.preview}
              </span>
            ) : null}
          </ContextPaneItem>
        </li>
      ))}
    </ul>
  )
}
