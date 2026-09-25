import { MessageSquare } from "lucide-react"

import { ChatMessageItem } from "@/components/chat-message"
import {
  Thread,
  ThreadContent,
  ThreadScrollToBottom,
} from "@/components/nexus-ui/thread"
import { EmptyState } from "@/components/ui/empty-state"
import type { ChatMessage } from "@/features/chat/types"

type ChatViewProps = {
  messages: ChatMessage[]
}

export function ChatView({ messages }: ChatViewProps) {
  if (messages.length === 0) {
    return (
      <EmptyState
        icon={MessageSquare}
        title="Nothing said yet"
        className="h-full"
      >
        Write below to begin this conversation.
      </EmptyState>
    )
  }

  return (
    <Thread className="h-full">
      <ThreadContent className="mx-auto max-w-[44rem] gap-8 px-5 py-8 sm:px-8">
        {messages.map((message) => (
          <ChatMessageItem key={message.id} {...message} />
        ))}
      </ThreadContent>
      <ThreadScrollToBottom />
    </Thread>
  )
}
