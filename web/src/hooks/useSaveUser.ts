import { useMutation } from "@tanstack/react-query"

import { saveUser, type SaveUserInput } from "@/api/users"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"

/**
 * Persists a user profile through the platform transport: fetch on web, the
 * Rust-side HTTP proxy on desktop. The mock auth session's user id doubles as
 * the placeholder bearer token until real auth lands.
 */
export function useSaveUser() {
  const { user } = useAuthenticatedUser()

  return useMutation({
    mutationFn: (input: SaveUserInput) =>
      saveUser(input, { bearerToken: user?.id }),
  })
}
