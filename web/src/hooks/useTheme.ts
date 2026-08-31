import { useContext } from "react"

import { ThemeContext } from "@/constants/ThemeContext"

export function useTheme() {
  const ctx = useContext(ThemeContext)
  if (!ctx) {
    throw new Error("useTheme must be used inside a ThemeProvider")
  }
  return ctx
}
