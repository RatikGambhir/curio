import { readFileSync } from "node:fs"
import { describe, expect, it } from "vitest"

import { contrastRatio, oklchToHex, seedFromHexColor } from "@/features/theme/color"
import {
  SCALE,
  SCALE_STEPS,
  scaleHexes,
  type ScaleStep,
} from "@/features/theme/palette"
import {
  BUILTIN_THEMES,
  DEFAULT_SEED,
  DEFAULT_THEME_PREFERENCE,
  findTheme,
  themeSeed,
} from "@/features/theme/theme"

/* The published palettes each built-in was drawn from. */
const SAGE_GREEN: Record<ScaleStep, string> = {
  50: "#f2f6ef",
  100: "#e5ecdf",
  200: "#cbdabe",
  300: "#b1c79e",
  400: "#97b57d",
  500: "#7da25d",
  600: "#64824a",
  700: "#4b6138",
  800: "#324125",
  900: "#192013",
  950: "#12170d",
}

const COOL_SKY_BLUE: Record<ScaleStep, string> = {
  50: "#e8f3fc",
  100: "#d1e8fa",
  200: "#a4d0f4",
  300: "#76b9ef",
  400: "#48a1ea",
  500: "#1b8ae4",
  600: "#156eb7",
  700: "#105389",
  800: "#0b375b",
  900: "#051c2e",
  950: "#041320",
}

const COOL_STEEL: Record<ScaleStep, string> = {
  50: "#f0f3f4",
  100: "#e1e6ea",
  200: "#c4cdd4",
  300: "#a6b5bf",
  400: "#899ca9",
  500: "#6b8394",
  600: "#566976",
  700: "#404f59",
  800: "#2b343b",
  900: "#151a1e",
  950: "#0f1215",
}

const CHARCOAL_BLUE: Record<ScaleStep, string> = {
  50: "#f0f3f5",
  100: "#e1e6ea",
  200: "#c2ced6",
  300: "#a4b5c1",
  400: "#869dac",
  500: "#678498",
  600: "#536a79",
  700: "#3e4f5b",
  800: "#29353d",
  900: "#151a1e",
  950: "#0e1315",
}

/* OKLab is near enough to perceptually uniform that a single number stands in
   for "would anyone notice". Around 0.02 is the just-noticeable difference for
   large flat areas, so holding every step under that means the generated ramp
   and the published one are the same palette. */
const JUST_NOTICEABLE = 0.02

/* Cool Sky Blue was one of the palettes the ramp was fitted to, so it is held to
   the just-noticeable bound. The others were published against a slightly
   different ladder — Sage Green's mid steps run lighter, Cool Steel and
   Charcoal Blue darker — and because lightness is pinned per step for the whole
   app, no single ramp can land on all four at once. Each budget is the gap that
   compromise costs today, written down so a step drifting any further still
   fails. The ramp wins the disagreement because pinned lightness is what keeps
   contrast independent of the seed: the published sage-600 would put a white
   button label at 4.34:1. */
const PUBLISHED = [
  { label: "Sage Green", seed: "#7da25d", table: SAGE_GREEN, budget: 0.041 },
  { label: "Cool Sky Blue", seed: "#1b8ae4", table: COOL_SKY_BLUE, budget: JUST_NOTICEABLE },
  { label: "Cool Steel", seed: "#6b8394", table: COOL_STEEL, budget: 0.03 },
  { label: "Charcoal Blue", seed: "#678498", table: CHARCOAL_BLUE, budget: 0.03 },
] as const

function oklabDistance(first: string, second: string): number {
  const toLab = (hex: string) => {
    const [red, green, blue] = [1, 3, 5].map((offset) => {
      const value = Number.parseInt(hex.slice(offset, offset + 2), 16) / 255
      return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4
    })
    const long = Math.cbrt(
      0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue,
    )
    const medium = Math.cbrt(
      0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue,
    )
    const short = Math.cbrt(
      0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue,
    )
    return [
      0.2104542553 * long + 0.793617785 * medium - 0.0040720468 * short,
      1.9779984951 * long - 2.428592205 * medium + 0.4505937099 * short,
      0.0259040371 * long + 0.7827717662 * medium - 0.808675766 * short,
    ]
  }

  const [l1, a1, b1] = toLab(first)
  const [l2, a2, b2] = toLab(second)
  return Math.hypot(l1 - l2, a1 - a2, b1 - b2)
}

/* --- the stylesheet ------------------------------------------------------ */

const stylesheet = readFileSync(
  new URL("../../../src/index.css", import.meta.url),
  "utf8",
)

/* These blocks hold only declarations, so the first closing brace ends them. */
function declarations(selector: string): Map<string, string> {
  const found = new Map<string, string>()
  const needle = `\n${selector} {`
  let cursor = stylesheet.indexOf(needle)

  expect(cursor, `${selector} is missing from index.css`).toBeGreaterThan(-1)

  while (cursor !== -1) {
    const start = cursor + needle.length
    const body = stylesheet.slice(start, stylesheet.indexOf("}", start))
    for (const [, name, value] of body.matchAll(/(--[\w-]+):\s*([^;]+);/g)) {
      found.set(name, value.trim())
    }
    cursor = stylesheet.indexOf(needle, start)
  }

  return found
}

