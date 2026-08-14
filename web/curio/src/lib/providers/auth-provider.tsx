import { useCallback, useMemo, useState, type ReactNode } from "react";
import { AuthContext } from "../../constants/AuthContext.tsx";
import type {
  AuthContextValue,
} from "@/types/LoginRegisterTypes.ts";
import {
  AUTH_STORAGE_KEY,
  buildMockUser,
  readStoredUser,
} from "@/features/auth/storage";

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState(() => readStoredUser(window.localStorage));

  const loginUser = useCallback((email: string) => {
    const normalizedEmail = email.trim().toLowerCase();
    const mockUser = buildMockUser(normalizedEmail);
    window.localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(mockUser));
    setUser(mockUser);
  }, []);

  const logoutUser = useCallback(() => {
    window.localStorage.removeItem(AUTH_STORAGE_KEY);
    setUser(null);
  }, []);

  const value = useMemo<AuthContextValue>(
    () => ({
      user,
      isAuthenticated: Boolean(user),
      loginUser,
      logoutUser,
    }),
    [loginUser, logoutUser, user],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
