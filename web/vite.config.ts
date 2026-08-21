import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig, loadEnv } from "vite"

const DEVELOPMENT_SERVICE_URL = "http://127.0.0.1:3000"

function resolveServiceUrl(
  command: "build" | "serve",
  configuredUrl: string | undefined,
  allowInsecureLocalBuild: boolean,
): string {
  const value = configuredUrl?.trim() ||
    (command === "serve" ? DEVELOPMENT_SERVICE_URL : "")

  if (!value) {
    throw new Error(
      "Production builds require VITE_CURIO_SERVICE_URL to be set to the deployed HTTPS Curio service.",
    )
  }

  let url: URL
  try {
    url = new URL(value)
  } catch {
    throw new Error("VITE_CURIO_SERVICE_URL must be a valid absolute URL.")
  }
  if (url.protocol !== "https:" && url.protocol !== "http:") {
    throw new Error("VITE_CURIO_SERVICE_URL must use HTTP or HTTPS.")
  }
  if (command === "build" && url.protocol !== "https:" && !allowInsecureLocalBuild) {
    throw new Error(
      "Production builds require an HTTPS VITE_CURIO_SERVICE_URL. Set CURIO_ALLOW_INSECURE_LOCAL_BUILD=1 only for local debug artifacts.",
    )
  }
  return url.href.replace(/\/$/, "")
}

// https://vite.dev/config/
export default defineConfig(({ command, mode }) => {
  const isDesktop = mode === "desktop"
  const env = loadEnv(mode, process.cwd(), "")
  const host = env.TAURI_DEV_HOST
  const serviceUrl = resolveServiceUrl(
    command,
    env.VITE_CURIO_SERVICE_URL,
    env.CURIO_ALLOW_INSECURE_LOCAL_BUILD === "1",
  )

  return {
    plugins: [react(), tailwindcss()],
    define: {
      __CURIO_SERVICE_URL__: JSON.stringify(serviceUrl),
    },
    resolve: {
      alias: {
        "@": new URL("./src", import.meta.url).pathname,
        "@curio/platform-runtime": isDesktop
          ? new URL("./src/platform/desktop.ts", import.meta.url).pathname
          : new URL("./src/platform/web.ts", import.meta.url).pathname,
        "@curio/router-runtime": isDesktop
          ? new URL("./src/app/router.desktop.tsx", import.meta.url).pathname
          : new URL("./src/app/router.web.tsx", import.meta.url).pathname,
      },
    },
    clearScreen: !isDesktop,
    server: {
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ["**/src-tauri/**"],
      },
    },
  }
})