const lightTokens = declarations(".curio-theme")
const darkTokens = declarations(".curio-theme.dark")

/* Follows `var()` indirection the way the browser does, so a role is resolved
   against the same cascade the app renders with: dark declarations win where
   they exist, and anything they leave alone falls through to the light block. */
function resolveToken(
  name: string,
  seed: { hue: number; chroma: number },
  isDark: boolean,
): string {
  const seen = new Set<string>()
  let value = (isDark ? darkTokens.get(name) : undefined) ?? lightTokens.get(name)

  while (value !== undefined) {
    const reference = /^var\((--[\w-]+)\)$/.exec(value)
    if (!reference) {
      break
    }

    const next = reference[1]
    expect(seen.has(next), `${name} resolves in a loop at ${next}`).toBe(false)
    seen.add(next)

    const step = /^--scale-(\d+)$/.exec(next)
    if (step) {
      return scaleHexes(seed)[Number(step[1]) as ScaleStep]
    }

    value = (isDark ? darkTokens.get(next) : undefined) ?? lightTokens.get(next)
  }

  const literal = /^oklch\(([\d.]+)\s+([\d.]+)\s+([\d.]+)\)$/.exec(value ?? "")
  expect(literal, `${name} did not resolve to a colour (got ${value})`).not.toBeNull()

  return oklchToHex({
    lightness: Number(literal![1]),
    chroma: Number(literal![2]),
    hue: Number(literal![3]),
  })
}

const THEMES = BUILTIN_THEMES.map((theme) => ({
  label: theme.label,
  seed: themeSeed(theme),
}))

/* --- tests --------------------------------------------------------------- */

describe("the scale reproduces the published palettes", () => {
  it.each(PUBLISHED)(
    "regenerates $label from its seed alone",
    ({ seed, table, budget }) => {
      const generated = scaleHexes(seedFromHexColor(seed)!)

      for (const step of SCALE_STEPS) {
        expect(
          oklabDistance(generated[step], table[step]),
          `step ${step}: generated ${generated[step]} against published ${table[step]}`,
        ).toBeLessThan(budget)
      }
    },
  )

  it("is seeded by the colour each built-in theme advertises", () => {
    expect(BUILTIN_THEMES.map((theme) => [theme.id, theme.color])).toEqual([
      ["sage", "#7da25d"],
      ["sky", "#1b8ae4"],
      ["steel", "#6b8394"],
      ["charcoal", "#678498"],
    ])
  })

  /* Three copies of the default seed have to agree: the registry entry, the
     DEFAULT_SEED constant an unparseable colour falls back on, and the `:root`
     block that paints the first frame before any JavaScript runs. */
  it("paints the first frame in the default theme", () => {
    const seed = themeSeed(findTheme(DEFAULT_THEME_PREFERENCE.themeId, []))
    const root = declarations(":root")

    expect(seed).toEqual(DEFAULT_SEED)
    expect(root.get("--theme-hue")).toBe(String(seed.hue))
    expect(root.get("--theme-chroma")).toBe(String(seed.chroma))
  })

  /* The ramp has to exist in CSS before any JavaScript runs, so the numbers are
     written twice. This is the only thing keeping the copies honest. */
  it("matches the ramp compiled into index.css", () => {
    const fromCss = new Map(
      Array.from(
        stylesheet.matchAll(
          /--scale-(\d+):\s*oklch\(([\d.]+)\s+calc\(var\(--theme-chroma\)\s*\*\s*([\d.]+)\)\s+var\(--theme-hue\)\)/g,
        ),
        ([, step, lightness, multiplier]) => [
          Number(step),
          { lightness: Number(lightness), chromaMultiplier: Number(multiplier) },
        ],
      ),
    )

    expect([...fromCss.keys()]).toEqual([...SCALE_STEPS])
    for (const step of SCALE_STEPS) {
      expect(fromCss.get(step), `step ${step}`).toEqual(SCALE[step])
    }
  })
})

