import { afterEach, describe, expect, it, vi } from "vitest"

import { platformServices } from "@/platform/web"

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

describe("web service transport", () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it("posts JSON bodies and streams chunks after reporting the status", async () => {
    const encoder = new TextEncoder()
    const fetchMock = vi.fn(async (input: URL | RequestInfo, init?: RequestInit) => {
      expect(String(input)).toBe("http://127.0.0.1:3000/v1/chat/stream")
      expect(init?.method).toBe("POST")
      expect(new Headers(init?.headers).get("Content-Type")).toBe(
        "application/json",
      )
      expect(JSON.parse(String(init?.body))).toEqual({ prompt: "Hello" })

      return new Response(
        new ReadableStream({
          start(controller) {
            controller.enqueue(encoder.encode("first "))
            controller.enqueue(encoder.encode("second"))
            controller.close()
          },
        }),
      )
    })
    vi.stubGlobal("fetch", fetchMock)
    const { calls, handlers } = collectingHandlers()

    await platformServices.service.stream(
      { method: "POST", path: "/v1/chat/stream", body: { prompt: "Hello" } },
      handlers,
    )

    expect(fetchMock).toHaveBeenCalledOnce()
    expect(calls).toEqual([
      ["status", 200],
      ["chunk", "first "],
      ["chunk", "second"],
    ])
  })

  it("serializes GET queries and attaches bearer tokens", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo, init?: RequestInit) => {
      const url = new URL(String(input))
      expect(url.pathname).toBe("/v1/conversations")
      expect(url.searchParams.get("limit")).toBe("10")
      expect(init?.method).toBe("GET")
      expect(init?.body).toBeUndefined()
      expect(new Headers(init?.headers).get("Authorization")).toBe(
        "Bearer token-1",
      )
      return new Response("[]")
    })
    vi.stubGlobal("fetch", fetchMock)
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

    expect(calls).toEqual([
      ["status", 200],
      ["chunk", "[]"],
    ])
  })

  it("maps network failures to a transport error", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new TypeError("Failed to fetch")
      }),
    )
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

  it("reports the status of empty-bodied responses", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(null, { status: 204 })),
    )
    const { calls, handlers } = collectingHandlers()

    await platformServices.service.stream(
      { method: "DELETE", path: "/v1/conversations/conversation-1" },
      handlers,
    )

    expect(calls).toEqual([["status", 204]])
  })
})
