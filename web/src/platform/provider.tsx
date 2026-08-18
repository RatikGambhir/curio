import type { ReactNode } from "react"

import { PlatformContext } from "@/platform/context"
import type { PlatformServices } from "@/platform/contracts"

export function PlatformProvider({
  children,
  services,
}: {
  children: ReactNode
  services: PlatformServices
}) {
  return (
    <PlatformContext.Provider value={services}>
      {children}
    </PlatformContext.Provider>
  )
}
