export type ChatStreamTokenEvent = {
  type: "token"
  conversationId: string
  messageId: string
  token: string
}

export type ChatStreamErrorEvent = {
  type: "error"
  conversationId: string
  messageId: string
  code: string
  message: string
}

export type ChatStreamDoneEvent = {
  type: "done"
  conversationId: string
  messageId: string
  responseId: string
}

export type ChatStreamEvent =
  | ChatStreamTokenEvent
  | ChatStreamErrorEvent
  | ChatStreamDoneEvent

export type ChatStreamRequest = {
  userId: string
  conversationId: string
  userMessageId: string
  assistantMessageId: string
  prompt: string
}

export type ExpectedChatStream = {
  conversationId: string
  messageId: string
}

export type ChatStreamProtocolErrorCode =
  | "malformed_event"
  | "truncated_stream"
  | "event_after_terminal"

export class ChatStreamProtocolError extends Error {
  readonly code: ChatStreamProtocolErrorCode

  constructor(code: ChatStreamProtocolErrorCode, message: string) {
    super(message)
    this.name = "ChatStreamProtocolError"
    this.code = code
  }
}

export class ChatStreamRequestError extends Error {
  readonly code: string

  constructor(code: string, message: string) {
    super(message)
    this.name = "ChatStreamRequestError"
    this.code = code
  }
}

function malformed(message: string): never {
  throw new ChatStreamProtocolError("malformed_event", message)
}

function requireObject(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return malformed("Stream event data must be a JSON object.")
  }

  return value as Record<string, unknown>
}

function requireString(
  payload: Record<string, unknown>,
  field: string,
  eventName: string,
): string {
  const value = payload[field]
  if (typeof value !== "string" || value.length === 0) {
    return malformed(`Stream ${eventName} event is missing ${field}.`)
  }

  return value
}

export function parseChatStreamEvent(eventBlock: string): ChatStreamEvent | null {
  let eventName = ""
  const dataLines: string[] = []

  for (const line of eventBlock.split(/\r?\n/)) {
    if (!line || line.startsWith(":")) {
      continue
    }

    const separatorIndex = line.indexOf(":")
    const field = separatorIndex === -1 ? line : line.slice(0, separatorIndex)
    let value = separatorIndex === -1 ? "" : line.slice(separatorIndex + 1)
    if (value.startsWith(" ")) {
      value = value.slice(1)
    }

    if (field === "event") {
      eventName = value
    } else if (field === "data") {
      dataLines.push(value)
    }
  }

  if (!eventName && dataLines.length === 0) {
    return null
  }

  if (eventName !== "token" && eventName !== "error" && eventName !== "done") {
    return malformed(`Unsupported stream event: ${eventName || "message"}.`)
  }

  if (dataLines.length === 0) {
    return malformed(`Stream ${eventName} event is missing data.`)
  }

  let decoded: unknown
  try {
    decoded = JSON.parse(dataLines.join("\n"))
  } catch {
    return malformed(`Stream ${eventName} event contains malformed JSON.`)
  }

  const payload = requireObject(decoded)
  const common = {
    conversationId: requireString(payload, "conversationId", eventName),
    messageId: requireString(payload, "messageId", eventName),
  }

  if (eventName === "token") {
    const token = payload.token
    if (typeof token !== "string") {
      return malformed("Stream token event is missing token.")
    }

    return { type: "token", ...common, token }
  }

  if (eventName === "error") {
    return {
      type: "error",
      ...common,
      code: requireString(payload, "code", eventName),
      message: requireString(payload, "message", eventName),
    }
  }

  return {
    type: "done",
    ...common,
    responseId: requireString(payload, "responseId", eventName),
  }
}

export class ChatStreamParser {
  private buffer = ""
  private terminalEventReceived = false

  push(chunk: string): ChatStreamEvent[] {
    if (this.terminalEventReceived && chunk.trim()) {
      throw new ChatStreamProtocolError(
        "event_after_terminal",
        "Received stream data after a terminal event.",
      )
    }

    this.buffer += chunk
    const events: ChatStreamEvent[] = []
    let boundary = this.buffer.match(/\r?\n\r?\n/)

    while (boundary?.index !== undefined) {
      const eventBlock = this.buffer.slice(0, boundary.index)
      this.buffer = this.buffer.slice(boundary.index + boundary[0].length)
      const event = parseChatStreamEvent(eventBlock)
      if (event) {
        this.accept(event, events)
      }
      boundary = this.buffer.match(/\r?\n\r?\n/)
    }

    return events
  }

