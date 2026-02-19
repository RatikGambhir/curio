// src/auth/AuthProvider.tsx
import { useState, type ReactNode } from "react";
import { AuthContext } from "../constants/AuthContext";
import type { AuthContextValue } from "@/types/LoginRegisterTypes";

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<string | null>(null);

  //TODO: Implement user state management with id, email, name, token, and refresh token
  function loginUser(id: string | null) {
    setUser(id);
  }

  const value: AuthContextValue = {
    user,
    isAuthenticated: !!user,
    loginUser,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
