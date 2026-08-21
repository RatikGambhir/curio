import { BrowserRouter } from "react-router-dom"

import { AppRoutes } from "@/app/router"

export function AppRouter() {
  return (
    <BrowserRouter>
      <AppRoutes target="web" />
    </BrowserRouter>
  )
}
