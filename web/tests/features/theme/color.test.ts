import { describe, expect, it } from "vitest"

import {
  hexToOklch,
  maxDisplayableChroma,
  normalizeHexColor,
  seedFromHexColor,
} from "@/features/theme/color"

describe("normalizeHexColor", () => {
  it("accepts three- and six-digit hex with or without a hash", () => {
    expect(normalizeHexColor("#ABC")).toBe("#aabbcc")
    expect(normalizeHexColor("abc")).toBe("#aabbcc")
    expect(normalizeHexColor("  #2D432D  ")).toBe("#2d432d")
    expect(normalizeHexColor("156eb7")).toBe("#156eb7")
  })

  it("rejects anything that is not a hex colour", () => {
    for (const value of ["", "#", "#ff", "#ffff", "#fffffff", "rebeccapurple", "#12345g"]) {
      expect(normalizeHexColor(value)).toBeNull()
    }
  })
})

describe("hexToOklch", () => {
  it("converts the palette seeds to their known coordinates", () => {
    const sage = hexToOklch("#7da25d")
    expect(sage?.lightness).toBeCloseTo(0.6679, 3)
    expect(sage?.chroma).toBeCloseTo(0.1041, 3)
    expect(sage?.hue).toBeCloseTo(131.69, 1)

    const sky = hexToOklch("#1b8ae4")
    expect(sky?.lightness).toBeCloseTo(0.6213, 3)
    expect(sky?.chroma).toBeCloseTo(0.1640, 3)
    expect(sky?.hue).toBeCloseTo(249.55, 1)

    const steel = hexToOklch("#6b8394")
    expect(steel?.lightness).toBeCloseTo(0.5972, 3)
    expect(steel?.chroma).toBeCloseTo(0.0380, 3)
    expect(steel?.hue).toBeCloseTo(239.12, 1)

    const charcoal = hexToOklch("#678498")
    expect(charcoal?.lightness).toBeCloseTo(0.5983, 3)
    expect(charcoal?.chroma).toBeCloseTo(0.0451, 3)
    expect(charcoal?.hue).toBeCloseTo(238.62, 1)
  })

  it("reports greys as having no chroma", () => {
    expect(hexToOklch("#808080")?.chroma).toBeCloseTo(0, 4)
  })

  it("returns null for unparseable input", () => {
    expect(hexToOklch("nope")).toBeNull()
  })
})

describe("maxDisplayableChroma", () => {
  it("leaves room for colour at mid lightness and almost none at black or white", () => {
    const mid = maxDisplayableChroma(0.53, 145)
    expect(mid).toBeGreaterThan(0.1)

    expect(maxDisplayableChroma(0, 145)).toBeLessThan(0.03)
    expect(maxDisplayableChroma(1, 145)).toBeLessThan(0.03)
  })

  it("rises and falls with lightness rather than jumping around", () => {
    const samples = [0.1, 0.3, 0.5, 0.7, 0.9].map((lightness) =>
      maxDisplayableChroma(lightness, 250),
    )

    expect(samples[0]).toBeLessThan(samples[2])
    expect(samples[4]).toBeLessThan(samples[2])
  })
})

describe("seedFromHexColor", () => {
  it("reproduces the seeds the built-in palettes were sampled from", () => {
    expect(seedFromHexColor("#7da25d")).toMatchObject({
      hue: expect.closeTo(131.69, 1),
      chroma: expect.closeTo(0.0828, 3),
    })
    expect(seedFromHexColor("#1b8ae4")).toMatchObject({
      hue: expect.closeTo(249.55, 1),
      chroma: expect.closeTo(0.1402, 3),
    })
    /* Cool Steel is the near-neutral end of the range the seed maths has to
       cover: a tenth of Cool Sky's chroma, but still a colour rather than grey. */
    expect(seedFromHexColor("#6b8394")).toMatchObject({
      hue: expect.closeTo(239.12, 1),
      chroma: expect.closeTo(0.0338, 3),
    })
    expect(seedFromHexColor("#678498")).toMatchObject({
      hue: expect.closeTo(238.62, 1),
      chroma: expect.closeTo(0.04, 3),
    })
  })

  /* The whole point of scaling by available chroma: a dark green and a mid
     green of the same vividness must produce the same theme, because lightness
     is fixed by the palette and only hue and vividness come from the seed. */
  it("maps equally vivid colours to the same seed regardless of lightness", () => {
    const dark = seedFromHexColor("#2d432d")!
    const mid = seedFromHexColor("#507952")!

    expect(Math.abs(dark.hue - mid.hue)).toBeLessThan(2)
    expect(Math.abs(dark.chroma - mid.chroma)).toBeLessThan(0.01)
  })

  it("keeps a near-grey pick near-grey", () => {
    expect(seedFromHexColor("#8a8f8a")?.chroma).toBeLessThan(0.02)
  })

  it("turns achromatic picks into a grey theme rather than dividing by zero", () => {
    for (const color of ["#000000", "#ffffff", "#808080"]) {
      expect(seedFromHexColor(color)?.chroma).toBe(0)
    }
  })

  it("never exceeds what the reference lightness can display", () => {
    for (const color of ["#e11d48", "#7c3aed", "#f59e0b", "#00ff00"]) {
      const seed = seedFromHexColor(color)!
      expect(seed.chroma).toBeLessThanOrEqual(
        maxDisplayableChroma(0.531, seed.hue) + 1e-4,
      )
    }
  })

  it("returns null for unparseable input", () => {
    expect(seedFromHexColor("#nope")).toBeNull()
  })
})
