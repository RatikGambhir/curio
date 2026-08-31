import { describe, expect, it } from "vitest"

import {
  BUILTIN_THEMES,
  DEFAULT_THEME_PREFERENCE,
  findTheme,
  listThemes,
  MAX_CUSTOM_THEMES,
  normalizeThemeLabel,
  parseThemePreference,
  readThemePreference,
  resolveAppearance,
  THEME_STORAGE_KEY,
  themeSeedStyle,
  writeThemePreference,
  type CustomTheme,
} from "@/features/theme/theme"

function memoryStorage(values: Record<string, string> = {}): Storage {
  const entries = new Map(Object.entries(values))
  return {
    get length() {
      return entries.size
    },
    clear: () => entries.clear(),
    getItem: (key) => entries.get(key) ?? null,
    key: (index) => Array.from(entries.keys())[index] ?? null,
    removeItem: (key) => {
      entries.delete(key)
    },
    setItem: (key, value) => {
      entries.set(key, value)
    },
  }
}

const custom: CustomTheme = {
  id: "custom-1",
  label: "Ink",
  color: "#2d432d",
}

describe("parseThemePreference", () => {
  it("falls back to the default when there is nothing stored or it is corrupt", () => {
    expect(parseThemePreference(null)).toEqual(DEFAULT_THEME_PREFERENCE)
    expect(parseThemePreference("not-json")).toEqual(DEFAULT_THEME_PREFERENCE)
    expect(parseThemePreference("[]")).toEqual(DEFAULT_THEME_PREFERENCE)
  })

  /* A preference written by an older build should not lose the fields it did
     get right, so each one falls back independently. */
  it("recovers field by field", () => {
    expect(
      parseThemePreference('{"themeId":"sky","appearance":"ultraviolet"}'),
    ).toEqual({
      themeId: "sky",
      appearance: DEFAULT_THEME_PREFERENCE.appearance,
      customThemes: [],
    })

    expect(parseThemePreference('{"appearance":"dark"}')).toEqual({
      themeId: DEFAULT_THEME_PREFERENCE.themeId,
      appearance: "dark",
      customThemes: [],
    })
  })

  /* Sage Green replaced Dark Spruce. An unknown id already falls back to the
     default, so without this rewrite a stored "spruce" would now land on Cool
     Steel and look like the choice was forgotten. The stored id has to become
     "sage" too, or the next write persists a theme that no longer exists and a
     later build could give the name to something else. */
  it("migrates a retired built-in id to its successor", () => {
    expect(
      parseThemePreference('{"themeId":"spruce","appearance":"dark"}'),
    ).toEqual({
      themeId: "sage",
      appearance: "dark",
      customThemes: [],
    })
  })

  /* Prototype keys are not migrations: `{}["constructor"]` is a function, and
     assigning that as a theme id would break every lookup downstream. */
  it("does not treat inherited object keys as renamed themes", () => {
    expect(parseThemePreference('{"themeId":"constructor"}').themeId).toBe(
      "constructor",
    )
  })

  it("keeps well-formed custom themes and drops broken ones", () => {
    const parsed = parseThemePreference(
      JSON.stringify({
        themeId: "custom-1",
        appearance: "light",
        customThemes: [
          custom,
          { id: "custom-2", color: "not-a-colour" },
          { id: "", color: "#ffffff" },
          { color: "#ffffff" },
          "nonsense",
          null,
        ],
      }),
    )

    expect(parsed.customThemes).toEqual([custom])
  })

  it("normalizes stored colours and defaults a missing name to the hex", () => {
    const parsed = parseThemePreference(
      JSON.stringify({
        customThemes: [{ id: "custom-3", color: "ABC" }],
      }),
    )

    expect(parsed.customThemes[0]).toEqual({
      id: "custom-3",
      label: "#AABBCC",
      color: "#aabbcc",
    })
  })

  it("caps how many custom themes it will load", () => {
    const parsed = parseThemePreference(
      JSON.stringify({
        customThemes: Array.from({ length: MAX_CUSTOM_THEMES + 5 }, (_, index) => ({
          id: `custom-${index}`,
          color: "#123456",
        })),
      }),
    )

    expect(parsed.customThemes).toHaveLength(MAX_CUSTOM_THEMES)
  })

  it("drops custom ids that would collide in the picker", () => {
    const parsed = parseThemePreference(
      JSON.stringify({
        customThemes: [
          { id: "sky", color: "#123456" },
          { id: " custom-1 ", label: "First", color: "#123456" },
          { id: "custom-1", label: "Duplicate", color: "#654321" },
        ],
      }),
    )

    expect(parsed.customThemes).toEqual([
      { id: "custom-1", label: "First", color: "#123456" },
    ])
  })
})