  finish(): ChatStreamEvent[] {
    const events: ChatStreamEvent[] = []
    if (this.buffer.trim()) {
      const event = parseChatStreamEvent(this.buffer)
      this.buffer = ""
      if (event) {
        this.accept(event, events)
      }
    }

    if (!this.terminalEventReceived) {
      throw new ChatStreamProtocolError(
        "truncated_stream",
        "Chat stream ended before a done or error event.",
      )
    }

    return events
  }

  private accept(event: ChatStreamEvent, events: ChatStreamEvent[]) {
    if (this.terminalEventReceived) {
      throw new ChatStreamProtocolError(
        "event_after_terminal",
        "Received a stream event after a terminal event.",
      )
    }

    events.push(event)
    if (event.type === "done" || event.type === "error") {
      this.terminalEventReceived = true
    }
  }
}

export function workerErrorMessage(value: unknown): string | null {
  if (typeof value === "string") {
    const valueTrimmed = value.trim()
    if (!valueTrimmed) {
      return null
    }

    try {
      return workerErrorMessage(JSON.parse(valueTrimmed)) ?? valueTrimmed
    } catch {
      return valueTrimmed
    }
  }

  if (value && typeof value === "object") {
    const payload = value as { message?: unknown; error?: unknown }
    return workerErrorMessage(payload.message) ?? workerErrorMessage(payload.error)
  }

  return null
}

function validateCorrelation(event: ChatStreamEvent, expected: ExpectedChatStream) {
  if (
    event.conversationId !== expected.conversationId ||
    event.messageId !== expected.messageId
  ) {
    throw new ChatStreamProtocolError(
      "malformed_event",
      "Chat stream event identifiers do not match the request.",
    )
  }
}

export class ChatStreamSession {
  private readonly decoder = new TextDecoder()
  private readonly parser = new ChatStreamParser()
  private terminal: ChatStreamDoneEvent | ChatStreamErrorEvent | null = null

  constructor(
    private readonly expected: ExpectedChatStream,
    private readonly onEvent: (event: ChatStreamEvent) => void,
  ) {}

  push(bytes: Uint8Array): void {
    this.process(this.parser.push(this.decoder.decode(bytes, { stream: true })))
  }

  finish(): ChatStreamDoneEvent | ChatStreamErrorEvent {
    const decodedTail = this.decoder.decode()
    if (decodedTail) {
      this.process(this.parser.push(decodedTail))
    }
    this.process(this.parser.finish())

    if (!this.terminal) {
      throw new ChatStreamProtocolError(
        "truncated_stream",
        "Chat stream ended before a terminal event.",
      )
    }
    return this.terminal
  }

  private process(events: ChatStreamEvent[]): void {
    for (const event of events) {
      validateCorrelation(event, this.expected)
      this.onEvent(event)
      if (event.type === "done" || event.type === "error") {
        this.terminal = event
      }
    }
  }
}

export async function readChatStream(
  body: ReadableStream<Uint8Array> | null,
  expected: ExpectedChatStream,
  onEvent: (event: ChatStreamEvent) => void,
): Promise<ChatStreamDoneEvent | ChatStreamErrorEvent> {
  if (!body) {
    throw new ChatStreamRequestError(
      "missing_response_body",
      "The chat worker returned an empty response.",
    )
  }

  const reader = body.getReader()
  const session = new ChatStreamSession(expected, onEvent)

  while (true) {
    const { value, done } = await reader.read()
    if (done) {
      break
    }

    session.push(value)
  }
  return session.finish()
}

export async function streamCurioChat(
  workerUrl: string,
  request: ChatStreamRequest,
  signal: AbortSignal,
  onEvent: (event: ChatStreamEvent) => void,
) {
  const endpoint = new URL("/v1/chat/stream", workerUrl)
  const response = await fetch(endpoint, {
    body: JSON.stringify(request),
    method: "POST",
    headers: { "Content-Type": "application/json" },
    signal,
  })

  if (!response.ok) {
    const responseText = await response.text()
    throw new ChatStreamRequestError(
      "http_error",
      workerErrorMessage(responseText) ?? `Chat worker returned HTTP ${response.status}.`,
    )
  }

  return readChatStream(
    response.body,
    { conversationId: request.conversationId, messageId: request.assistantMessageId },
    onEvent,
  )
}
