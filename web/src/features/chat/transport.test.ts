import { describe, expect, it } from "vitest"

import { streamChat } from "@/features/chat/transport"
import type {
  ServiceRequestInit,
  ServiceStreamHandlers,
  ServiceTransport,
} from "@/platform/contracts"

const request = {
  userId: "mock-user-1",
  conversationId: "conversation-1",
  userMessageId: "user-1",
  assistantMessageId: "assistant-1",
  prompt: "Hello",
}

function block(event: string, data: Record<string, unknown>) {
  return `event: ${event}\ndata: ${JSON.stringify(data)}\n\n`
}

function fakeTransport(
  status: number,
  chunks: string[],
  onRequest?: (init: ServiceRequestInit) => void,
): ServiceTransport {
  const encoder = new TextEncoder()
  return {
    async stream(init: ServiceRequestInit, handlers: ServiceStreamHandlers) {
      onRequest?.(init)
      handlers.onStatus(status)
      for (const chunk of chunks) {
        handlers.onChunk(encoder.encode(chunk))
      }
    },
  }
}

describe("streamChat", () => {
  it("posts the chat contract and emits parsed events from raw chunks", async () => {
    const ids = { conversationId: "conversation-1", messageId: "assistant-1" }
    const body =
      block("token", { ...ids, token: "Hello" }) +
      block("done", { ...ids, responseId: "response-1" })
    let seenInit: ServiceRequestInit | undefined
    const transport = fakeTransport(
      200,
      [body.slice(0, 29), body.slice(29)],
      (init) => {
        seenInit = init
      },
    )
    const received: string[] = []

    const terminal = await streamChat(
      request,
      {
        signal: new AbortController().signal,
        onEvent: (event) => received.push(event.type),
      },
      transport,
    )

    expect(seenInit).toMatchObject({
      method: "POST",
      path: "/v1/chat/stream",
      body: request,
    })
    expect(received).toEqual(["token", "done"])
    expect(terminal.type).toBe("done")
  })

  it("maps non-2xx responses to a request error with the service message", async () => {
    const transport = fakeTransport(429, [
      JSON.stringify({ error: "Too many requests." }),
    ])

    await expect(
      streamChat(
        request,
        { signal: new AbortController().signal, onEvent: () => undefined },
        transport,
      ),
    ).rejects.toMatchObject({
      code: "http_error",
      message: "Too many requests.",
    })
  })

  it("rejects transports that never report a status", async () => {
    const transport: ServiceTransport = {
      async stream() {},
    }

    await expect(
      streamChat(
        request,
        { signal: new AbortController().signal, onEvent: () => undefined },
        transport,
      ),
    ).rejects.toMatchObject({ code: "missing_status" })
  })

  it("rejects streams that end without a terminal event", async () => {
    const transport = fakeTransport(200, [
      block("token", {
        conversationId: "conversation-1",
        messageId: "assistant-1",
        token: "Hello",
      }),
    ])

    await expect(
      streamChat(
        request,
        { signal: new AbortController().signal, onEvent: () => undefined },
        transport,
      ),
    ).rejects.toMatchObject({ code: "truncated_stream" })
  })
})
