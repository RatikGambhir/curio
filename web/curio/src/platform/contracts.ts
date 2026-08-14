import type {
  ChatStreamDoneEvent,
  ChatStreamErrorEvent,
  ChatStreamEvent,
  ChatStreamRequest,
} from "@/features/chat/chat-stream"

export type AppTarget = "web" | "desktop"

export type ChatStreamOptions = {
  signal: AbortSignal
  onEvent: (event: ChatStreamEvent) => void
}

export interface ChatTransport {
  stream(
    request: ChatStreamRequest,
    options: ChatStreamOptions,
  ): Promise<ChatStreamDoneEvent | ChatStreamErrorEvent>
}

export interface PlatformServices {
  target: AppTarget
  chat: ChatTransport
  openExternalUrl(url: string): Promise<void>
}

export function parseExternalHttpUrl(value: string): URL {
  const url = new URL(value)
  if (url.protocol !== "https:" && url.protocol !== "http:") {
    throw new Error("Curio can only open HTTP or HTTPS links.")
  }
  return url
}
