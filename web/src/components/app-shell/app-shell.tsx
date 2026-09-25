import { Suspense } from "react"
import { Outlet } from "react-router-dom"

import { AppSidebar } from "@/components/app-sidebar"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import { Spinner } from "@/components/ui/spinner"

function PageLoadingState() {
  return (
    <div
      role="status"
      className="flex flex-1 items-center justify-center gap-2 text-sm text-muted-foreground"
    >
      <Spinner aria-hidden="true" />
      <span>Opening page…</span>
    </div>
  )
}

/* The authenticated frame. Mounted once as a layout route, so the spine keeps
   its width, collapsed state and scroll position while pages change inside
   the content sheet. Pages render their own PageHeader first, then content. */
export default function AppShell() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <Suspense fallback={<PageLoadingState />}>
          <Outlet />
        </Suspense>
      </SidebarInset>
    </SidebarProvider>
  )
}
