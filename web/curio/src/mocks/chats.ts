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

export const funnyThinkingTerms = {
  general: [
    "Let me consult the archives.",
    "Stand by while I manufacture a thought.",
    "Loading wisdom…",
    "Buffering.",
    "Give me a sec, my brain is on dial-up.",
    "Hang on, the hamsters are still running.",
    "Let me pretend I know this immediately.",
    "One moment while I assemble the brain cells.",
    "The gears are turning.",
    "I’m downloading the answer from headquarters.",
    "Please hold while I overthink this.",
    "Let me cook.",
    "My thoughts are in transit.",
    "Processing… please do not disturb the machinery.",
    "Give me a second, the committee in my head is voting.",
    "Hold please, the oracle is waking up.",
    "I’ve got a loading circle where my answer should be.",
    "Let me reach into the void real quick.",
    "Brain.exe is starting.",
    "One sec, I’m trying to make this sound smarter than it is."
  ],
  deadpan: [
    "Excellent question. Regrettably, I now have to think.",
    "You’ve forced me into active cognition.",
    "I was hoping instinct would handle this.",
    "Now entering: thought.",
    "This may require the deluxe version of my brain."
  ],
  chaotic: [
    "Hold on, my last brain cell is doing parkour.",
    "Give me a sec, the raccoons in my head are sorting it out.",
    "My brain just hit a red light.",
    "One moment, the internal PowerPoint is still loading.",
    "Thoughts are currently circling for landing."
  ],
  casualTexting: [
    "Wait lol let me think",
    "Hold up I’m processing",
    "Lmao one sec",
    "Waitttt gears turning",
    "Hang on I’m cooking",
    "Brain loading…"
  ],
  bestAllAround: [
    "Hold on, the gears are turning.",
    "Give me a sec, my brain is on dial-up.",
    "Let me cook.",
    "Please hold while I overthink this.",
    "One moment, the committee in my head is voting."
  ]
}
