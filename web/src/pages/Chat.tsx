import { useState } from "react"
import { History, SquarePen } from "lucide-react"

import {
  ContextPane,
  ContextPaneToggle,
} from "@/components/app-shell/context-pane"
import { ChatComposer } from "@/components/chat-composer"
import { ChatEmptyState } from "@/components/chat-empty-state"
import { ChatHistory } from "@/components/chat-history"
import { ChatView } from "@/components/chat-view"
import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { demoChats, demoMessagesByChatId } from "@/features/chat/demo-data"
import type { ChatListItem } from "@/features/chat/types"
import { useChat } from "@/hooks/useChat"
import { useContextPane } from "@/hooks/useContextPane"

function buildChatTitle(text: string) {
  const normalized = text.trim().replace(/\s+/g, " ")
  if (!normalized) {
    return "New chat"
  }

  return normalized.length > 32 ? `${normalized.slice(0, 32).trimEnd()}...` : normalized
}

function buildChatPreview(text: string) {
  const normalized = text.trim().replace(/\s+/g, " ")
  return normalized.length > 52 ? `${normalized.slice(0, 52).trimEnd()}...` : normalized
}

const Chat = () => {
  const [chats, setChats] = useState<ChatListItem[]>(demoChats)
  const [messagesByChatId, setMessagesByChatId] = useState(demoMessagesByChatId)
  const [selectedChatId, setSelectedChatId] = useState<string | null>(null)
  const isNewChat = selectedChatId === null
  const { cancelStream, isStreaming, sendMessage } = useChat()
  const pane = useContextPane()
  const messages = selectedChatId ? messagesByChatId[selectedChatId] ?? [] : []
  const selectedChat = chats.find((chat) => chat.id === selectedChatId)

  const handleStartNewChat = () => {
    setSelectedChatId(null)
    pane.dismissOverlay()
  }

  const handleSelectChat = (chatId: string) => {
    setSelectedChatId(chatId)
    pane.dismissOverlay()
  }

  const upsertChatMeta = (chatId: string, text: string) => {
    setChats((currentChats) => {
      const existingChat = currentChats.find((chat) => chat.id === chatId)
      const nextMeta: ChatListItem = {
        id: chatId,
        title: existingChat?.title ?? buildChatTitle(text),
        updatedAt: "Just now",
        preview: buildChatPreview(text),
      }

      const otherChats = currentChats.filter((chat) => chat.id !== chatId)
      return [nextMeta, ...otherChats]
    })
  }

  const handleCreateChat = async (text: string) => {
    if (isStreaming) {
      return
    }

    const threadId = crypto.randomUUID()
    const userMessageId = crypto.randomUUID()
    const assistantMessageId = crypto.randomUUID()

    //TODO: persist to local storage
    upsertChatMeta(threadId, text)
    setSelectedChatId(threadId)
    await sendMessage({
      chatId: threadId,
      text,
      setMessagesByChatId,
      userMessageId,
      assistantMessageId,
    })
  }

  const handleSendMessage = async (text: string) => {
    if (!selectedChatId || isStreaming) {
      return
    }

    //TODO: persist to local storage

    await sendMessage({ chatId: selectedChatId, text, setMessagesByChatId })
    upsertChatMeta(selectedChatId, text)
  }

  const newChatButton = (
    <Button
      type="button"
      variant="ghost"
      size="sm"
      disabled={isNewChat}
      onClick={handleStartNewChat}
      className="-mr-1"
    >
      <SquarePen aria-hidden="true" />
      New
    </Button>
  )

  return (
    <div className="flex min-h-0 flex-1">
      <ContextPane
        pane={pane}
        title="Conversations"
        description="Your recent conversations with Curio."
        action={newChatButton}
      >
        <ChatHistory
          chats={chats}
          selectedChatId={selectedChatId}
          onSelectChat={handleSelectChat}
        />
      </ContextPane>

      <div className="flex min-w-0 flex-1 flex-col">
        <PageHeader
          title={selectedChat?.title ?? "Chat"}
          meta={isStreaming ? "Writing…" : undefined}
          leading={<ContextPaneToggle pane={pane} label="conversations" overlayIcon={History} />}
          actions={
            pane.open && pane.isInline ? null : (
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                disabled={isNewChat}
                onClick={handleStartNewChat}
                aria-label="New chat"
                title="New chat"
              >
                <SquarePen aria-hidden="true" />
              </Button>
            )
          }
        />

        {isNewChat ? (
          <ChatEmptyState disabled={isStreaming} onSubmit={handleCreateChat} />
        ) : (
          <div
            key={selectedChatId}
            className="rise-in flex min-h-0 flex-1 flex-col"
          >
            <div className="min-h-0 flex-1 overflow-hidden">
              <ChatView messages={messages} />
            </div>
            <div className="mx-auto w-full max-w-[44rem] px-5 pb-5 sm:px-8 sm:pb-6">
              <ChatComposer
                disabled={isStreaming}
                isStreaming={isStreaming}
                placeholder="Ask a follow-up"
                onStop={() => cancelStream(selectedChatId ?? undefined)}
                onSubmit={handleSendMessage}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default Chat
