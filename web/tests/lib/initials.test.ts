import { describe, expect, it } from "vitest"

import { initialsFor } from "@/lib/initials"

describe("avatar initials", () => {
  it("takes the first letter of the first two words, uppercased", () => {
    expect(initialsFor("ada lovelace")).toBe("AL")
    expect(initialsFor("Grace Brewster Murray Hopper")).toBe("GB")
    expect(initialsFor("Curio")).toBe("C")
  })

  it("ignores repeated and surrounding whitespace", () => {
    expect(initialsFor("  ada \t  lovelace\n")).toBe("AL")
  })

  it("falls back when there is no name to abbreviate", () => {
    expect(initialsFor(undefined)).toBe("CU")
    expect(initialsFor("")).toBe("CU")
    expect(initialsFor("   ")).toBe("CU")
    expect(initialsFor("", "?")).toBe("?")
  })
})
