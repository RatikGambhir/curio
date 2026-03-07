import { useMemo, useState } from "react"
import { ChatPrompt } from "@/components/chat-prompt"
import { ChatSidebar } from "@/components/chat-sidebar"
import { ChatView } from "@/components/chat-view"
import type { ChatListItem } from "@/components/ui/chat-nav"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar"
import {
  Clock3,
  Code2,
  GraduationCap,
  Lightbulb,
  Pencil,
  Plus,
  Send,
  Sparkles,
} from "lucide-react"

const linenNoise =
  "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 400 400' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E\")"

const Chat = () => {
  const chats: ChatListItem[] = useMemo(
    () => [
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
    ],
    [],
  )
  const [selectedChatId, setSelectedChatId] = useState<string | null>(
    chats[0]?.id ?? null,
  )
  const [draft, setDraft] = useState("")
  const [model, setModel] = useState("Gemini-3.5")

  const messages: number[] = []
  const promptShortcuts = useMemo(
    () => [
      { id: "write", icon: Pencil, label: "Write" },
      { id: "learn", icon: GraduationCap, label: "Learn" },
      { id: "code", icon: Code2, label: "Code" },
      { id: "life", icon: Lightbulb, label: "Life stuff" },
      { id: "choice", icon: Sparkles, label: "Claude's choice" },
    ],
    [],
  )
  // TODO: Replace mock data and local state with backend chat sessions.

  return (
    <SidebarProvider className="h-screen w-full font-sans">
      <ChatSidebar
        chats={chats}
        selectedChatId={selectedChatId}
        onSelectChat={setSelectedChatId}
        className="bg-background"
      />
      <SidebarInset className="bg-background">
        <div className="relative flex h-full w-full flex-col overflow-hidden bg-background px-4 py-6 md:px-8">
          <div
            aria-hidden="true"
            className="pointer-events-none absolute inset-0 opacity-[0.03]"
            style={{ backgroundImage: linenNoise, backgroundRepeat: "repeat" }}
          />
          <div className="relative z-10 flex h-full w-full flex-col">
            {messages.length > 0 ? (
              <div className="mx-auto flex h-full w-full max-w-4xl flex-col gap-4">
                <div className="min-h-0 flex-1 overflow-y-auto rounded-xl border border-border bg-card">
                  <ChatView />
                </div>
                <ChatPrompt />
              </div>
            ) : (
              <div className="mx-auto flex h-full w-full max-w-[110rem] flex-col items-center justify-center px-4 pb-16 pt-8 md:px-8">
                <div className="w-full h-full max-h-[20rem] max-w-[96rem] space-y-12 flex flex-col items-center justify-center">
                  <div className="w-full max-w-[64rem] h-full rounded-[2.85rem] border border-primary/18 bg-[linear-gradient(180deg,rgba(255,255,255,0.42),rgba(124,153,130,0.08))] shadow-[0_18px_60px_rgba(55,72,58,0.10)] backdrop-blur-xl">
                    <div className="grid grid-cols-[auto_1fr_auto] items-start gap-4 px-6 py-6 md:px-11 md:py-8">
                      <button
                        type="button"
                        aria-label="Attach files"
                        className="mt-1 inline-flex size-12 items-center justify-center rounded-2xl text-muted-foreground transition-colors hover:bg-white/45 hover:text-foreground"
                      >
                        <Plus className="size-8 stroke-[1.75]" />
                      </button>
                      <textarea
                        value={draft}
                        onChange={(event) => setDraft(event.target.value)}
                        placeholder="How can I help you today?"
                        rows={2}
                        className="min-h-24 w-full resize-none bg-transparent py-1 leading-tight font-semibold text-foreground outline-none placeholder:text-foreground/55 md:min-h-28 md:text-[1rem]"
                      />
                      <button
                        type="button"
                        aria-label="Prompt history"
                        className="mt-1 inline-flex size-12 items-center justify-center rounded-2xl text-muted-foreground transition-colors hover:bg-white/45 hover:text-foreground"
                      >
                        <Clock3 className="size-8 stroke-[1.75]" />
                      </button>
                    </div>

                    <div className="mx-6 h-px bg-primary/14 md:mx-11" />

                    <div className="flex items-center justify-between gap-4 px-2 py-2 md:px-4 md:py-4">
                      <Select value={model} onValueChange={setModel}>
                        <SelectTrigger className="h-14 min-w-44 rounded-[1rem] border-primary/15 bg-white/40 px-4 text-lg font-sans font-semibold text-foreground shadow-none md:min-w-52">
                          <SelectValue placeholder="Select model">
                            <span className="font-sans font-semibold text-foreground">
                              {model}
                            </span>
                          </SelectValue>
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="sonnet-4.5">ChatGPT-5.3</SelectItem>
                          <SelectItem value="haiku-3.5">Sonnet-3.5</SelectItem>
                          <SelectItem value="opus-4">Gemini-3.5-Pro</SelectItem>
                        </SelectContent>
                      </Select>

                      <button
                        type="button"
                        aria-label="Send message"
                        className="inline-flex size-11 items-center justify-center rounded-[1.15rem] bg-[linear-gradient(180deg,rgba(143,164,145,0.95),rgba(172,190,173,0.95))] text-primary-foreground shadow-[0_10px_24px_rgba(97,124,102,0.22)] transition-transform hover:-translate-y-0.5"
                      >
                        <Send className="size-7 stroke-[2]" />
                      </button>
                    </div>
                  </div>

                  <div className="flex flex-wrap items-center justify-center gap-5">
                    {promptShortcuts.map((shortcut) => (
                      <button
                        key={shortcut.id}
                        type="button"
                        className="inline-flex items-center gap-2.5 rounded-[1.35rem] border border-border/70 bg-white/65 px-6 py-3 text-lg font-sans font-semibold text-foreground shadow-[0_10px_28px_rgba(37,42,52,0.08)] backdrop-blur-md transition-all hover:-translate-y-0.5 hover:bg-white/80"
                      >
                        <shortcut.icon
                          className="size-5 text-foreground"
                          strokeWidth={1.9}
                          aria-hidden="true"
                        />
                        <span>{shortcut.label}</span>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}

export default Chat
