import { Channel, invoke } from "@tauri-apps/api/core"
import { openUrl } from "@tauri-apps/plugin-opener"

import {
  ChatStreamRequestError,
  ChatStreamSession,
  workerErrorMessage,
  type ChatStreamDoneEvent,
  type ChatStreamErrorEvent,
  type ChatStreamRequest,
} from "@/features/chat/chat-stream"
import {
  parseExternalHttpUrl,
  type ChatStreamOptions,
  type PlatformServices,
} from "@/platform/contracts"

type DesktopStreamPacket =
  | { type: "started"; status: number }
  | { type: "chunk"; bytes: number[] }
  | { type: "end" }
  | { type: "error"; code: string; message: string }

function abortError(): DOMException {
  return new DOMException("The chat response was canceled.", "AbortError")
}

async function streamDesktopChat(
  request: ChatStreamRequest,
  { signal, onEvent }: ChatStreamOptions,
): Promise<ChatStreamDoneEvent | ChatStreamErrorEvent> {
  if (signal.aborted) {
    throw abortError()
  }

  const requestId = crypto.randomUUID()
  const session = new ChatStreamSession(
    {
      conversationId: request.conversationId,
      messageId: request.assistantMessageId,
    },
    onEvent,
  )
  const errorChunks: Uint8Array[] = []
  const channel = new Channel<DesktopStreamPacket>()
  let status: number | null = null
  let ended = false
  let packetError: Error | null = null

  channel.onmessage = (packet) => {
    if (packetError) {
      return
    }

    try {
      if (packet.type === "started") {
        status = packet.status
      } else if (packet.type === "chunk") {
        const bytes = Uint8Array.from(packet.bytes)
        if (status !== null && status >= 200 && status < 300) {
          session.push(bytes)
        } else {
          errorChunks.push(bytes)
        }
      } else if (packet.type === "end") {
        ended = true
      } else {
        packetError = new ChatStreamRequestError(packet.code, packet.message)
      }
    } catch (error) {
      packetError = error instanceof Error ? error : new Error(String(error))
      void invoke("cancel_chat", { requestId })
    }
  }

  const handleAbort = () => {
    void invoke("cancel_chat", { requestId })
  }
  signal.addEventListener("abort", handleAbort, { once: true })

  try {
    await invoke("stream_chat", {
      requestId,
      workerUrl: __CURIO_CHAT_WORKER_URL__,
      request,
      onPacket: channel,
    })

    if (signal.aborted) {
      throw abortError()
    }
    if (packetError) {
      throw packetError
    }
    if (status === null) {
      throw new ChatStreamRequestError(
        "missing_status",
        "The desktop bridge did not report the chat worker status.",
      )
    }
    if (!ended) {
      throw new ChatStreamRequestError(
        "incomplete_transport",
        "The desktop bridge ended before the chat response was complete.",
      )
    }
    if (status < 200 || status >= 300) {
      const responseText = new TextDecoder().decode(concatBytes(errorChunks))
      throw new ChatStreamRequestError(
        "http_error",
        workerErrorMessage(responseText) ??
          `Chat worker returned HTTP ${status}.`,
      )
    }

    return session.finish()
  } catch (error) {
    if (signal.aborted) {
      throw abortError()
    }
    if (error instanceof Error) {
      throw error
    }
    throw new ChatStreamRequestError(
      "desktop_bridge_error",
      typeof error === "string"
        ? error
        : "The desktop chat bridge could not complete the request.",
    )
  } finally {
    signal.removeEventListener("abort", handleAbort)
    channel.onmessage = () => undefined
  }
}

function concatBytes(chunks: Uint8Array[]): Uint8Array {
  const length = chunks.reduce((total, chunk) => total + chunk.byteLength, 0)
  const combined = new Uint8Array(length)
  let offset = 0
  for (const chunk of chunks) {
    combined.set(chunk, offset)
    offset += chunk.byteLength
  }
  return combined
}

export const platformServices: PlatformServices = {
  target: "desktop",
  chat: { stream: streamDesktopChat },
  async openExternalUrl(value) {
    const url = parseExternalHttpUrl(value)
    if (url.protocol !== "https:") {
      throw new Error("The desktop app can only open secure HTTPS links.")
    }
    await openUrl(url.href)
  },
}
