import { useCallback, useState } from "react"
import type { Dispatch, SetStateAction } from "react"

import type { ChatMessage } from "@/mocks/chats"

type MessagesByChatId = Record<string, ChatMessage[]>

type StreamParams = {
  chatId: string
  text: string
  setMessagesByChatId: Dispatch<SetStateAction<MessagesByChatId>>
  userId?: string | null
  userMessageId?: string
  threadId?: string
  assistantMessageId?: string
}

type StreamChunk = {
  token: string
}

type WorkerRequestBody = {
  userId: string
  prompt: string
  attachments: null
  threadId?: string
}

const workerURLString = import.meta.env.CHAT_WORKER_URL
const workerURL = new URL(workerURLString ?? "https://api.gettingcurio.com/chat")

function createMessage(
  from: ChatMessage["from"],
  value: string,
  id: string = crypto.randomUUID(),
): ChatMessage {
  return {
    id,
    from,
    value,
  }
}

function appendTokenToAssistantMessage(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  token: string,
) {
  setMessagesByChatId((currentMessages) => {
    const chatMessages = currentMessages[chatId] ?? []
    const nextChatMessages = chatMessages.map((message) => {
      if (message.id !== assistantMessageId) {
        return message
      }

      return {
        ...message,
        value: message.value + token,
      }
    })

    return {
      ...currentMessages,
      [chatId]: nextChatMessages,
    }
  })
}

function setAssistantMessageValue(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  value: string,
) {
  setMessagesByChatId((currentMessages) => {
    const chatMessages = currentMessages[chatId] ?? []
    const nextChatMessages = chatMessages.map((message) => {
      if (message.id !== assistantMessageId) {
        return message
      }

      return {
        ...message,
        value,
      }
    })

    return {
      ...currentMessages,
      [chatId]: nextChatMessages,
    }
  })
}

function parseStreamEvent(eventBlock: string): { token?: string; error?: string } {
  const normalized = eventBlock.trim()
  if (!normalized) {
    return {}
  }

  if (normalized.startsWith("data:")) {
    const payload = normalized.slice(5).trim()
    const parsed = JSON.parse(payload) as StreamChunk
    return { token: parsed.token ?? "" }
  }

  if (normalized.startsWith("ERROR:")) {
    const payload = normalized.slice(6).trim()
    const parsed = JSON.parse(payload) as { error?: unknown }
    if (typeof parsed.error === "string") {
      return { error: parsed.error }
    }

    return { error: "Unknown stream error" }
  }

  return {}
}

export function useChat() {
  const [isStreaming, setIsStreaming] = useState(false)
  const [streamError, setStreamError] = useState<string | null>(null)

  const sendMessage = useCallback(
    async ({
      chatId,
      text,
      setMessagesByChatId,
      userId,
      userMessageId,
      threadId,
      assistantMessageId,
    }: StreamParams) => {
      setStreamError(null)
      setIsStreaming(true)

      const userMessage = createMessage("user", text, userMessageId)
      const assistantMessage = createMessage("assistant", "", assistantMessageId)

      setMessagesByChatId((currentMessages) => ({
        ...currentMessages,
        [chatId]: [...(currentMessages[chatId] ?? []), userMessage, assistantMessage],
      }))

      try {
        const body: WorkerRequestBody = {
          userId: userId ?? "266ee938-12db-47d1-9ffd-6d53d0b25808",
          prompt: text,
          attachments: null,
          threadId,
        }

        const response = await fetch(workerURL, {
          body: JSON.stringify(body),
          method: "POST",
          headers: { "Content-Type": "application/json" },
        })

        if (!response.ok) {
          const responseText = await response.text()
          console.log(`HTTP ${response.status}: ${responseText}`)

        }

        if (!response.body) {
          return
        }

        const reader = response.body.getReader()
        const decoder = new TextDecoder()
        let streamBuffer = ""

        while (true) {
          const { value, done } = await reader.read()
          if (done) {
            break
          }

          streamBuffer += decoder.decode(value, { stream: true })
          const events = streamBuffer.split("\n\n")
          streamBuffer = events.pop() ?? ""

          for (const eventBlock of events) {
            const parsedEvent = parseStreamEvent(eventBlock)
            if (parsedEvent.error) {
             console.log("ERROR")
            }
            if (parsedEvent.token) {
              appendTokenToAssistantMessage(
                setMessagesByChatId,
                chatId,
                assistantMessage.id,
                parsedEvent.token,
              )
            }
          }
        }

        const tailEvent = parseStreamEvent(streamBuffer)
        if (tailEvent.error) {
          console.log("ERROR")

        }
        if (tailEvent.token) {
          appendTokenToAssistantMessage(
            setMessagesByChatId,
            chatId,
            assistantMessage.id,
            tailEvent.token,
          )
        }
      } catch (error) {
        const nextError = error instanceof Error ? error.message : "Unknown stream failure"
        setStreamError(nextError)
        setAssistantMessageValue(
          setMessagesByChatId,
          chatId,
          assistantMessage.id,
          "Something went wrong while streaming the response.",
        )
      } finally {
        setIsStreaming(false)
      }
    },
    [],
  )

  return {
    isStreaming,
    sendMessage,
    streamError,
  }
}
