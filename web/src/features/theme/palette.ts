/* The colour scale.
 *
 * A theme is one seed colour; this is the eleven-step ramp the whole palette is
 * built from. Lightness is pinned per step so text contrast never depends on
 * which colour was seeded, and chroma is carried as a multiple of the seed's
 * chroma so a muted pick stays muted and a vivid one stays vivid.
 *
 * The ladder is not arbitrary. It was fitted to published palettes by averaging
 * their measured OKLCH lightness per step and their chroma relative to each
 * seed. Cool Sky Blue comes back out of it to within about 0.015 in OKLab,
 * under the ~0.02 threshold where a difference becomes visible, so seeding
 * #1b8ae4 reproduces its published table while any other colour gets the same
 * ramp.
 *
 * Sage Green, Cool Steel and Charcoal Blue were published against a slightly
 * different ladder: Sage Green's mid steps run roughly 0.04 lighter, Cool
 * Steel and Charcoal Blue roughly 0.03 darker than this one places them. No
 * single pinned ladder can hold all four, because their step 500 lightnesses
 * span 0.60 to 0.67. This one wins anyway, because pinning
 * lightness is what keeps contrast independent of the seed — taking the
 * published sage-600 (#64824a) literally drops a white button label to 4.34:1,
 * under the 4.5:1 a label needs. palette.test.ts holds each published table
 * alongside the gap it is allowed, so a step drifting past that still fails.
 *
 * index.css holds these same numbers as `--scale-50` ... `--scale-950`, because
 * the palette has to exist before JavaScript runs. palette.test.ts parses the
 * stylesheet and fails if the two ever drift apart.
 */

import { oklchToHex, type Oklch, type ThemeSeed } from "@/features/theme/color"

export const SCALE_STEPS = [
  50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950,
] as const

export type ScaleStep = (typeof SCALE_STEPS)[number]

export type ScaleStop = {
  /* OKLCH lightness, fixed for every theme. */
  lightness: number
  /* Multiple of the seed's chroma. Peaks at step 500 because that step sits at
     a lightness with more chroma headroom than the seed is measured at. */
  chromaMultiplier: number
}

export const SCALE: Record<ScaleStep, ScaleStop> = {
  50: { lightness: 0.9618, chromaMultiplier: 0.114 },
  100: { lightness: 0.9242, chromaMultiplier: 0.244 },
  200: { lightness: 0.8471, chromaMultiplier: 0.468 },
  300: { lightness: 0.7722, chromaMultiplier: 0.721 },
  400: { lightness: 0.6974, chromaMultiplier: 0.952 },
  500: { lightness: 0.6272, chromaMultiplier: 1.181 },
  600: { lightness: 0.5332, chromaMultiplier: 0.982 },
  700: { lightness: 0.4368, chromaMultiplier: 0.787 },
  800: { lightness: 0.3324, chromaMultiplier: 0.557 },
  900: { lightness: 0.2218, chromaMultiplier: 0.332 },
  950: { lightness: 0.1832, chromaMultiplier: 0.237 },
}

/* What each step is for. Mirrors the role mappings in index.css and is shown
   next to the swatches in settings, so the guidance lives with the colours
   rather than in a document nobody opens. Surfaces and text come from the
   fixed paper ladder, so the seed only has to supply ink. */
export const SCALE_USAGE: Record<ScaleStep, string> = {
  50: "Faint tints",
  100: "Highlights and selected marks",
  200: "Dark-mode pressed action",
  300: "Dark-mode action hover",
  400: "Dark-mode action, navigation marker",
  500: "Brand accent",
  600: "Primary action and links",
  700: "Primary hover",
  800: "Primary pressed, dark-mode highlight",
  900: "Deep tints",
  950: "Text on dark-mode actions",
}

export function scaleColor(seed: ThemeSeed, step: ScaleStep): Oklch {
  const { lightness, chromaMultiplier } = SCALE[step]
  return {
    lightness,
    chroma: seed.chroma * chromaMultiplier,
    hue: seed.hue,
  }
}

export function scaleHex(seed: ThemeSeed, step: ScaleStep): string {
  return oklchToHex(scaleColor(seed, step))
}

export function scaleHexes(seed: ThemeSeed): Record<ScaleStep, string> {
  return Object.fromEntries(
    SCALE_STEPS.map((step) => [step, scaleHex(seed, step)]),
  ) as Record<ScaleStep, string>
}
