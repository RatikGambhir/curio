import { createContext } from "react"

import type { PlatformServices } from "@/platform/contracts"

export const PlatformContext = createContext<PlatformServices | null>(null)
