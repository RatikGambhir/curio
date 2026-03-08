import { motion } from "framer-motion"

import { cn } from "@/lib/utils"
import type { ChatMessage } from "@/mocks/chats"

type BaseMessageProps = {
  children: React.ReactNode
  className?: string
}

function MessageRow({ children, className }: BaseMessageProps) {
  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 18 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.24, ease: [0.22, 1, 0.36, 1] }}
      className={cn("flex w-full", className)}
    >
      {children}
    </motion.div>
  )
}

function MessageBubble({ children, className }: BaseMessageProps) {
  return (
    <div
      className={cn(
        "max-w-[80%] rounded-2xl px-4 py-3 text-sm leading-6 shadow-sm",
        className,
      )}
    >
      {children}
    </div>
  )
}

export function UserMessage({ value }: Pick<ChatMessage, "value">) {
  return (
    <MessageRow className="justify-end">
      <MessageBubble className="bg-secondary text-foreground">
        {value}
      </MessageBubble>
    </MessageRow>
  )
}

export function AssistantMessage({ value }: Pick<ChatMessage, "value">) {
  return (
    <MessageRow className="justify-start">
      <MessageBubble className="bg-primary/10 text-foreground">
        {value}
      </MessageBubble>
    </MessageRow>
  )
}

export function ChatMessageItem(message: ChatMessage) {
  if (message.from === "user") {
    return <UserMessage value={message.value} />
  }

  return <AssistantMessage value={message.value} />
}
