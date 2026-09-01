import { MessageSquare } from "lucide-react"

import { ChatMessageItem } from "@/components/chat-message"
import {
  Thread,
  ThreadContent,
  ThreadScrollToBottom,
} from "@/components/nexus-ui/thread"
import type { ChatMessage } from "@/features/chat/types"

type ChatViewProps = {
  messages: ChatMessage[]
}

function ChatViewEmpty() {
  return (
    <div className="flex h-full items-center justify-center p-8 text-center">
      <div className="space-y-3">
        <div className="flex justify-center text-muted-foreground">
          <MessageSquare className="size-12" />
        </div>
        <div className="space-y-1">
          <h3 className="font-medium text-sm">Start a conversation</h3>
          <p className="text-sm text-muted-foreground">
            Type a message below to begin chatting
          </p>
        </div>
      </div>
    </div>
  )
}

export function ChatView({ messages }: ChatViewProps) {
  if (messages.length === 0) {
    return <ChatViewEmpty />
  }

  return (
    <Thread className="h-full">
      <ThreadContent className="px-2 py-6 md:px-4">
        {messages.map((message) => (
          <ChatMessageItem key={message.id} {...message} />
        ))}
      </ThreadContent>
      <ThreadScrollToBottom />
    </Thread>
  )
}
