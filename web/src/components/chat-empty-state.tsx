import { useRef, useState } from "react"
import {
  Code2,
  GraduationCap,
  Lightbulb,
  Pencil,
  Sparkles,
  type LucideIcon,
} from "lucide-react"

import { ChatComposer } from "@/components/chat-composer"
import {
  Suggestion,
  SuggestionList,
  Suggestions,
} from "@/components/nexus-ui/suggestions"
import { cn } from "@/lib/utils"

type ChatEmptyStateProps = {
  className?: string
  disabled?: boolean
  onSubmit?: (text: string) => void
}

const promptShortcuts: Array<{
  icon: LucideIcon
  label: string
  prompt: string
}> = [
  { icon: Pencil, label: "Write", prompt: "Help me write " },
  { icon: GraduationCap, label: "Learn", prompt: "Teach me about " },
  { icon: Code2, label: "Code", prompt: "Help me build " },
  { icon: Lightbulb, label: "Think", prompt: "Help me think through " },
  {
    icon: Sparkles,
    label: "Surprise me",
    prompt: "Give me something interesting to explore",
  },
]

export function ChatEmptyState({
  className,
  disabled = false,
  onSubmit,
}: ChatEmptyStateProps) {
  const [text, setText] = useState("")
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  // The shortcuts are sentence openers, so they prefill the composer and hand
  // focus back rather than sending an unfinished prompt.
  const selectShortcut = (prompt: string) => {
    setText(prompt)
    requestAnimationFrame(() => {
      const textarea = textareaRef.current
      textarea?.focus()
      textarea?.setSelectionRange(prompt.length, prompt.length)
    })
  }

  return (
    <div
      className={cn(
        "mx-auto flex h-full w-full flex-col items-center justify-center overflow-y-auto px-5 pb-16 pt-8 md:px-8",
        className,
      )}
    >
      <div className="w-full max-w-[42rem]">
        <div className="rise-in mb-8">
          <p className="eyebrow text-muted-foreground">New conversation</p>
          <h2 className="mt-3 font-display text-display-lg text-foreground">
            What are you curious about?
          </h2>
          <p className="mt-3 font-display text-lg italic text-muted-foreground">
            Start with a question, an idea, or something you want to make.
          </p>
        </div>

        <ChatComposer
          className="rise-in [--rise-index:1]"
          disabled={disabled}
          textareaRef={textareaRef}
          value={text}
          onValueChange={setText}
          onSubmit={onSubmit}
        />

        <Suggestions onSelect={selectShortcut}>
          <SuggestionList className="rise-in mt-4 justify-start [--rise-index:2]">
            {promptShortcuts.map(({ icon: Icon, label, prompt }) => (
              <Suggestion key={label} variant="outline" value={prompt}>
                <Icon className="size-3.5" aria-hidden="true" />
                <span>{label}</span>
              </Suggestion>
            ))}
          </SuggestionList>
        </Suggestions>
      </div>
    </div>
  )
}
