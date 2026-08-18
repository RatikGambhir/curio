import { beforeEach, describe, expect, it, vi } from "vitest"

const tauri = vi.hoisted(() => ({
  invoke: vi.fn(),
  openUrl: vi.fn(),
}))

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class<T> {
    onmessage: (packet: T) => void = () => undefined
  },
  invoke: tauri.invoke,
}))

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: tauri.openUrl,
}))

import { platformServices } from "@/platform/desktop"

type TestChannel = {
  onmessage(packet: unknown): void
}

describe("desktop chat transport", () => {
  beforeEach(() => {
    tauri.invoke.mockReset()
    tauri.openUrl.mockReset()
  })

  it("parses raw channel bytes with the shared stream parser", async () => {
    const encoder = new TextEncoder()
    const events: string[] = []
    tauri.invoke.mockImplementation(async (command, args) => {
      expect(command).toBe("stream_chat")
      const channel = (args as { onPacket: TestChannel }).onPacket
      channel.onmessage({ type: "started", status: 200 })
      const response =
        'event: token\ndata: {"conversationId":"conversation-1","messageId":"assistant-1","token":"Hello"}\n\n' +
        'event: done\ndata: {"conversationId":"conversation-1","messageId":"assistant-1","responseId":"response-1"}\n\n'
      const bytes = encoder.encode(response)
      channel.onmessage({ type: "chunk", bytes: Array.from(bytes.slice(0, 31)) })
      channel.onmessage({ type: "chunk", bytes: Array.from(bytes.slice(31)) })
      channel.onmessage({ type: "end" })
    })

    const terminal = await platformServices.chat.stream(
      {
        userId: "mock-user-1",
        conversationId: "conversation-1",
        userMessageId: "user-1",
        assistantMessageId: "assistant-1",
        prompt: "Hello",
      },
      {
        signal: new AbortController().signal,
        onEvent: (event) => events.push(event.type),
      },
    )

    expect(events).toEqual(["token", "done"])
    expect(terminal.type).toBe("done")
  })

  it("cancels the native request when its AbortSignal fires", async () => {
    let finishStream: (() => void) | undefined
    tauri.invoke.mockImplementation((command) => {
      if (command === "stream_chat") {
        return new Promise<void>((resolve) => {
          finishStream = resolve
        })
      }
      if (command === "cancel_chat") {
        finishStream?.()
        return Promise.resolve(true)
      }
      return Promise.resolve()
    })

    const controller = new AbortController()
    const result = platformServices.chat.stream(
      {
        userId: "mock-user-1",
        conversationId: "conversation-1",
        userMessageId: "user-1",
        assistantMessageId: "assistant-1",
        prompt: "Hello",
      },
      { signal: controller.signal, onEvent: () => undefined },
    )
    controller.abort()

    await expect(result).rejects.toMatchObject({ name: "AbortError" })
    expect(tauri.invoke).toHaveBeenCalledWith("cancel_chat", {
      requestId: expect.any(String),
    })
  })
})
