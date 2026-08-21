import { HashRouter } from "react-router-dom"

import { AppRoutes } from "@/app/router"

export function AppRouter() {
  return (
    <HashRouter>
      <AppRoutes target="desktop" />
    </HashRouter>
  )
}
