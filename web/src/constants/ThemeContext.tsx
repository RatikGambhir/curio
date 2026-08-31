import { createContext } from "react"

import type {
  Appearance,
  ResolvedAppearance,
  ThemeDefinition,
} from "@/features/theme/theme"

export type ThemeContextValue = {
  /* Built-ins plus anything the user has added, in picker order. */
  themes: ThemeDefinition[]
  theme: ThemeDefinition
  appearance: Appearance
  resolvedAppearance: ResolvedAppearance
  canAddTheme: boolean
  setThemeId: (themeId: string) => void
  setAppearance: (appearance: Appearance) => void
  /* Returns the created theme, or null when the colour could not be parsed or
     the limit has been reached. Selects the new theme on success. */
  addTheme: (color: string, label?: string) => ThemeDefinition | null
  removeTheme: (themeId: string) => void
}

export const ThemeContext = createContext<ThemeContextValue | null>(null)
