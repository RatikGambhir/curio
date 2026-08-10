import { motion } from "framer-motion"
import { AlertTriangle } from "lucide-react"
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
        "max-w-[80%] rounded-[1.15rem] px-4 py-2.5 text-sm font-normal leading-6 shadow-none",
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
      <MessageBubble className="border border-foreground/[0.06] bg-muted/80 text-foreground">
        {value}
      </MessageBubble>
    </MessageRow>
  )
}

function AssistantErrorMessage({ value }: Pick<ChatMessage, "value">) {
  return (
    <div className="flex items-start gap-2 text-red-900" role="alert">
      <AlertTriangle className="mt-0.5 size-4 shrink-0 text-red-600" aria-hidden="true" />
      <div>
        <p className="font-semibold">Response unavailable</p>
        <p className="mt-1 text-red-800/90">{value}</p>
      </div>
    </div>
  )
}

export function AssistantMessage({ value, status }: Pick<ChatMessage, "value" | "status">) {
  const isError = status === "error"

  return (
    <MessageRow className="justify-start">
      <MessageBubble
        className={
          isError
            ? "border border-red-300 bg-red-50 text-red-900 shadow-sm"
            : "max-w-[92%] rounded-none bg-transparent px-1 py-2 text-foreground"
        }
      >
        {isError ? (
          <AssistantErrorMessage value={value} />
        ) : value.trim() ? (
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

  return <AssistantMessage value={message.value} status={message.status} />
}
