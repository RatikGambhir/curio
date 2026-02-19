import { AuthContext } from "../constants/AuthContext";
import { useContext } from "react";

export function useAuthenticatedUser() {
  const ctx = useContext(AuthContext);
  if (!ctx)
    throw new Error("useAuthenticatedUser must be used inside an AuthProvider");
  return ctx;
}
