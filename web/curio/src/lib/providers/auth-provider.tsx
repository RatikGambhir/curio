// src/auth/AuthProvider.tsx
import { useState, type ReactNode } from "react";
import { AuthContext } from "../../constants/AuthContext.tsx";
import type { AuthContextValue } from "@/types/LoginRegisterTypes.ts";

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<string | null>(null);

  //TODO: Implement user state management with id, email, name, token, and refresh token
  function loginUser(id: string | null) {
    setUser(id);
  }

  function logoutUser() {
    setUser(null);
  }

  const value: AuthContextValue = {
    user,
    isAuthenticated: !!user,
    loginUser,
    logoutUser,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
