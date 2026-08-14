import { existsSync, readdirSync, readFileSync, statSync } from "node:fs"
import { resolve } from "node:path"

const appRoot = process.cwd()
const legacyReactRoots = [
  resolve(appRoot, "../../desktop/Curio/src"),
  resolve(appRoot, "../src"),
]
for (const legacyReactRoot of legacyReactRoots) {
  if (existsSync(legacyReactRoot)) {
    throw new Error(`Duplicate React source tree found at ${legacyReactRoot}`)
  }
}

const sourceRoot = resolve(appRoot, "src")
const allowedTauriModule = resolve(sourceRoot, "platform/desktop.ts")

function sourceFiles(directory) {
  return readdirSync(directory).flatMap((entry) => {
    const path = resolve(directory, entry)
    if (statSync(path).isDirectory()) {
      return sourceFiles(path)
    }
    return /\.(ts|tsx)$/.test(path) ? [path] : []
  })
}

const violations = sourceFiles(sourceRoot).filter((file) => {
  if (file === allowedTauriModule) {
    return false
  }
  return /(?:from\s+|import\s*)["']@tauri-apps\//.test(
    readFileSync(file, "utf8"),
  )
})

if (violations.length > 0) {
  throw new Error(
    `Tauri imports escaped the desktop adapter:\n${violations.join("\n")}`,
  )
}

console.log("Frontend architecture boundaries are intact.")
