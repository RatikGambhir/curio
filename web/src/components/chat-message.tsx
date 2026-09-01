import { AlertTriangle } from "lucide-react"
import { useEffect, useState } from "react"

import {
  Message,
  MessageContent,
  MessageMarkdown,
  MessageStack,
} from "@/components/nexus-ui/message"
import { TextShimmer } from "@/components/nexus-ui/text-shimmer"
import { funnyThinkingTerms } from "@/features/chat/demo-data"
import type { ChatMessage } from "@/features/chat/types"

const thinkingTerms = Array.from(new Set(Object.values(funnyThinkingTerms).flat()))

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
    <TextShimmer
      className="text-sm font-normal text-muted-foreground"
      duration={1.6}
      aria-live="polite"
    >
      {thinkingTerms[termIndex]}
    </TextShimmer>
  )
}

function AssistantErrorMessage({ value }: Pick<ChatMessage, "value">) {
  return (
    <div
      className="flex items-start gap-2 rounded-2xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive"
      role="alert"
    >
      <AlertTriangle
        className="mt-0.5 size-4 shrink-0 text-destructive"
        aria-hidden="true"
      />
      <div>
        <p className="font-semibold">Response unavailable</p>
        <p className="mt-1 text-destructive/90">{value}</p>
      </div>
    </div>
  )
}

export function UserMessage({ value }: Pick<ChatMessage, "value">) {
  return (
    <Message from="user">
      <MessageStack>
        <MessageContent>{value}</MessageContent>
      </MessageStack>
    </Message>
  )
}

export function AssistantMessage({
  value,
  status,
}: Pick<ChatMessage, "value" | "status">) {
  const hasContent = value.trim().length > 0

  return (
    <Message from="assistant">
      <MessageStack>
        <MessageContent className={status === "error" ? "px-0" : undefined}>
          {status === "error" ? (
            <AssistantErrorMessage value={value} />
          ) : hasContent ? (
            <MessageMarkdown parseIncompleteMarkdown>{value}</MessageMarkdown>
          ) : (
            <AssistantThinkingIndicator />
          )}
        </MessageContent>
      </MessageStack>
    </Message>
  )
}

export function ChatMessageItem(message: ChatMessage) {
  if (message.from === "user") {
    return <UserMessage value={message.value} />
  }

  return <AssistantMessage value={message.value} status={message.status} />
}
