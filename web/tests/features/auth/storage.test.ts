import { describe, expect, it } from "vitest"

import {
  AUTH_STORAGE_KEY,
  parseStoredUser,
  readStoredUser,
} from "@/features/auth/storage"

function memoryStorage(values: Record<string, string>): Storage {
  const entries = new Map(Object.entries(values))
  return {
    get length() {
      return entries.size
    },
    clear: () => entries.clear(),
    getItem: (key) => entries.get(key) ?? null,
    key: (index) => Array.from(entries.keys())[index] ?? null,
    removeItem: (key) => {
      entries.delete(key)
    },
    setItem: (key, value) => {
      entries.set(key, value)
    },
  }
}

describe("auth storage", () => {
  it("normalizes valid users and rejects invalid data", () => {
    expect(parseStoredUser('{"email":"Ada.Lovelace@Example.com"}')).toMatchObject({
      id: "mock-ada.lovelace@example.com",
      name: "Ada Lovelace",
      email: "ada.lovelace@example.com",
    })
    expect(parseStoredUser("not-json")).toBeNull()
    expect(parseStoredUser('{"name":"Missing email"}')).toBeNull()
  })

  it("cleans invalid data and migrates a legacy target-specific key", () => {
    const storage = memoryStorage({
      [AUTH_STORAGE_KEY]: "invalid-json",
      "curio-desktop-mock-auth-user-v1": '{"email":"desktop@example.com"}',
    })

    expect(readStoredUser(storage)).toMatchObject({ email: "desktop@example.com" })
    expect(storage.getItem(AUTH_STORAGE_KEY)).toContain("desktop@example.com")
    expect(storage.getItem("curio-desktop-mock-auth-user-v1")).toBeNull()
  })
})
