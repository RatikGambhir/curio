import { describe, expect, it } from "vitest"

import {
  HOME_THOUGHT_MAX_LENGTH,
  homeThoughtStorageKey,
  readHomeThought,
  writeHomeThought,
} from "@/features/home/thought-storage"

function memoryStorage(): Storage {
  const entries = new Map<string, string>()
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

describe("seedling thought storage", () => {
  it("keeps drafts separate for each signed-in user", () => {
    const storage = memoryStorage()

    writeHomeThought(storage, "user-a", "First draft")
    writeHomeThought(storage, "user-b", "Second draft")

    expect(readHomeThought(storage, "user-a")).toBe("First draft")
    expect(readHomeThought(storage, "user-b")).toBe("Second draft")
    expect(homeThoughtStorageKey("person+test@example.com")).not.toContain("+")
  })

  it("caps stored and restored drafts at the textarea limit", () => {
    const storage = memoryStorage()
    const oversized = "x".repeat(HOME_THOUGHT_MAX_LENGTH + 20)

    writeHomeThought(storage, "user-a", oversized)

    expect(readHomeThought(storage, "user-a")).toHaveLength(
      HOME_THOUGHT_MAX_LENGTH,
    )
  })

  it("continues safely when browser storage is unavailable", () => {
    const throwingStorage = {
      ...memoryStorage(),
      getItem: () => {
        throw new Error("denied")
      },
      setItem: () => {
        throw new Error("quota")
      },
    } as unknown as Storage

    expect(readHomeThought(throwingStorage, "user-a")).toBe("")
    expect(() =>
      writeHomeThought(throwingStorage, "user-a", "Draft"),
    ).not.toThrow()
  })
})
