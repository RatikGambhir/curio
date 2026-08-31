import {
  normalizeHexColor,
  seedFromHexColor,
  type ThemeSeed,
} from "@/features/theme/color"

export const THEME_CLASS = "curio-theme"
export const THEME_ATTRIBUTE = "data-curio-theme"
export const THEME_STORAGE_KEY = "curio-theme-v1"
export const THEME_HUE_PROPERTY = "--theme-hue"
export const THEME_CHROMA_PROPERTY = "--theme-chroma"

export const MAX_CUSTOM_THEMES = 12
const MAX_LABEL_LENGTH = 32

export type Appearance = "light" | "dark" | "system"
export type ResolvedAppearance = Exclude<Appearance, "system">

export type ThemeOrigin = "builtin" | "custom"

export type ThemeDefinition = {
  id: string
  label: string
  description: string
  /* The seed. Every token in index.css is derived from this one colour. */
  color: string
  origin: ThemeOrigin
}

export type CustomTheme = {
  id: string
  label: string
  color: string
}

export type ThemePreference = {
  themeId: string
  appearance: Appearance
  customThemes: CustomTheme[]
}

export type AppearanceOption = {
  value: Appearance
  label: string
}

/* Adding a built-in theme is adding an entry here. The palette is generated
   from `color`, so no CSS change is required — but keep the `:root` fallback
   seed in index.css in step with whichever entry is the default.

   Each `color` is its palette's step 500, the brand tone. Because the seed is
   normalised by vividness rather than taken literally, any step of the same hue
   would yield the same theme; 500 is used because it is the colour a reader
   would call the brand. */
export const BUILTIN_THEMES: readonly ThemeDefinition[] = [
  {
    id: "sage",
    label: "Sage Green",
    description: "Calm garden greens on pale linen.",
    color: "#7da25d",
    origin: "builtin",
  },
  {
    id: "sky",
    label: "Cool Sky Blue",
    description: "Crisp cobalt blues on cool white.",
    color: "#1b8ae4",
    origin: "builtin",
  },
  {
    id: "steel",
    label: "Cool Steel",
    description: "Restrained blue-greys on brushed steel.",
    color: "#6b8394",
    origin: "builtin",
  },
  {
    id: "charcoal",
    label: "Charcoal Blue",
    description: "Sophisticated charcoal-navy on cool linen.",
    color: "#678498",
    origin: "builtin",
  },
]

export const APPEARANCE_OPTIONS: readonly AppearanceOption[] = [
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
  { value: "system", label: "System" },
]

/* Light rather than system: the app shipped light-only until now, so following
   the OS would silently move existing users into a palette they never chose. */
export const DEFAULT_THEME_PREFERENCE: ThemePreference = {
  themeId: "steel",
  appearance: "light",
  customThemes: [],
}

export const PREFERS_DARK_QUERY = "(prefers-color-scheme: dark)"

/* Cool Steel, matching the `:root` fallback in index.css. */
export const DEFAULT_SEED: ThemeSeed = { hue: 239.12, chroma: 0.0338 }

function isAppearance(value: unknown): value is Appearance {
  return APPEARANCE_OPTIONS.some((option) => option.value === value)
}

function parseCustomTheme(value: unknown): CustomTheme | null {
  if (typeof value !== "object" || value === null) {
    return null
  }

  const candidate = value as Partial<CustomTheme>
  const color = typeof candidate.color === "string"
    ? normalizeHexColor(candidate.color)
    : null

  const id = typeof candidate.id === "string" ? candidate.id.trim() : ""

  if (!color || !id) {
    return null
  }

  return {
    id,
    label: normalizeThemeLabel(candidate.label, color),
    color,
  }
}

export function normalizeThemeLabel(
  label: string | undefined,
  color: string,
): string {
  const trimmed = label?.trim().slice(0, MAX_LABEL_LENGTH)
  return trimmed || color.toUpperCase()
}

/* Built-in ids that have been retired. Sage Green replaced Dark Spruce, so a
   preference still naming the old theme lands on its successor instead of
   falling through to the default and looking like the choice was forgotten. A
   Map rather than an object literal, so a stored id like "constructor" cannot
   reach Object.prototype. */
const RENAMED_THEME_IDS = new Map([["spruce", "sage"]])

