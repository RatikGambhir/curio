export type AppTarget = "web" | "desktop"

export type ServiceMethod = "GET" | "POST" | "PATCH" | "DELETE"

export type ServiceQuery = Record<string, string | number | boolean>

export type ServiceRequestInit = {
  method: ServiceMethod
  /** Service-relative path beginning with a single forward slash. */
  path: string
  /** Query parameters. Only used for GET requests. */
  query?: ServiceQuery
  /** JSON payload. Only used for non-GET requests. */
  body?: unknown
  /** Attached as an Authorization bearer header when provided. */
  bearerToken?: string
  signal?: AbortSignal
}

export type ServiceStreamHandlers = {
  /** Called exactly once with the HTTP status, before any chunk. */
  onStatus: (status: number) => void
  /** Called with each raw response body chunk, in order. */
  onChunk: (bytes: Uint8Array) => void
}

export class ServiceTransportError extends Error {
  readonly code: string

  constructor(code: string, message: string) {
    super(message)
    this.name = "ServiceTransportError"
    this.code = code
  }
}

/**
 * The single network primitive each platform implements: web routes it
 * through fetch, desktop through an app-owned Tauri command that performs the
 * HTTP request in the Rust process.
 *
 * Contract: `stream` resolves after the response body has been fully
 * delivered. It rejects with an "AbortError" DOMException when the signal
 * aborts, with the handler's error when a handler throws (after canceling the
 * underlying request), and with ServiceTransportError on transport failures.
 */
export interface ServiceTransport {
  stream(
    init: ServiceRequestInit,
    handlers: ServiceStreamHandlers,
  ): Promise<void>
}

export type ServiceResponse = {
  status: number
  bytes: Uint8Array
}

export function concatBytes(chunks: Uint8Array[]): Uint8Array {
  const length = chunks.reduce((total, chunk) => total + chunk.byteLength, 0)
  const combined = new Uint8Array(length)
  let offset = 0
  for (const chunk of chunks) {
    combined.set(chunk, offset)
    offset += chunk.byteLength
  }
  return combined
}

/** Buffered convenience wrapper over ServiceTransport.stream. */
export async function requestService(
  transport: ServiceTransport,
  init: ServiceRequestInit,
): Promise<ServiceResponse> {
  let status: number | null = null
  const chunks: Uint8Array[] = []

  await transport.stream(init, {
    onStatus: (value) => {
      status = value
    },
    onChunk: (bytes) => {
      chunks.push(bytes)
    },
  })

  if (status === null) {
    throw new ServiceTransportError(
      "missing_status",
      "The platform transport did not report a response status.",
    )
  }
  return { status, bytes: concatBytes(chunks) }
}

export interface PlatformServices {
  target: AppTarget
  service: ServiceTransport
  openExternalUrl(url: string): Promise<void>
}

export function parseExternalHttpUrl(value: string): URL {
  const url = new URL(value)
  if (url.protocol !== "https:" && url.protocol !== "http:") {
    throw new Error("Curio can only open HTTP or HTTPS links.")
  }
  return url
}
