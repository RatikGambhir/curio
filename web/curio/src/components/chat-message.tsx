import { motion } from "framer-motion"
import { useEffect, useState, type ReactNode } from "react"
import ReactMarkdown from "react-markdown"

import { cn } from "@/lib/utils"
import { funnyThinkingTerms, type ChatMessage } from "@/mocks/chats"

type BaseMessageProps = {
  children: ReactNode
  className?: string
}

const thinkingTerms = Array.from(new Set(Object.values(funnyThinkingTerms).flat()))

function MessageRow({ children, className }: BaseMessageProps) {
  return (
    <motion.div
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

function AssistantThinkingIndicator() {
  const [termIndex, setTermIndex] = useState(0)

  useEffect(() => {
    const intervalId = window.setInterval(() => {
      setTermIndex((currentIndex) => (currentIndex + 1) % thinkingTerms.length)
    }, 1800)

    return () => {
      window.clearInterval(intervalId)
    }
  }, [])

  return (
    <div className="flex items-center gap-2 text-muted-foreground" aria-live="polite">
      <span className="flex items-center gap-1" aria-hidden="true">
        <span className="size-1.5 animate-bounce rounded-full bg-current [animation-delay:-0.2s]" />
        <span className="size-1.5 animate-bounce rounded-full bg-current [animation-delay:-0.1s]" />
        <span className="size-1.5 animate-bounce rounded-full bg-current" />
      </span>
      <span>{thinkingTerms[termIndex]}</span>
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
        {value.trim() ? (
          <ReactMarkdown
            components={{
              p: ({ children }) => <p className="mb-3 last:mb-0">{children}</p>,
              h1: ({ children }) => (
                <h1 className="mb-3 text-lg font-semibold leading-7">{children}</h1>
              ),
              h2: ({ children }) => (
                <h2 className="mb-3 text-base font-semibold leading-7">{children}</h2>
              ),
              h3: ({ children }) => (
                <h3 className="mb-2 text-sm font-semibold leading-6">{children}</h3>
              ),
              ul: ({ children }) => (
                <ul className="mb-3 list-disc space-y-1 pl-5 last:mb-0">{children}</ul>
              ),
              ol: ({ children }) => (
                <ol className="mb-3 list-decimal space-y-1 pl-5 last:mb-0">{children}</ol>
              ),
              li: ({ children }) => <li className="pl-1">{children}</li>,
              strong: ({ children }) => <strong className="font-semibold">{children}</strong>,
              code: ({ children }) => (
                <code className="rounded-md bg-background/70 px-1.5 py-0.5 font-mono text-[0.85em]">
                  {children}
                </code>
              ),
              pre: ({ children }) => (
                <pre className="mb-3 overflow-x-auto rounded-xl bg-background/70 p-3 text-xs leading-5 last:mb-0">
                  {children}
                </pre>
              ),
            }}
          >
            {value}
          </ReactMarkdown>
        ) : (
          <AssistantThinkingIndicator />
        )}
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
