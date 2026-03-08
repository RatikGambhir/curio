import { AnimatePresence, motion } from "framer-motion"
import {useCallback, useState} from "react"
import { ChatEmptyState } from "@/components/chat-empty-state"
import { ChatPrompt } from "@/components/chat-prompt"
import { ChatSidebar } from "@/components/chat-sidebar"
import { ChatView } from "@/components/chat-view"
import { mockMessagesByChatId, type ChatMessage } from "@/mocks/chats"
import type { ChatListItem } from "@/components/ui/chat-nav"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import {streamChat} from "@/lib/api/chat-api.ts";

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

function createUserMessage(value: string): ChatMessage {
  return {
    id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    from: "user",
    value,
  }
}

function createLLMMessage(value: string): ChatMessage {
  return {
    id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    from: "assistant",
    value,
  }
}

const Chat = () => {
  const [chats, setChats] = useState<ChatListItem[]>(initialChats)
  const [messagesByChatId, setMessagesByChatId] = useState(mockMessagesByChatId)
  const [selectedChatId, setSelectedChatId] = useState<string | null>(null)
  const isNewChat = selectedChatId === null
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
    const nextChatId = `chat-${Date.now()}`
    const firstMessage = createUserMessage(text)

    setMessagesByChatId((currentMessages) => ({
      ...currentMessages,
      [nextChatId]: [firstMessage],
    }))
    upsertChatMeta(nextChatId, text)
    setSelectedChatId(nextChatId)
    await streamChat(text, askQuestion)
  }

  const handleSendMessage = async (text: string) => {
    if (!selectedChatId) {
      console.log("HELLO")
      return
    }


    const nextMessage = createUserMessage(text)
    setMessagesByChatId((currentMessages) => ({
      ...currentMessages,
      [selectedChatId]: [...(currentMessages[selectedChatId] ?? []), nextMessage],
    }))
    upsertChatMeta(selectedChatId, text)
  }

  const askQuestion = useCallback((chunk: string, done: boolean) => {
    let text = ""
    text += chunk
    console.log("text - > ", text)
    const object = {
      "chat": [
        createLLMMessage(text)
      ]
    }

    if (done) {
    setMessagesByChatId(object)
    }
  }, [])

  return (
    <SidebarProvider className="h-screen w-full font-sans">
      <ChatSidebar
        chats={chats}
        selectedChatId={selectedChatId}
        isNewChat={isNewChat}
        onSelectChat={setSelectedChatId}
        onStartNewChat={handleStartNewChat}
        className="bg-background"
      />
      <SidebarInset className="bg-background">
        <div className="relative flex h-full w-full flex-col overflow-hidden bg-background px-4 py-6 md:px-8">
          <div
            aria-hidden="true"
            className="pointer-events-none absolute inset-0 opacity-[0.03]"
            style={{ backgroundRepeat: "repeat" }}
          />
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
                layout
                className="relative z-10 mx-auto flex h-full w-full max-w-6xl flex-col gap-4 px-2 md:px-3"
                initial={{ opacity: 0, y: 24, scale: 0.99 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: -16, scale: 0.995 }}
                transition={{ duration: 0.34, ease: [0.22, 1, 0.36, 1] }}
              >
                <motion.div
                  layout
                  className="min-h-0 flex-1 overflow-hidden rounded-[1.75rem] bg-background"
                >
                  <ChatView messages={messages} />
                </motion.div>
                <motion.div layout>
                  <ChatPrompt onSubmit={handleSendMessage} />
                </motion.div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}

export default Chat
