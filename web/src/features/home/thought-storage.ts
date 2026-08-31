export const HOME_THOUGHT_MAX_LENGTH = 500

const HOME_THOUGHT_STORAGE_PREFIX = "curio-seedling-thought-v1"

export function homeThoughtStorageKey(userId: string): string {
  return `${HOME_THOUGHT_STORAGE_PREFIX}:${encodeURIComponent(userId)}`
}

export function readHomeThought(storage: Storage, userId: string): string {
  try {
    return (storage.getItem(homeThoughtStorageKey(userId)) ?? "").slice(
      0,
      HOME_THOUGHT_MAX_LENGTH,
    )
  } catch {
    return ""
  }
}

export function writeHomeThought(
  storage: Storage,
  userId: string,
  thought: string,
): void {
  try {
    const key = homeThoughtStorageKey(userId)
    const value = thought.slice(0, HOME_THOUGHT_MAX_LENGTH)
    if (value) {
      storage.setItem(key, value)
    } else {
      storage.removeItem(key)
    }
  } catch {
    // The draft remains in React state when storage is unavailable or full.
  }
}
