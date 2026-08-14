import path from "path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig, loadEnv } from "vite"

const DEVELOPMENT_CHAT_WORKER_URL = "http://127.0.0.1:8787"

function resolveChatWorkerUrl(
  command: "build" | "serve",
  configuredUrl: string | undefined,
  allowInsecureLocalBuild: boolean,
): string {
  const value = configuredUrl?.trim() ||
    (command === "serve" ? DEVELOPMENT_CHAT_WORKER_URL : "")

  if (!value) {
    throw new Error(
      "Production builds require VITE_CURIO_CHAT_WORKER_URL to be set to the deployed HTTPS chat worker.",
    )
  }

  let url: URL
  try {
    url = new URL(value)
  } catch {
    throw new Error("VITE_CURIO_CHAT_WORKER_URL must be a valid absolute URL.")
  }
  if (url.protocol !== "https:" && url.protocol !== "http:") {
    throw new Error("VITE_CURIO_CHAT_WORKER_URL must use HTTP or HTTPS.")
  }
  if (command === "build" && url.protocol !== "https:" && !allowInsecureLocalBuild) {
    throw new Error(
      "Production builds require an HTTPS VITE_CURIO_CHAT_WORKER_URL. Set CURIO_ALLOW_INSECURE_LOCAL_BUILD=1 only for local debug artifacts.",
    )
  }
  return url.href.replace(/\/$/, "")
}

// https://vite.dev/config/
export default defineConfig(({ command, mode }) => {
  const isDesktop = mode === "desktop"
  const env = loadEnv(mode, process.cwd(), "")
  const host = env.TAURI_DEV_HOST
  const chatWorkerUrl = resolveChatWorkerUrl(
    command,
    env.VITE_CURIO_CHAT_WORKER_URL,
    env.CURIO_ALLOW_INSECURE_LOCAL_BUILD === "1",
  )

  return {
    plugins: [react(), tailwindcss()],
    define: {
      __CURIO_CHAT_WORKER_URL__: JSON.stringify(chatWorkerUrl),
    },
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
        "@curio/platform-runtime": path.resolve(
          __dirname,
          isDesktop ? "./src/platform/desktop.ts" : "./src/platform/web.ts",
        ),
      },
    },
    clearScreen: !isDesktop,
    server: isDesktop
      ? {
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
        }
      : undefined,
  }
})
