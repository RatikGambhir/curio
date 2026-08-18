import type { AuthContextValue } from "@/types/LoginRegisterTypes";
import { createContext } from "react";

export const AuthContext = createContext<AuthContextValue | null>(null);

// Hook to consume the context
