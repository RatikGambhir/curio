import type { AppTarget } from "@/platform/contracts"

export type RouteAccess = "public" | "anonymous" | "authenticated"
/* "app" routes render inside the persistent shell (navigation spine plus
   content sheet), mounted once so it survives navigation between them;
   "standalone" routes own the whole window. */
export type RouteLayout = "app" | "standalone"
export type RouteId =
  | "root"
  | "login"
  | "verify-email"
  | "profile-setup"
  | "home"
  | "chat"
  | "calendar"
  | "notes"
  | "vault"
  | "atlas"
  | "profile"
  | "settings"

export type AppRoute = {
  id: RouteId
  path: string
  access: RouteAccess
  layout: RouteLayout
  targets: readonly AppTarget[]
}

const allTargets = ["web", "desktop"] as const

export const routeManifest: readonly AppRoute[] = [
  {
    id: "root",
    path: "/",
    access: "public",
    layout: "standalone",
    targets: allTargets,
  },
  {
    id: "login",
    path: "/login",
    access: "anonymous",
    layout: "standalone",
    targets: allTargets,
  },
  {
    id: "verify-email",
    path: "/verify-email",
    access: "public",
    layout: "standalone",
    targets: allTargets,
  },
  {
    id: "profile-setup",
    path: "/profile-setup",
    access: "authenticated",
    layout: "standalone",
    targets: allTargets,
  },
  {
    id: "home",
    path: "/home",
    access: "authenticated",
    layout: "app",
    targets: allTargets,
  },
  {
    id: "chat",
    path: "/chat",
    access: "authenticated",
    layout: "app",
    targets: allTargets,
  },
  {
    id: "calendar",
    path: "/calendar",
    access: "authenticated",
    layout: "app",
    targets: allTargets,
  },
  {
    id: "notes",
    path: "/notes",
    access: "authenticated",
    layout: "app",
    targets: allTargets,
  },
  {
    id: "vault",
    path: "/vault",
    access: "authenticated",
    layout: "app",
    targets: allTargets,
  },
  {
    id: "atlas",
    path: "/atlas",
    access: "authenticated",
    layout: "app",
    targets: allTargets,
  },
  {
    id: "profile",
    path: "/profile",
    access: "authenticated",
    layout: "app",
    targets: allTargets,
  },
  {
    id: "settings",
    path: "/settings",
    access: "authenticated",
    layout: "app",
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
