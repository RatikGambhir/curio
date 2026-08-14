import type { AppTarget } from "@/platform/contracts"

export type RouteAccess = "public" | "anonymous" | "authenticated"
export type RouteId =
  | "root"
  | "login"
  | "verify-email"
  | "profile-setup"
  | "home"
  | "chat"
  | "vault"
  | "atlas"
  | "profile"
  | "settings"

export type AppRoute = {
  id: RouteId
  path: string
  access: RouteAccess
  targets: readonly AppTarget[]
}

const allTargets = ["web", "desktop"] as const

export const routeManifest: readonly AppRoute[] = [
  { id: "root", path: "/", access: "public", targets: allTargets },
  { id: "login", path: "/login", access: "anonymous", targets: allTargets },
  {
    id: "verify-email",
    path: "/verify-email",
    access: "public",
    targets: allTargets,
  },
  {
    id: "profile-setup",
    path: "/profile-setup",
    access: "authenticated",
    targets: allTargets,
  },
  { id: "home", path: "/home", access: "authenticated", targets: allTargets },
  { id: "chat", path: "/chat", access: "authenticated", targets: allTargets },
  { id: "vault", path: "/vault", access: "authenticated", targets: allTargets },
  { id: "atlas", path: "/atlas", access: "authenticated", targets: allTargets },
  {
    id: "profile",
    path: "/profile",
    access: "authenticated",
    targets: allTargets,
  },
  {
    id: "settings",
    path: "/settings",
    access: "authenticated",
    targets: allTargets,
  },
]

export function routesForTarget(target: AppTarget): readonly AppRoute[] {
  return routeManifest.filter((route) => route.targets.includes(target))
}

export function rootDestination(
  target: AppTarget,
  isAuthenticated: boolean,
): "landing" | "/login" | "/home" {
  if (target === "web") {
    return "landing"
  }
  return isAuthenticated ? "/home" : "/login"
}
