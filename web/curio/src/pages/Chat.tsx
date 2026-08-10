import { AnimatePresence, motion } from "framer-motion"
import { useState } from "react"
import { ChatEmptyState } from "@/components/chat-empty-state"
import { ChatPrompt } from "@/components/chat-prompt"
import { ChatSidebar } from "@/components/chat-sidebar"
import { PageHeader } from "@/components/page-header"
import { ChatView } from "@/components/chat-view"
import { mockMessagesByChatId } from "@/mocks/chats"
import type { ChatListItem } from "@/components/ui/chat-nav"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { useChat } from "@/hooks/useChat"

const initialChats: ChatListItem[] = [
  {
    id: "chat-1",
    title: "Product brainstorming",
    updatedAt: "2m ago",
    preview: "Can you draft three launch headlines?",
  },
  {
    id: "chat-2",
    title: "Workout plan",
    updatedAt: "1h ago",
    preview: "Build a 4-day split for strength and mobility.",
  },
  {
    id: "chat-3",
    title: "Travel itinerary",
    updatedAt: "Yesterday",
    preview: "Weekend plan for Austin with coffee stops.",
  },
  {
    id: "chat-4",
    title: "Code review notes",
    updatedAt: "Thu",
    preview: "Summarize regression risks from the latest PR.",
  },
]

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
  const [chats, setChats] = useState<ChatListItem[]>(initialChats)
  const [messagesByChatId, setMessagesByChatId] = useState(mockMessagesByChatId)
  const [selectedChatId, setSelectedChatId] = useState<string | null>(null)
  const isNewChat = selectedChatId === null
  const { sendMessage } = useChat()
  const messages = selectedChatId ? messagesByChatId[selectedChatId] ?? [] : []
  const handleStartNewChat = () => {
    setSelectedChatId(null)
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
    if (!selectedChatId) {
      return
    }

    //TODO: persist to local storage

    await sendMessage({ chatId: selectedChatId, text, setMessagesByChatId })
    upsertChatMeta(selectedChatId, text)
  }


  return (
    <SidebarProvider className="h-screen w-full font-sans">
      <ChatSidebar
        chats={chats}
        selectedChatId={selectedChatId}
        isNewChat={isNewChat}
        onSelectChat={setSelectedChatId}
        onStartNewChat={handleStartNewChat}
      />
      <SidebarInset className="bg-background">
        <div className="flex h-full w-full flex-col bg-background">
          <PageHeader />
          <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden bg-background px-4 py-5 md:px-8 md:py-6">
          <AnimatePresence mode="popLayout" initial={false}>
            {isNewChat ? (
              <motion.div
                key="new-chat"
                className="relative z-10 flex h-full w-full flex-col"
                initial={{ opacity: 0, y: 20, scale: 0.985 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: -16, scale: 0.99 }}
                transition={{ duration: 0.32, ease: [0.22, 1, 0.36, 1] }}
              >
                <ChatEmptyState onSubmit={handleCreateChat} />
              </motion.div>
            ) : (
              <motion.div
                key={selectedChatId ?? "thread"}
                className="relative z-10 mx-auto flex h-full w-full max-w-4xl flex-col gap-3"
                initial={{ opacity: 0, y: 24, scale: 0.99 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: -16, scale: 0.995 }}
                transition={{ duration: 0.34, ease: [0.22, 1, 0.36, 1] }}
              >
                <motion.div
                  className="min-h-0 flex-1 overflow-hidden"
                >
                  <ChatView messages={messages} />
                </motion.div>
                <motion.div>
                  <ChatPrompt onSubmit={handleSendMessage} />
                </motion.div>
              </motion.div>
            )}
          </AnimatePresence>
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}

export default Chat
