/* Colour maths for the theme seed.
 *
 * A theme is described by one colour. The palette in index.css spends that
 * colour as an OKLCH hue plus a chroma, holding lightness fixed per token so
 * text contrast does not depend on which colour was picked.
 *
 * Taking the seed's chroma literally would punish dark or pale picks: OKLCH
 * chroma is bounded by lightness, so `#2d432d` measures far less chroma than
 * an equally vivid mid-tone green. Instead we measure how much of the chroma
 * available at the seed's own lightness it actually uses, then spend that same
 * fraction of what is available at the palette's reference lightness. Picking a
 * dark green and a mid green therefore lands on the same theme.
 */

/* Matches --primary in the light palette: the token whose vividness a reader
   judges the theme by. */
const REFERENCE_LIGHTNESS = 0.531

/* Below this, a chroma is indistinguishable from floating-point residue. Both
   the seed's chroma and the headroom at its lightness are checked against it,
   because near-black and near-white leave so little headroom that dividing by
   it turns rounding error into a vivid hue — pure white would otherwise seed a
   yellow theme. */
const CHROMA_EPSILON = 1e-4

export type Oklch = {
  lightness: number
  chroma: number
  hue: number
}

export type ThemeSeed = {
  hue: number
  chroma: number
}

export function normalizeHexColor(input: string): string | null {
  const value = input.trim().replace(/^#/, "").toLowerCase()

  if (/^[0-9a-f]{3}$/.test(value)) {
    return `#${value.replace(/./g, (digit) => digit + digit)}`
  }
  if (/^[0-9a-f]{6}$/.test(value)) {
    return `#${value}`
  }
  return null
}

function srgbToLinear(channel: number): number {
  const value = channel / 255
  return value <= 0.04045
    ? value / 12.92
    : ((value + 0.055) / 1.055) ** 2.4
}

function linearToSrgb(value: number): number {
  const channel =
    value <= 0.0031308 ? value * 12.92 : 1.055 * value ** (1 / 2.4) - 0.055
  return Math.round(Math.min(1, Math.max(0, channel)) * 255)
}

function linearSrgbToOklch(red: number, green: number, blue: number): Oklch {
  const long = Math.cbrt(
    0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue,
  )
  const medium = Math.cbrt(
    0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue,
  )
  const short = Math.cbrt(
    0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue,
  )

  const lightness =
    0.2104542553 * long + 0.793617785 * medium - 0.0040720468 * short
  const a = 1.9779984951 * long - 2.428592205 * medium + 0.4505937099 * short
  const b = 0.0259040371 * long + 0.7827717662 * medium - 0.808675766 * short

  return {
    lightness,
    chroma: Math.hypot(a, b),
    hue: ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360,
  }
}

function oklchToLinearSrgb({ lightness, chroma, hue }: Oklch): [number, number, number] {
  const radians = (hue * Math.PI) / 180
  const a = chroma * Math.cos(radians)
  const b = chroma * Math.sin(radians)

  const long = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3
  const medium = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3
  const short = (lightness - 0.0894841775 * a - 1.291485548 * b) ** 3

  return [
    4.0767416621 * long - 3.3077115913 * medium + 0.2309699292 * short,
    -1.2684380046 * long + 2.6097574011 * medium - 0.3413193965 * short,
    -0.0041960863 * long - 0.7034186147 * medium + 1.707614701 * short,
  ]
}

function isDisplayable(lightness: number, chroma: number, hue: number): boolean {
  /* Tight, because these are linear-light values: every channel scales with
     chroma cubed, so a tolerance loose enough to absorb rounding at mid
     lightness would report near-black as holding real colour. */
  const tolerance = 1e-6
  return oklchToLinearSrgb({ lightness, chroma, hue }).every(
    (channel) => channel >= -tolerance && channel <= 1 + tolerance,
  )
}

/* Clamps out-of-gamut channels, matching what a browser does when it paints an
   `oklch()` value it cannot display. Only used to check the palette in tests
   and to render swatches; the app itself lets CSS do the conversion. */
export function oklchToHex(color: Oklch): string {
  return `#${oklchToLinearSrgb(color)
    .map((channel) => linearToSrgb(channel).toString(16).padStart(2, "0"))
    .join("")}`
}

/* WCAG 2.1 relative luminance, which is why this uses sRGB coefficients rather
   than OKLCH lightness: contrast ratios are defined against the former. */
export function relativeLuminance(hex: string): number | null {
  const normalized = normalizeHexColor(hex)
  if (!normalized) {
    return null
  }

  const [red, green, blue] = [1, 3, 5].map((offset) =>
    srgbToLinear(Number.parseInt(normalized.slice(offset, offset + 2), 16)),
  )
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue
}

export function contrastRatio(foreground: string, background: string): number | null {
  const first = relativeLuminance(foreground)
  const second = relativeLuminance(background)
  if (first === null || second === null) {
    return null
  }

  const lighter = Math.max(first, second)
  const darker = Math.min(first, second)
  return (lighter + 0.05) / (darker + 0.05)
}

/* Largest chroma that still fits in sRGB at this lightness and hue. Bisection
   is valid because the gamut is convex along a chroma ray from the neutral
   axis: once a ray leaves the gamut it does not re-enter. */
export function maxDisplayableChroma(lightness: number, hue: number): number {
  let low = 0
  let high = 0.5

  for (let step = 0; step < 24; step += 1) {
    const mid = (low + high) / 2
    if (isDisplayable(lightness, mid, hue)) {
      low = mid
    } else {
      high = mid
    }
  }

  return low
}

export function hexToOklch(hex: string): Oklch | null {
  const normalized = normalizeHexColor(hex)
  if (!normalized) {
    return null
  }

  const [red, green, blue] = [1, 3, 5].map((offset) =>
    srgbToLinear(Number.parseInt(normalized.slice(offset, offset + 2), 16)),
  )

  return linearSrgbToOklch(red, green, blue)
}

export function seedFromHexColor(hex: string): ThemeSeed | null {
  const seedColor = hexToOklch(hex)
  if (!seedColor) {
    return null
  }

  const availableAtSeed = maxDisplayableChroma(
    seedColor.lightness,
    seedColor.hue,
  )

  const isAchromatic =
    seedColor.chroma < CHROMA_EPSILON || availableAtSeed < CHROMA_EPSILON
  const vividness = isAchromatic
    ? 0
    : Math.min(seedColor.chroma / availableAtSeed, 1)

  const chroma =
    vividness * maxDisplayableChroma(REFERENCE_LIGHTNESS, seedColor.hue)

  return {
    hue: Number(seedColor.hue.toFixed(2)),
    chroma: Number(chroma.toFixed(4)),
  }
}
