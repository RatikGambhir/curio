import { useCallback, useMemo, useState, type ReactNode } from "react";
import { AuthContext } from "../../constants/AuthContext.tsx";
import type {
  AuthContextValue,
  AuthUser,
} from "@/types/LoginRegisterTypes.ts";

const AUTH_STORAGE_KEY = "curio-web-mock-auth-user-v1";

function buildMockUser(email: string): AuthUser {
  const localPart = email.split("@")[0] ?? "curio";
  const name = localPart
    .split(/[._-]/g)
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");

  return {
    id: `mock-${email}`,
    name: name || "Curio User",
    email,
    avatar: "",
  };
}

function readStoredUser(): AuthUser | null {
  const storedUser = window.localStorage.getItem(AUTH_STORAGE_KEY);
  if (!storedUser) {
    return null;
  }

  try {
    const user = JSON.parse(storedUser) as Partial<AuthUser>;
    if (typeof user.email !== "string" || !user.email) {
      window.localStorage.removeItem(AUTH_STORAGE_KEY);
      return null;
    }
    return buildMockUser(user.email);
  } catch {
    window.localStorage.removeItem(AUTH_STORAGE_KEY);
    return null;
  }
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<AuthUser | null>(readStoredUser);

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
