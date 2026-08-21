import { platformServices } from "@curio/platform-runtime"

import { serviceErrorMessage } from "@/api/errors"
import {
  ChatStreamRequestError,
  ChatStreamSession,
  type ChatStreamDoneEvent,
  type ChatStreamErrorEvent,
  type ChatStreamEvent,
  type ChatStreamRequest,
} from "@/features/chat/chat-stream"
import { concatBytes, type ServiceTransport } from "@/platform/contracts"

export const CHAT_STREAM_PATH = "/v1/chat/stream"

export type ChatStreamOptions = {
  signal: AbortSignal
  onEvent: (event: ChatStreamEvent) => void
}

export type ChatStreamFn = (
  request: ChatStreamRequest,
  options: ChatStreamOptions,
) => Promise<ChatStreamDoneEvent | ChatStreamErrorEvent>

/**
 * Streams a chat completion through the platform transport, decoding the
 * shared SSE protocol regardless of whether bytes arrive over the browser's
 * HTTP stack (web) or the Tauri bridge (desktop).
 */
export async function streamChat(
  request: ChatStreamRequest,
  { signal, onEvent }: ChatStreamOptions,
  transport: ServiceTransport = platformServices.service,
): Promise<ChatStreamDoneEvent | ChatStreamErrorEvent> {
  const session = new ChatStreamSession(
    {
      conversationId: request.conversationId,
      messageId: request.assistantMessageId,
    },
    onEvent,
  )
  let status: number | null = null
  const errorChunks: Uint8Array[] = []

  await transport.stream(
    { method: "POST", path: CHAT_STREAM_PATH, body: request, signal },
    {
      onStatus: (value) => {
        status = value
      },
      onChunk: (bytes) => {
        if (status !== null && status >= 200 && status < 300) {
          session.push(bytes)
        } else {
          errorChunks.push(bytes)
        }
      },
    },
  )

  if (status === null) {
    throw new ChatStreamRequestError(
      "missing_status",
      "The platform transport did not report the chat response status.",
    )
  }
  if (status < 200 || status >= 300) {
    const responseText = new TextDecoder().decode(concatBytes(errorChunks))
    throw new ChatStreamRequestError(
      "http_error",
      serviceErrorMessage(responseText) ??
        `The Curio service returned HTTP ${status}.`,
    )
  }
  return session.finish()
}