/* Each field falls back on its own so a preference written by an older build,
   or one naming a theme that has since been deleted, still contributes the
   parts it got right instead of resetting everything. */
export function parseThemePreference(value: string | null): ThemePreference {
  if (!value) {
    return DEFAULT_THEME_PREFERENCE
  }

  try {
    const parsed = JSON.parse(value) as Partial<ThemePreference>
    const parsedCustomThemes = Array.isArray(parsed.customThemes)
      ? parsed.customThemes
          .map(parseCustomTheme)
          .filter((theme): theme is CustomTheme => theme !== null)
      : []
    const reservedIds = new Set(BUILTIN_THEMES.map((theme) => theme.id))
    const customThemes: CustomTheme[] = []
    for (const theme of parsedCustomThemes) {
      if (reservedIds.has(theme.id)) {
        continue
      }
      reservedIds.add(theme.id)
      customThemes.push(theme)
      if (customThemes.length === MAX_CUSTOM_THEMES) {
        break
      }
    }

    const themeId =
      typeof parsed.themeId === "string" && parsed.themeId
        ? parsed.themeId
        : DEFAULT_THEME_PREFERENCE.themeId

    return {
      themeId: RENAMED_THEME_IDS.get(themeId) ?? themeId,
      appearance: isAppearance(parsed.appearance)
        ? parsed.appearance
        : DEFAULT_THEME_PREFERENCE.appearance,
      customThemes,
    }
  } catch {
    return DEFAULT_THEME_PREFERENCE
  }
}

export function readThemePreference(storage: Storage): ThemePreference {
  try {
    return parseThemePreference(storage.getItem(THEME_STORAGE_KEY))
  } catch {
    return DEFAULT_THEME_PREFERENCE
  }
}

/* Storage throws in private-mode Safari and when the quota is full. A theme is
   cosmetic, so a failed write must not take down the render that triggered it. */
export function writeThemePreference(
  storage: Storage,
  preference: ThemePreference,
): void {
  try {
    storage.setItem(THEME_STORAGE_KEY, JSON.stringify(preference))
  } catch {
    /* the preference stays in memory for this session */
  }
}

export function customThemeToDefinition(theme: CustomTheme): ThemeDefinition {
  return {
    id: theme.id,
    label: theme.label,
    description: theme.color.toUpperCase(),
    color: theme.color,
    origin: "custom",
  }
}

export function listThemes(
  customThemes: readonly CustomTheme[],
): ThemeDefinition[] {
  return [...BUILTIN_THEMES, ...customThemes.map(customThemeToDefinition)]
}

export function findTheme(
  themeId: string,
  customThemes: readonly CustomTheme[],
): ThemeDefinition {
  const themes = listThemes(customThemes)
  return (
    themes.find((theme) => theme.id === themeId) ??
    themes.find((theme) => theme.id === DEFAULT_THEME_PREFERENCE.themeId) ??
    BUILTIN_THEMES[0]
  )
}

export function resolveAppearance(
  appearance: Appearance,
  prefersDark: boolean,
): ResolvedAppearance {
  if (appearance === "system") {
    return prefersDark ? "dark" : "light"
  }
  return appearance
}

export function themeSeed(theme: ThemeDefinition): ThemeSeed {
  return seedFromHexColor(theme.color) ?? DEFAULT_SEED
}

/* Custom properties as a plain style object, so a subtree can render in a theme
   other than the active one. This is how the settings picker previews a theme
   that has not been selected. */
export function themeSeedStyle(theme: ThemeDefinition): Record<string, string> {
  const seed = themeSeed(theme)
  return {
    [THEME_HUE_PROPERTY]: String(seed.hue),
    [THEME_CHROMA_PROPERTY]: String(seed.chroma),
  }
}

export function applyTheme(
  root: HTMLElement,
  theme: ThemeDefinition,
  resolvedAppearance: ResolvedAppearance,
): void {
  const seed = themeSeed(theme)

  root.classList.add(THEME_CLASS)
  root.setAttribute(THEME_ATTRIBUTE, theme.id)
  root.style.setProperty(THEME_HUE_PROPERTY, String(seed.hue))
  root.style.setProperty(THEME_CHROMA_PROPERTY, String(seed.chroma))
  root.classList.toggle("dark", resolvedAppearance === "dark")
  root.style.colorScheme = resolvedAppearance
}
