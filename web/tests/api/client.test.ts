import { afterEach, describe, expect, it, vi } from "vitest"

import { api } from "@/api/client"

describe("api client", () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it("parses successful JSON responses", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async (input: URL | RequestInfo, init?: RequestInit) => {
        expect(String(input)).toBe("http://127.0.0.1:3000/v1/users")
        expect(init?.method).toBe("POST")
        expect(new Headers(init?.headers).get("Authorization")).toBe(
          "Bearer user-1",
        )
        return new Response(JSON.stringify({ id: "user-1", name: "Curio" }), {
          status: 201,
        })
      }),
    )

    const record = await api.post<{ id: string; name: string }>(
      "/v1/users",
      { id: "user-1", name: "Curio" },
      { bearerToken: "user-1" },
    )

    expect(record).toEqual({ id: "user-1", name: "Curio" })
  })

  it("maps error bodies to an ApiError with the service message", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(
        async () =>
          new Response(JSON.stringify({ error: "That email is taken." }), {
            status: 409,
          }),
      ),
    )

    await expect(api.get("/v1/users")).rejects.toMatchObject({
      name: "ApiError",
      status: 409,
      message: "That email is taken.",
    })
  })

  it("returns undefined for empty success bodies", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(null, { status: 204 })),
    )

    await expect(api.delete("/v1/conversations/c-1")).resolves.toBeUndefined()
  })
})
