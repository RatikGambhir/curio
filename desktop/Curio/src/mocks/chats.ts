export type ChatListItem = {
  id: string;
  title: string;
  updatedAt: string;
  preview?: string;
};

export type ChatMessage = {
  id: string;
  from: "user" | "assistant";
  value: string;
};

export const mockChats: ChatListItem[] = [
  {
    id: "chat-1",
    title: "Product brainstorming",
    updatedAt: "2m ago",
    preview: "Can you draft three launch headlines?",
  },
  {
    id: "chat-2",
    title: "Workout plan",
    updatedAt: "1h ago",
    preview: "Build a 4-day split for strength and mobility.",
  },
  {
    id: "chat-3",
    title: "Travel itinerary",
    updatedAt: "Yesterday",
    preview: "Weekend plan for Austin with coffee stops.",
  },
  {
    id: "chat-4",
    title: "Code review notes",
    updatedAt: "Thu",
    preview: "Summarize regression risks from the latest PR.",
  },
];

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
};
