export class ApiError extends Error {
  readonly status: number

  constructor(status: number, message: string) {
    super(message)
    this.name = "ApiError"
    this.status = status
  }
}

/**
 * Extracts a human-readable message from a Curio service error body, which
 * may be plain text or JSON shaped as { message } or { error }.
 */
export function serviceErrorMessage(value: unknown): string | null {
  if (typeof value === "string") {
    const valueTrimmed = value.trim()
    if (!valueTrimmed) {
      return null
    }

    try {
      return serviceErrorMessage(JSON.parse(valueTrimmed)) ?? valueTrimmed
    } catch {
      return valueTrimmed
    }
  }

  if (value && typeof value === "object") {
    const payload = value as { message?: unknown; error?: unknown }
    return (
      serviceErrorMessage(payload.message) ?? serviceErrorMessage(payload.error)
    )
  }

  return null
}
