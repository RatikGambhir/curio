import { lazy, Suspense, type ReactElement } from "react"
import {
  Navigate,
  Route,
  Routes,
  useLocation,
} from "react-router-dom"

import {
  rootDestination,
  routesForTarget,
  type AppRoute,
  type RouteId,
} from "@/app/route-manifest"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"
import type { AppTarget } from "@/platform/contracts"

const Atlas = lazy(() => import("@/pages/Atlas"))
const Chat = lazy(() => import("@/pages/Chat"))
const Home = lazy(() => import("@/pages/Home"))
const Landing = lazy(() => import("@/pages/Landing"))
const Login = lazy(() => import("@/pages/Login"))
const ProfileSetupWizard = lazy(() => import("@/pages/ProfileSetupWizard"))
const ProfileSettings = lazy(() => import("@/pages/ProfileSettings"))
const Vault = lazy(() => import("@/pages/Vault"))

function RequireAuth({ children }: { children: ReactElement }) {
  const { isAuthenticated } = useAuthenticatedUser()
  const location = useLocation()

  if (!isAuthenticated) {
    return <Navigate replace to="/login" state={{ from: location.pathname }} />
  }
  return children
}

function RedirectIfAuthenticated({ children }: { children: ReactElement }) {
  const { isAuthenticated } = useAuthenticatedUser()
  return isAuthenticated ? <Navigate replace to="/home" /> : children
}

function RootRoute({ target }: { target: AppTarget }) {
  const { isAuthenticated } = useAuthenticatedUser()
  const destination = rootDestination(target, isAuthenticated)
  return destination === "landing" ? <Landing /> : <Navigate replace to={destination} />
}

function pageForRoute(id: RouteId, target: AppTarget): ReactElement {
  switch (id) {
    case "root":
      return <RootRoute target={target} />
    case "login":
      return <Login />
    case "verify-email":
      return <Navigate replace to="/login" />
    case "profile-setup":
      return <ProfileSetupWizard />
    case "home":
      return <Home />
    case "chat":
      return <Chat />
    case "vault":
      return <Vault />
    case "atlas":
      return <Atlas />
    case "profile":
    case "settings":
      return <ProfileSettings />
  }
}

function routeElement(route: AppRoute, target: AppTarget): ReactElement {
  const page = pageForRoute(route.id, target)
  if (route.access === "authenticated") {
    return <RequireAuth>{page}</RequireAuth>
  }
  if (route.access === "anonymous") {
    return <RedirectIfAuthenticated>{page}</RedirectIfAuthenticated>
  }
  return page
}

function RouteLoadingState() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background text-sm text-muted-foreground">
      Loading Curio…
    </div>
  )
}

export function AppRoutes({ target }: { target: AppTarget }) {
  const routes = routesForTarget(target)

  return (
    <Suspense fallback={<RouteLoadingState />}>
      <Routes>
        {routes.map((route) => (
          <Route
            key={route.id}
            path={route.path}
            element={routeElement(route, target)}
          />
        ))}
        <Route path="*" element={<Navigate replace to="/" />} />
      </Routes>
    </Suspense>
  )
}
