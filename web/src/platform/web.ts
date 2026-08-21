import {
  parseExternalHttpUrl,
  ServiceTransportError,
  type PlatformServices,
  type ServiceRequestInit,
  type ServiceStreamHandlers,
} from "@/platform/contracts"

function buildEndpoint(init: ServiceRequestInit): URL {
  const url = new URL(init.path, __CURIO_SERVICE_URL__)
  if (init.method === "GET" && init.query) {
    for (const [key, value] of Object.entries(init.query)) {
      url.searchParams.set(key, String(value))
    }
  }
  return url
}

async function streamService(
  init: ServiceRequestInit,
  handlers: ServiceStreamHandlers,
): Promise<void> {
  const headers: Record<string, string> = {}
  let body: string | undefined
  if (init.method !== "GET" && init.body !== undefined) {
    headers["Content-Type"] = "application/json"
    body = JSON.stringify(init.body)
  }
  if (init.bearerToken) {
    headers.Authorization = `Bearer ${init.bearerToken}`
  }

  let response: Response
  try {
    response = await fetch(buildEndpoint(init), {
      method: init.method,
      headers,
      body,
      signal: init.signal,
    })
  } catch (error) {
    if (error instanceof DOMException && error.name === "AbortError") {
      throw error
    }
    throw new ServiceTransportError(
      "service_unavailable",
      "The Curio service is unavailable.",
    )
  }

  handlers.onStatus(response.status)
  if (!response.body) {
    return
  }

  const reader = response.body.getReader()
  while (true) {
    let result: ReadableStreamReadResult<Uint8Array>
    try {
      result = await reader.read()
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") {
        throw error
      }
      throw new ServiceTransportError(
        "service_unavailable",
        "The connection to the Curio service was interrupted.",
      )
    }

    if (result.done) {
      return
    }

    try {
      handlers.onChunk(result.value)
    } catch (error) {
      await reader.cancel().catch(() => undefined)
      throw error
    }
  }
}

export const platformServices: PlatformServices = {
  target: "web",
  service: { stream: streamService },
  async openExternalUrl(value) {
    const url = parseExternalHttpUrl(value)
    const opened = window.open(url, "_blank", "noopener,noreferrer")
    if (!opened) {
      throw new Error("The browser blocked Curio from opening this link.")
    }
  },
}
