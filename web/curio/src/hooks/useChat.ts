import { useCallback, useEffect, useRef, useState } from "react"
import type { Dispatch, SetStateAction } from "react"

import { streamCurioChat } from "@/lib/chat-stream"
import type { ChatMessage } from "@/mocks/chats"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"

type MessagesByChatId = Record<string, ChatMessage[]>

type StreamParams = {
  chatId: string
  text: string
  setMessagesByChatId: Dispatch<SetStateAction<MessagesByChatId>>
  userMessageId?: string
  assistantMessageId?: string
}

const workerUrl =
  import.meta.env.VITE_CURIO_CHAT_WORKER_URL ?? "http://127.0.0.1:8787"

function createMessage(
  from: ChatMessage["from"],
  value: string,
  id: string = crypto.randomUUID(),
): ChatMessage {
  return { id, from, value }
}

function updateAssistantMessage(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  update: (message: ChatMessage) => ChatMessage,
) {
  setMessagesByChatId((currentMessages) => ({
    ...currentMessages,
    [chatId]: (currentMessages[chatId] ?? []).map((message) =>
      message.id === assistantMessageId ? update(message) : message,
    ),
  }))
}

function appendAssistantToken(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  token: string,
) {
  updateAssistantMessage(setMessagesByChatId, chatId, assistantMessageId, (message) => ({
    ...message,
    value: message.value + token,
  }))
}

function setAssistantError(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  message: string,
) {
  updateAssistantMessage(setMessagesByChatId, chatId, assistantMessageId, (current) => ({
    ...current,
    value: message,
    status: "error",
  }))
}

export function useChat() {
  const userId = useAuthenticatedUser().user?.id
  const [isStreaming, setIsStreaming] = useState(false)
  const [streamError, setStreamError] = useState<string | null>(null)
  const activeStreams = useRef(new Map<string, AbortController>())

  useEffect(
    () => () => {
      for (const controller of activeStreams.current.values()) {
        controller.abort()
      }
      activeStreams.current.clear()
    },
    [],
  )

  const cancelStream = useCallback((chatId?: string) => {
    if (chatId) {
      activeStreams.current.get(chatId)?.abort()
      return
    }

    for (const controller of activeStreams.current.values()) {
      controller.abort()
    }
  }, [])

  const sendMessage = useCallback(
    async ({
      chatId,
      text,
      setMessagesByChatId,
      userMessageId,
      assistantMessageId,
    }: StreamParams) => {
      if (!userId) {
        setStreamError("Sign in with an email before starting a chat.")
        return
      }

      activeStreams.current.get(chatId)?.abort()
      const controller = new AbortController()
      activeStreams.current.set(chatId, controller)
      setStreamError(null)
      setIsStreaming(true)

      const userMessage = createMessage("user", text, userMessageId)
      const assistantMessage = createMessage("assistant", "", assistantMessageId)

      setMessagesByChatId((currentMessages) => ({
        ...currentMessages,
        [chatId]: [...(currentMessages[chatId] ?? []), userMessage, assistantMessage],
      }))

      try {
        const terminal = await streamCurioChat(
          workerUrl,
          {
            userId,
            conversationId: chatId,
            userMessageId: userMessage.id,
            assistantMessageId: assistantMessage.id,
            prompt: text,
          },
          controller.signal,
          (event) => {
            if (event.type === "token") {
              appendAssistantToken(
                setMessagesByChatId,
                chatId,
                assistantMessage.id,
                event.token,
              )
            }
          },
        )

        if (terminal.type === "error") {
          setStreamError(terminal.message)
          setAssistantError(
            setMessagesByChatId,
            chatId,
            assistantMessage.id,
            terminal.message,
          )
        }
      } catch (error) {
        const message =
          error instanceof DOMException && error.name === "AbortError"
            ? "Response canceled."
            : error instanceof Error
              ? error.message
              : "Something went wrong while streaming the response."
        setStreamError(message)
        setAssistantError(setMessagesByChatId, chatId, assistantMessage.id, message)
      } finally {
        if (activeStreams.current.get(chatId) === controller) {
          activeStreams.current.delete(chatId)
          setIsStreaming(false)
        }
      }
    },
    [userId],
  )

  return { cancelStream, isStreaming, sendMessage, streamError }
}
