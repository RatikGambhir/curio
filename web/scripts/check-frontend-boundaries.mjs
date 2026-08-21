import { existsSync, readdirSync, readFileSync, statSync } from "node:fs"
import { resolve, sep } from "node:path"

const appRoot = process.cwd()
const legacyReactRoots = [
  resolve(appRoot, "../desktop/Curio/src"),
  resolve(appRoot, "../desktop"),
]
for (const legacyReactRoot of legacyReactRoots) {
  if (existsSync(legacyReactRoot)) {
    throw new Error(`Legacy desktop source tree found at ${legacyReactRoot}`)
  }
}

const sourceRoot = resolve(appRoot, "src")
const allowedTauriModule = resolve(sourceRoot, "platform/desktop.ts")
const allowedNetworkModule = resolve(sourceRoot, "platform/web.ts")
const platformRuntimeConsumerRoots = [
  resolve(sourceRoot, "api") + sep,
  resolve(sourceRoot, "features/chat") + sep,
]

function sourceFiles(directory) {
  return readdirSync(directory).flatMap((entry) => {
    const path = resolve(directory, entry)
    if (statSync(path).isDirectory()) {
      return sourceFiles(path)
    }
    return /\.(ts|tsx)$/.test(path) ? [path] : []
  })
}

const files = sourceFiles(sourceRoot)
const tauriViolations = files.filter((file) => {
  if (file === allowedTauriModule) {
    return false
  }
  return /(?:from\s+|import\s*)["']@tauri-apps\//.test(
    readFileSync(file, "utf8"),
  )
})

if (tauriViolations.length > 0) {
  throw new Error(
    `Tauri imports escaped the desktop adapter:\n${tauriViolations.join("\n")}`,
  )
}

if (/@tauri-apps\/plugin-/.test(readFileSync(allowedTauriModule, "utf8"))) {
  throw new Error(
    "The desktop adapter must use app-owned Tauri commands/events instead of invoking frontend Tauri plugins.",
  )
}

const networkPrimitive =
  /(?:\bfetch\s*\(|\bXMLHttpRequest\b|\bWebSocket\b|\bEventSource\b|\bsendBeacon\s*\()/
const networkViolations = files.filter((file) => {
  if (file === allowedNetworkModule || /\.test\.(?:ts|tsx)$/.test(file)) {
    return false
  }
  return networkPrimitive.test(readFileSync(file, "utf8"))
})

if (networkViolations.length > 0) {
  throw new Error(
    `Browser network APIs escaped the web platform adapter:\n${networkViolations.join("\n")}`,
  )
}

const concretePlatformImport =
  /(?:from\s+|import\s*)["']@\/platform\/(?:desktop|web)["']/
const concretePlatformViolations = files.filter((file) => {
  if (/\.test\.(?:ts|tsx)$/.test(file)) {
    return false
  }
  return concretePlatformImport.test(readFileSync(file, "utf8"))
})

if (concretePlatformViolations.length > 0) {
  throw new Error(
    `Shared code must import the build-selected @curio/platform-runtime module:\n${concretePlatformViolations.join("\n")}`,
  )
}

// Only the API layer and the chat feature may touch the platform runtime;
// everything else goes through src/api or features/chat so service access
// stays behind one typed entry point.
const platformRuntimeImport = /(?:from\s+|import\s*)["']@curio\/platform-runtime["']/
const platformRuntimeViolations = files.filter((file) => {
  if (/\.test\.(?:ts|tsx)$/.test(file)) {
    return false
  }
  if (platformRuntimeConsumerRoots.some((root) => file.startsWith(root))) {
    return false
  }
  return platformRuntimeImport.test(readFileSync(file, "utf8"))
})

if (platformRuntimeViolations.length > 0) {
  throw new Error(
    `Only src/api and src/features/chat may use the platform runtime directly:\n${platformRuntimeViolations.join("\n")}`,
  )
}

console.log("Frontend architecture boundaries are intact.")
