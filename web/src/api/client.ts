import { platformServices } from "@curio/platform-runtime"

import { ApiError, serviceErrorMessage } from "@/api/errors"
import {
  requestService,
  type ServiceQuery,
  type ServiceRequestInit,
} from "@/platform/contracts"

export type ApiRequestOptions = {
  bearerToken?: string
  signal?: AbortSignal
}

async function requestJson<T>(init: ServiceRequestInit): Promise<T> {
  const { status, bytes } = await requestService(platformServices.service, init)
  const text = new TextDecoder().decode(bytes)

  if (status < 200 || status >= 300) {
    throw new ApiError(
      status,
      serviceErrorMessage(text) ?? `The Curio service returned HTTP ${status}.`,
    )
  }
  if (!text.trim()) {
    return undefined as T
  }

  try {
    return JSON.parse(text) as T
  } catch {
    throw new ApiError(status, "The Curio service returned malformed JSON.")
  }
}

export const api = {
  get<T>(
    path: string,
    query?: ServiceQuery,
    options?: ApiRequestOptions,
  ): Promise<T> {
    return requestJson<T>({ method: "GET", path, query, ...options })
  },
  post<T>(
    path: string,
    body?: unknown,
    options?: ApiRequestOptions,
  ): Promise<T> {
    return requestJson<T>({ method: "POST", path, body, ...options })
  },
  patch<T>(
    path: string,
    body?: unknown,
    options?: ApiRequestOptions,
  ): Promise<T> {
    return requestJson<T>({ method: "PATCH", path, body, ...options })
  },
  delete<T>(path: string, options?: ApiRequestOptions): Promise<T> {
    return requestJson<T>({ method: "DELETE", path, ...options })
  },
}
