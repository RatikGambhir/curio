export type ChatMessage = {
  id: string
  from: "user" | "assistant"
  value: string
}

export const mockMessagesByChatId: Record<string, ChatMessage[]> = {
  "chat-1": [
    {
      id: "1",
      from: "user",
      value: "Can you give me three launch headline directions for Curio?",
    },
    {
      id: "2",
      from: "assistant",
      value:
        "Here are three directions: trust-first knowledge capture, elegant AI workspace, and personal research companion.",
    },
  ],
  "chat-2": [
    {
      id: "3",
      from: "user",
      value: "Build a 4-day split for strength and mobility.",
    },
    {
      id: "4",
      from: "assistant",
      value:
        "A balanced plan would pair upper and lower strength days with two shorter mobility-led accessory days.",
    },
  ],
  "chat-3": [],
  "chat-4": [
    {
      id: "5",
      from: "assistant",
      value:
        "The highest regression risk is around navigation state and mobile sidebar collapse behavior.",
    },
  ],
}
