export type ChatMessage = {
  id: string
  from: "user" | "assistant"
  value: string
  status?: "error"
}

export type ChatListItem = {
  id: string
  title: string
  updatedAt: string
  preview?: string
}

export type MessagesByChatId = Record<string, ChatMessage[]>
