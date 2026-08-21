import { Channel, invoke } from "@tauri-apps/api/core"

import {
  ServiceTransportError,
  type PlatformServices,
  type ServiceRequestInit,
  type ServiceStreamHandlers,
} from "@/platform/contracts"

type DesktopServicePacket =
  | { type: "started"; status: number }
  | { type: "chunk"; bytes: string }
  | { type: "end" }
  | { type: "error"; code: string; message: string }

function decodeBase64Chunk(value: string): Uint8Array {
  const binary = atob(value)
  const bytes = new Uint8Array(binary.length)
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index)
  }
  return bytes
}

function abortError(): DOMException {
  return new DOMException("The service request was canceled.", "AbortError")
}

async function streamService(
  init: ServiceRequestInit,
  handlers: ServiceStreamHandlers,
): Promise<void> {
  const { signal } = init
  if (signal?.aborted) {
    throw abortError()
  }

  const requestId = crypto.randomUUID()
  const channel = new Channel<DesktopServicePacket>()
  let ended = false
  let failure: Error | null = null

  channel.onmessage = (packet) => {
    if (failure) {
      return
    }

    try {
      if (packet.type === "started") {
        handlers.onStatus(packet.status)
      } else if (packet.type === "chunk") {
        handlers.onChunk(decodeBase64Chunk(packet.bytes))
      } else if (packet.type === "end") {
        ended = true
      } else {
        failure = new ServiceTransportError(packet.code, packet.message)
      }
    } catch (error) {
      failure = error instanceof Error ? error : new Error(String(error))
      void invoke("cancel_request", { requestId })
    }
  }

  const handleAbort = () => {
    void invoke("cancel_request", { requestId })
  }
  signal?.addEventListener("abort", handleAbort, { once: true })

  try {
    await invoke("service_request", {
      requestId,
      method: init.method,
      path: init.path,
      payload: (init.method === "GET" ? init.query : init.body) ?? null,
      bearerToken: init.bearerToken ?? null,
      onPacket: channel,
    })

    if (signal?.aborted) {
      throw abortError()
    }
    if (failure) {
      throw failure
    }
    if (!ended) {
      throw new ServiceTransportError(
        "incomplete_transport",
        "The desktop bridge ended before the service response was complete.",
      )
    }
  } catch (error) {
    if (signal?.aborted) {
      throw abortError()
    }
    if (error instanceof Error) {
      throw error
    }
    throw new ServiceTransportError(
      "desktop_bridge_error",
      typeof error === "string"
        ? error
        : "The desktop bridge could not complete the service request.",
    )
  } finally {
    signal?.removeEventListener("abort", handleAbort)
    channel.onmessage = () => undefined
  }
}

export const platformServices: PlatformServices = {
  target: "desktop",
  service: { stream: streamService },
  async openExternalUrl(value) {
    await invoke("open_external_url", { url: value })
  },
}
