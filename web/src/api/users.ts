import { api, type ApiRequestOptions } from "@/api/client"

export type SaveUserInput = {
  id: string
  name: string
  email: string
  avatarUrl?: string
}

export type UserRecord = {
  id: string
  name: string
  email: string
  avatarUrl: string | null
  createdAt: string
  updatedAt: string
}

export function saveUser(
  input: SaveUserInput,
  options?: ApiRequestOptions,
): Promise<UserRecord> {
  return api.post<UserRecord>("/v1/users", input, options)
}