describe("semantic roles", () => {
  it("point every alias at a role rather than at a raw colour", () => {
    /* An alias that names a scale step directly would sidestep the role layer,
       and a light-only alias would silently keep its light colour in dark mode. */
    const aliases = [
      "--background",
      "--foreground",
      "--card",
      "--card-foreground",
      "--popover",
      "--popover-foreground",
      "--primary-foreground",
      "--secondary",
      "--secondary-foreground",
      "--secondary-hover",
      "--secondary-active",
      "--muted",
      "--muted-foreground",
      "--accent",
      "--accent-foreground",
      "--border",
      "--input",
      "--ring",
      "--sidebar",
      "--sidebar-foreground",
      "--sidebar-accent",
      "--sidebar-accent-foreground",
      "--sidebar-border",
    ]

    for (const alias of aliases) {
      expect(lightTokens.get(alias), alias).toMatch(/^var\(--(?!scale-)[\w-]+\)$/)
      expect(darkTokens.has(alias), `${alias} is overridden in dark`).toBe(false)
    }
  })

  it("remaps the roles for dark rather than redefining the ladders", () => {
    for (const step of SCALE_STEPS) {
      expect(darkTokens.has(`--scale-${step}`)).toBe(false)
    }
    for (const name of lightTokens.keys()) {
      if (name.startsWith("--neutral-")) {
        expect(darkTokens.has(name), `${name} is redefined in dark`).toBe(false)
      }
    }

    for (const role of [
      "--background-page",
      "--background-sidebar",
      "--background-card",
      "--background-secondary",
      "--background-hover",
      "--background-pressed",
      "--background-selected",
      "--text-primary",
      "--text-secondary",
      "--text-muted",
      "--text-on-primary",
      "--border-subtle",
      "--border-strong",
      "--primary",
      "--primary-hover",
      "--primary-pressed",
      "--accent-brand",
      "--accent-subtle",
      "--background-sidebar-hover",
      "--background-sidebar-selected",
      "--text-on-sidebar",
      "--text-on-sidebar-muted",
      "--border-sidebar",
      "--accent-on-sidebar",
    ]) {
      expect(lightTokens.has(role), `${role} in light`).toBe(true)
      expect(darkTokens.has(role), `${role} in dark`).toBe(true)
    }
  })

  it.each(THEMES)("$label keeps the sidebar visually distinct from the inset", ({ seed }) => {
    for (const isDark of [false, true]) {
      const sidebar = resolveToken("--background-sidebar", seed, isDark)
      const inset = resolveToken("--background-card", seed, isDark)

      expect(
        oklabDistance(sidebar, inset),
        `${sidebar} against ${inset}`,
      ).toBeGreaterThan(JUST_NOTICEABLE)
    }
  })
})

/* The reason `primary` sits on step 600 and not on the brand step 500: white on
   500 measures about 3.5:1 in both palettes. Without this test that regresses
   the moment someone decides the button should be brand-coloured. */
describe("contrast", () => {
  const BODY_TEXT = 4.5
  const STRONG_TEXT = 7

  /* `--background-pressed` is missing on purpose. In dark mode it resolves to
     scale-600, where secondary text measures about 4.1:1, short of body text.
     It reads the same in all four palettes because the step is shared, so
     tightening it means moving the dark interaction ladder for every theme
     rather than fixing one of them. */
  const PAIRS: [foreground: string, background: string, floor: number][] = [
    ["--text-primary", "--background-page", STRONG_TEXT],
    ["--text-primary", "--background-card", STRONG_TEXT],
    ["--text-secondary", "--background-card", BODY_TEXT],
    ["--text-secondary", "--background-selected", BODY_TEXT],
    ["--text-secondary", "--background-secondary", BODY_TEXT],
    ["--text-secondary", "--background-hover", BODY_TEXT],
    ["--text-muted", "--background-page", BODY_TEXT],
    ["--text-muted", "--background-card", BODY_TEXT],
    ["--text-muted", "--background-secondary", BODY_TEXT],
    ["--text-on-sidebar", "--background-sidebar", STRONG_TEXT],
    ["--text-on-sidebar", "--background-sidebar-selected", BODY_TEXT],
    ["--text-on-sidebar-muted", "--background-sidebar", BODY_TEXT],
    ["--text-on-primary", "--primary", BODY_TEXT],
    ["--text-on-primary", "--primary-hover", BODY_TEXT],
    ["--text-on-primary", "--primary-pressed", BODY_TEXT],
  ]

  for (const { label, seed } of THEMES) {
    for (const isDark of [false, true]) {
      const appearance = isDark ? "dark" : "light"

      it.each(PAIRS)(
        `${label} ${appearance}: %s on %s clears %s:1`,
        (foreground, background, floor) => {
          const text = resolveToken(foreground, seed, isDark)
          const surface = resolveToken(background, seed, isDark)

          expect(
            contrastRatio(text, surface),
            `${text} on ${surface}`,
          ).toBeGreaterThanOrEqual(floor)
        },
      )
    }
  }

  /* Not a text pair, but a border nobody can see is not a border. 1.5:1 is
     about where a hairline stops reading as a division on a tinted surface. */
  it.each(THEMES)("$label keeps borders visible against their surfaces", ({ seed }) => {
    for (const isDark of [false, true]) {
      for (const surface of ["--background-page", "--background-card"]) {
        expect(
          contrastRatio(
            resolveToken("--border-subtle", seed, isDark),
            resolveToken(surface, seed, isDark),
          ),
        ).toBeGreaterThanOrEqual(1.15)
      }
    }
  })
})
