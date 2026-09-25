import { describe, expect, it } from "vitest"

import {
  rootDestination,
  routeManifest,
  routesForTarget,
} from "@/app/route-manifest"

describe("shared route manifest", () => {
  it("exposes the same authenticated product routes to both targets", () => {
    const authenticatedPaths = routeManifest
      .filter((route) => route.access === "authenticated")
      .map((route) => route.path)

    expect(routesForTarget("web").map((route) => route.path)).toEqual(
      expect.arrayContaining(authenticatedPaths),
    )
    expect(routesForTarget("desktop").map((route) => route.path)).toEqual(
      expect.arrayContaining(authenticatedPaths),
    )
    expect(routeManifest.some((route) => route.path === "/desktop-construction")).toBe(
      false,
    )
  })

  it("keeps every authenticated product page inside the persistent shell", () => {
    const shellRoutes = routeManifest
      .filter((route) => route.layout === "app")
      .map((route) => route.id)

    expect(shellRoutes).toEqual([
      "home",
      "chat",
      "calendar",
      "notes",
      "vault",
      "atlas",
      "profile",
      "settings",
    ])
    expect(
      routeManifest.every(
        (route) => route.layout === "standalone" || route.access === "authenticated",
      ),
    ).toBe(true)
  })

  it("uses a landing root on web and an auth-aware app root on desktop", () => {
    expect(rootDestination("web", false)).toBe("landing")
    expect(rootDestination("desktop", false)).toBe("/login")
    expect(rootDestination("desktop", true)).toBe("/home")
  })
})
