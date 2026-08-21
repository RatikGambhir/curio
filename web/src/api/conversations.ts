import { api, type ApiRequestOptions } from "@/api/client"

export type ConversationRecord = {
  id: string
  createdAt: string
  updatedAt: string
}

export type ConversationMessageRecord = {
  id: string
  conversationId: string
  role: "user" | "assistant"
  content: string
  status: string
  responseId: string | null
  errorCode: string | null
  createdAt: string
  updatedAt: string
}

export function listConversations(
  options?: ApiRequestOptions,
): Promise<{ conversations: ConversationRecord[] }> {
  return api.get("/v1/conversations", undefined, options)
}

export function conversationMessages(
  conversationId: string,
  options?: ApiRequestOptions,
): Promise<{
  conversationId: string
  messages: ConversationMessageRecord[]
}> {
  return api.get(
    `/v1/conversations/${encodeURIComponent(conversationId)}/messages`,
    undefined,
    options,
  )
}
