import { useContext } from "react"

import { PlatformContext } from "@/platform/context"
import type { PlatformServices } from "@/platform/contracts"

export function usePlatform(): PlatformServices {
  const services = useContext(PlatformContext)
  if (!services) {
    throw new Error("usePlatform must be used inside a PlatformProvider.")
  }
  return services
}
