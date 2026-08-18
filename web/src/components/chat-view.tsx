import { AnimatePresence } from "framer-motion"
import { MessageSquare } from "lucide-react"

import { ChatMessageItem } from "@/components/chat-message"
import type { ChatMessage } from "@/features/chat/types"

type ChatViewProps = {
  messages: ChatMessage[]
}

export function ChatView({ messages }: ChatViewProps) {
  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden bg-background">
      <div className="min-h-0 flex-1 overflow-y-auto">
        {messages.length > 0 ? (
          <AnimatePresence initial={false}>
            <div className="flex w-full flex-col gap-6 px-2 py-6 md:px-4">
              {messages.map((message) => (
                <ChatMessageItem key={message.id} {...message} />
              ))}
            </div>
          </AnimatePresence>
        ) : (
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
        )}
      </div>
    </div>
  )
}
