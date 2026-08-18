import type { AuthUser } from "@/types/LoginRegisterTypes"

export const AUTH_STORAGE_KEY = "curio-mock-auth-user-v1"
const LEGACY_AUTH_STORAGE_KEYS = [
  "curio-web-mock-auth-user-v1",
  "curio-desktop-mock-auth-user-v1",
] as const

export function buildMockUser(email: string): AuthUser {
  const normalizedEmail = email.trim().toLowerCase()
  const localPart = normalizedEmail.split("@")[0] ?? "curio"
  const name = localPart
    .split(/[._-]/g)
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ")

  return {
    id: `mock-${normalizedEmail}`,
    name: name || "Curio User",
    email: normalizedEmail,
    avatar: "",
  }
}

export function parseStoredUser(value: string): AuthUser | null {
  try {
    const user = JSON.parse(value) as Partial<AuthUser>
    if (typeof user.email !== "string" || !user.email.trim()) {
      return null
    }
    return buildMockUser(user.email)
  } catch {
    return null
  }
}

export function readStoredUser(storage: Storage): AuthUser | null {
  for (const key of [AUTH_STORAGE_KEY, ...LEGACY_AUTH_STORAGE_KEYS]) {
    const value = storage.getItem(key)
    if (!value) {
      continue
    }

    const user = parseStoredUser(value)
    if (!user) {
      storage.removeItem(key)
      continue
    }

    storage.setItem(AUTH_STORAGE_KEY, JSON.stringify(user))
    for (const legacyKey of LEGACY_AUTH_STORAGE_KEYS) {
      storage.removeItem(legacyKey)
    }
    return user
  }
  return null
}
