import { describe, expect, it } from "vitest"

import {
  ChatStreamParser,
  ChatStreamProtocolError,
  parseChatStreamEvent,
} from "./chat-stream"

const ids = {
  conversationId: "conversation-1",
  messageId: "assistant-1",
}

function block(event: string, data: Record<string, unknown>, lineEnding = "\n") {
  return `event: ${event}${lineEnding}data: ${JSON.stringify(data)}${lineEnding}${lineEnding}`
}

describe("ChatStreamParser", () => {
  it("parses an event split across arbitrary network chunks", () => {
    const parser = new ChatStreamParser()
    const input = block("token", { ...ids, token: "Hello" })

    expect(parser.push(input.slice(0, 11))).toEqual([])
    expect(parser.push(input.slice(11, 37))).toEqual([])
    expect(parser.push(input.slice(37))).toEqual([
      { type: "token", ...ids, token: "Hello" },
    ])
  })

  it("parses multiple CRLF-delimited events from one chunk", () => {
    const parser = new ChatStreamParser()
    const input =
      block("token", { ...ids, token: "Hel" }, "\r\n") +
      block("token", { ...ids, token: "lo" }, "\r\n")

    expect(parser.push(input)).toEqual([
      { type: "token", ...ids, token: "Hel" },
      { type: "token", ...ids, token: "lo" },
    ])
  })

  it("rejects malformed JSON", () => {
    const parser = new ChatStreamParser()

    expect(() => parser.push("event: token\ndata: {not-json}\n\n")).toThrowError(
      /malformed JSON/,
    )
  })

  it("surfaces normalized provider errors as terminal events", () => {
    const parser = new ChatStreamParser()
    const input = block("error", {
      ...ids,
      code: "provider_error",
      message: "The model request failed.",
    })

    expect(parser.push(input)).toEqual([
      {
        type: "error",
        ...ids,
        code: "provider_error",
        message: "The model request failed.",
      },
    ])
    expect(parser.finish()).toEqual([])
  })

  it("recognizes completion and accepts a clean end of stream", () => {
    const parser = new ChatStreamParser()
    const input = block("done", { ...ids, responseId: "response-1" })

    expect(parser.push(input)).toEqual([
      { type: "done", ...ids, responseId: "response-1" },
    ])
    expect(parser.finish()).toEqual([])
  })

  it("rejects a truncated stream without a terminal event", () => {
    const parser = new ChatStreamParser()
    parser.push(block("token", { ...ids, token: "partial" }))

    expect(() => parser.finish()).toThrowError(
      expect.objectContaining<Partial<ChatStreamProtocolError>>({
        code: "truncated_stream",
      }),
    )
  })
})

describe("parseChatStreamEvent", () => {
  it("rejects invalid normalized payloads", () => {
    expect(() =>
      parseChatStreamEvent('event: done\ndata: {"conversationId":"conversation-1"}'),
    ).toThrowError(/messageId/)
  })
})
