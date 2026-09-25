import { lazy, Suspense, type ReactElement } from "react"
import {
  Navigate,
  Route,
  Routes,
  useLocation,
} from "react-router-dom"

import { CurioMark } from "@/components/brand/curio-mark"

import {
  rootDestination,
  routesForTarget,
  type AppRoute,
  type RouteId,
} from "@/app/route-manifest"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"
import type { AppTarget } from "@/platform/contracts"

const AppShell = lazy(() => import("@/components/app-shell/app-shell"))
const Atlas = lazy(() => import("@/pages/Atlas"))
const Calendar = lazy(() => import("@/pages/Calendar"))
const Chat = lazy(() => import("@/pages/Chat"))
const Home = lazy(() => import("@/pages/Home"))
const Landing = lazy(() => import("@/pages/Landing"))
const Login = lazy(() => import("@/pages/Login"))
const Notes = lazy(() => import("@/pages/Notes"))
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
    case "calendar":
      return <Calendar />
    case "notes":
      return <Notes />
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
    <div
      role="status"
      className="flex min-h-svh flex-col items-center justify-center gap-3 bg-background text-muted-foreground"
    >
      <CurioMark className="size-7 animate-pulse text-foreground" />
      <span className="eyebrow">Loading Curio</span>
    </div>
  )
}

export function AppRoutes({ target }: { target: AppTarget }) {
  const routes = routesForTarget(target)
  const standaloneRoutes = routes.filter((route) => route.layout === "standalone")
  const shellRoutes = routes.filter((route) => route.layout === "app")

  return (
    <Suspense fallback={<RouteLoadingState />}>
      <Routes>
        {standaloneRoutes.map((route) => (
          <Route
            key={route.id}
            path={route.path}
            element={routeElement(route, target)}
          />
        ))}
        {/* One shell for every product page: the spine, its collapsed state
            and the content sheet persist across navigation, and only the page
            inside the sheet suspends while its chunk loads. */}
        <Route
          element={
            <RequireAuth>
              <AppShell />
            </RequireAuth>
          }
        >
          {shellRoutes.map((route) => (
            <Route
              key={route.id}
              path={route.path}
              element={routeElement(route, target)}
            />
          ))}
        </Route>
        <Route path="*" element={<Navigate replace to="/" />} />
      </Routes>
    </Suspense>
  )
}
