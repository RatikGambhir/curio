import { beforeEach, describe, expect, it, vi } from "vitest"

const tauri = vi.hoisted(() => ({
  invoke: vi.fn(),
}))

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class<T> {
    onmessage: (packet: T) => void = () => undefined
  },
  invoke: tauri.invoke,
}))

import { platformServices } from "@/platform/desktop"

type TestChannel = {
  onmessage(packet: unknown): void
}

function encodeChunk(value: string): string {
  return btoa(value)
}

function collectingHandlers() {
  const calls: Array<["status", number] | ["chunk", string]> = []
  const decoder = new TextDecoder()
  return {
    calls,
    handlers: {
      onStatus: (status: number) => calls.push(["status", status]),
      onChunk: (bytes: Uint8Array) =>
        calls.push(["chunk", decoder.decode(bytes)]),
    },
  }
}

describe("desktop service transport", () => {
  beforeEach(() => {
    tauri.invoke.mockReset()
  })

  it("routes external URL opening through the app-owned Tauri command", async () => {
    tauri.invoke.mockResolvedValue(undefined)

    await platformServices.openExternalUrl("https://example.com/docs")

    expect(tauri.invoke).toHaveBeenCalledWith("open_external_url", {
      url: "https://example.com/docs",
    })
  })

  it("invokes the service command and decodes base64 channel chunks", async () => {
    tauri.invoke.mockImplementation(async (command, args) => {
      expect(command).toBe("service_request")
      expect(args).toMatchObject({
        requestId: expect.any(String),
        method: "POST",
        path: "/v1/chat/stream",
        payload: { prompt: "Hello" },
        bearerToken: null,
      })
      const channel = (args as { onPacket: TestChannel }).onPacket
      channel.onmessage({ type: "started", status: 200 })
      channel.onmessage({ type: "chunk", bytes: encodeChunk("first ") })
      channel.onmessage({ type: "chunk", bytes: encodeChunk("second") })
      channel.onmessage({ type: "end" })
    })
    const { calls, handlers } = collectingHandlers()

    await platformServices.service.stream(
      { method: "POST", path: "/v1/chat/stream", body: { prompt: "Hello" } },
      handlers,
    )

    expect(calls).toEqual([
      ["status", 200],
      ["chunk", "first "],
      ["chunk", "second"],
    ])
  })

  it("sends GET queries as the command payload with the bearer token", async () => {
    tauri.invoke.mockImplementation(async (command, args) => {
      expect(command).toBe("service_request")
      expect(args).toMatchObject({
        method: "GET",
        path: "/v1/conversations",
        payload: { limit: 10 },
        bearerToken: "token-1",
      })
      const channel = (args as { onPacket: TestChannel }).onPacket
      channel.onmessage({ type: "started", status: 200 })
      channel.onmessage({ type: "end" })
    })
    const { calls, handlers } = collectingHandlers()

    await platformServices.service.stream(
      {
        method: "GET",
        path: "/v1/conversations",
        query: { limit: 10 },
        bearerToken: "token-1",
      },
      handlers,
    )

    expect(calls).toEqual([["status", 200]])
  })

  it("surfaces error packets as transport errors", async () => {
    tauri.invoke.mockImplementation(async (_command, args) => {
      const channel = (args as { onPacket: TestChannel }).onPacket
      channel.onmessage({
        type: "error",
        code: "service_unavailable",
        message: "The Curio service is unavailable.",
      })
    })
    const { handlers } = collectingHandlers()

    await expect(
      platformServices.service.stream(
        { method: "GET", path: "/health" },
        handlers,
      ),
    ).rejects.toMatchObject({
      name: "ServiceTransportError",
      code: "service_unavailable",
    })
  })

  it("cancels the native request when its AbortSignal fires", async () => {
    let finishStream: (() => void) | undefined
    tauri.invoke.mockImplementation((command) => {
      if (command === "service_request") {
        return new Promise<void>((resolve) => {
          finishStream = resolve
        })
      }
      if (command === "cancel_request") {
        finishStream?.()
        return Promise.resolve(true)
      }
      return Promise.resolve()
    })

    const controller = new AbortController()
    const { handlers } = collectingHandlers()
    const result = platformServices.service.stream(
      {
        method: "POST",
        path: "/v1/chat/stream",
        body: { prompt: "Hello" },
        signal: controller.signal,
      },
      handlers,
    )
    controller.abort()

    await expect(result).rejects.toMatchObject({ name: "AbortError" })
    expect(tauri.invoke).toHaveBeenCalledWith("cancel_request", {
      requestId: expect.any(String),
    })
  })

  it("cancels the native request when a handler throws", async () => {
    tauri.invoke.mockImplementation(async (command, args) => {
      if (command !== "service_request") {
        return
      }
      const channel = (args as { onPacket: TestChannel }).onPacket
      channel.onmessage({ type: "started", status: 200 })
      channel.onmessage({ type: "chunk", bytes: encodeChunk("boom") })
    })

    await expect(
      platformServices.service.stream(
        { method: "GET", path: "/health" },
        {
          onStatus: () => undefined,
          onChunk: () => {
            throw new Error("handler failure")
          },
        },
      ),
    ).rejects.toThrow("handler failure")
    expect(tauri.invoke).toHaveBeenCalledWith("cancel_request", {
      requestId: expect.any(String),
    })
  })
})
