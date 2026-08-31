import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react"
import { nanoid } from "nanoid"

import { ThemeContext, type ThemeContextValue } from "@/constants/ThemeContext"
import { normalizeHexColor } from "@/features/theme/color"
import {
  applyTheme,
  customThemeToDefinition,
  findTheme,
  listThemes,
  MAX_CUSTOM_THEMES,
  normalizeThemeLabel,
  PREFERS_DARK_QUERY,
  readThemePreference,
  resolveAppearance,
  writeThemePreference,
  type Appearance,
  type ThemeDefinition,
  type ThemePreference,
} from "@/features/theme/theme"

function systemPrefersDark(): boolean {
  return window.matchMedia(PREFERS_DARK_QUERY).matches
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [preference, setPreference] = useState<ThemePreference>(() =>
    readThemePreference(window.localStorage),
  )
  const [prefersDark, setPrefersDark] = useState(systemPrefersDark)

  const themes = useMemo(
    () => listThemes(preference.customThemes),
    [preference.customThemes],
  )
  const theme = useMemo(
    () => findTheme(preference.themeId, preference.customThemes),
    [preference.customThemes, preference.themeId],
  )
  const resolvedAppearance = resolveAppearance(
    preference.appearance,
    prefersDark,
  )

  /* Layout effect so <html> carries the palette before the browser paints;
     a plain effect lets the fallback seed flash first. */
  useLayoutEffect(() => {
    applyTheme(document.documentElement, theme, resolvedAppearance)
  }, [resolvedAppearance, theme])

  useEffect(() => {
    const query = window.matchMedia(PREFERS_DARK_QUERY)
    const handleChange = (event: MediaQueryListEvent) =>
      setPrefersDark(event.matches)

    query.addEventListener("change", handleChange)
    return () => query.removeEventListener("change", handleChange)
  }, [])

  const update = useCallback(
    (patch: (current: ThemePreference) => ThemePreference) => {
      setPreference((current) => {
        const next = patch(current)
        writeThemePreference(window.localStorage, next)
        return next
      })
    },
    [],
  )

  const setThemeId = useCallback(
    (themeId: string) => update((current) => ({ ...current, themeId })),
    [update],
  )

  const setAppearance = useCallback(
    (appearance: Appearance) => update((current) => ({ ...current, appearance })),
    [update],
  )

  const addTheme = useCallback(
    (color: string, label?: string): ThemeDefinition | null => {
      const normalizedColor = normalizeHexColor(color)
      if (
        !normalizedColor ||
        preference.customThemes.length >= MAX_CUSTOM_THEMES
      ) {
        return null
      }

      const created = {
        id: `custom-${nanoid(8)}`,
        label: normalizeThemeLabel(label, normalizedColor),
        color: normalizedColor,
      }

      update((current) => ({
        ...current,
        themeId: created.id,
        customThemes: [...current.customThemes, created],
      }))

      return customThemeToDefinition(created)
    },
    [preference.customThemes.length, update],
  )

  /* Deleting the selected theme falls back to the default rather than leaving
     the app pointing at a theme that no longer exists. */
  const removeTheme = useCallback(
    (themeId: string) => {
      update((current) => ({
        ...current,
        themeId:
          current.themeId === themeId
            ? findTheme("", current.customThemes).id
            : current.themeId,
        customThemes: current.customThemes.filter(
          (custom) => custom.id !== themeId,
        ),
      }))
    },
    [update],
  )

  const value = useMemo<ThemeContextValue>(
    () => ({
      themes,
      theme,
      appearance: preference.appearance,
      resolvedAppearance,
      canAddTheme: preference.customThemes.length < MAX_CUSTOM_THEMES,
      setThemeId,
      setAppearance,
      addTheme,
      removeTheme,
    }),
    [
      addTheme,
      preference.appearance,
      preference.customThemes.length,
      removeTheme,
      resolvedAppearance,
      setAppearance,
      setThemeId,
      theme,
      themes,
    ],
  )

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
}
