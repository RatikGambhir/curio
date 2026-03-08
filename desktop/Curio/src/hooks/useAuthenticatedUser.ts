import { useAuth } from "@/components/auth-provider";

export function useAuthenticatedUser() {
  return useAuth();
}