describe("theme preference storage", () => {
  it("round-trips through storage", () => {
    const storage = memoryStorage()
    const preference = {
      themeId: "custom-1",
      appearance: "dark" as const,
      customThemes: [custom],
    }

    writeThemePreference(storage, preference)
    expect(readThemePreference(storage)).toEqual(preference)
  })

  it("survives storage that refuses to co-operate", () => {
    const throwing = {
      ...memoryStorage(),
      getItem: () => {
        throw new Error("denied")
      },
      setItem: () => {
        throw new Error("quota")
      },
    } as unknown as Storage

    expect(() =>
      writeThemePreference(throwing, DEFAULT_THEME_PREFERENCE),
    ).not.toThrow()
    expect(readThemePreference(throwing)).toEqual(DEFAULT_THEME_PREFERENCE)
  })

  it("ignores a preference stored under a different key", () => {
    const storage = memoryStorage({ "some-other-key": '{"themeId":"sky"}' })
    expect(readThemePreference(storage).themeId).toBe(
      DEFAULT_THEME_PREFERENCE.themeId,
    )
    expect(storage.getItem(THEME_STORAGE_KEY)).toBeNull()
  })
})

describe("listThemes", () => {
  it("puts the built-ins before anything the user added", () => {
    const themes = listThemes([custom])

    expect(themes).toHaveLength(BUILTIN_THEMES.length + 1)
    expect(themes.slice(0, BUILTIN_THEMES.length).map((theme) => theme.id)).toEqual(
      BUILTIN_THEMES.map((theme) => theme.id),
    )
    expect(themes.at(-1)).toMatchObject({
      id: custom.id,
      label: custom.label,
      origin: "custom",
    })
  })
})

describe("findTheme", () => {
  it("finds built-in and custom themes by id", () => {
    expect(findTheme("sky", []).label).toBe("Cool Sky Blue")
    expect(findTheme("charcoal", []).label).toBe("Charcoal Blue")
    expect(findTheme(custom.id, [custom]).label).toBe(custom.label)
  })

  /* Deleting a custom theme, or loading a preference from a build that had
     more themes, must not leave the app without a palette. */
  it("falls back to the default for an unknown id", () => {
    expect(findTheme("deleted", []).id).toBe(DEFAULT_THEME_PREFERENCE.themeId)
    expect(findTheme("", [custom]).id).toBe(DEFAULT_THEME_PREFERENCE.themeId)
  })
})

describe("resolveAppearance", () => {
  it("follows the system only when asked to", () => {
    expect(resolveAppearance("system", true)).toBe("dark")
    expect(resolveAppearance("system", false)).toBe("light")
    expect(resolveAppearance("light", true)).toBe("light")
    expect(resolveAppearance("dark", false)).toBe("dark")
  })
})

describe("themeSeedStyle", () => {
  it("emits the two custom properties the palette reads", () => {
    expect(themeSeedStyle(findTheme("sky", []))).toEqual({
      "--theme-hue": "249.55",
      "--theme-chroma": "0.1402",
    })
  })
})

describe("normalizeThemeLabel", () => {
  it("trims, truncates and falls back to the colour", () => {
    expect(normalizeThemeLabel("  Ink  ", "#2d432d")).toBe("Ink")
    expect(normalizeThemeLabel("", "#2d432d")).toBe("#2D432D")
    expect(normalizeThemeLabel(undefined, "#2d432d")).toBe("#2D432D")
    expect(normalizeThemeLabel("x".repeat(80), "#2d432d")).toHaveLength(32)
  })
})
