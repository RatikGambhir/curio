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
        "mx-auto flex h-full w-full flex-col items-center justify-center px-4 pb-20 pt-8 md:px-8",
        className,
      )}
    >
      <div className="w-full max-w-3xl">
        <div className="mb-7 text-center">
          <h1 className="text-2xl font-normal tracking-[-0.025em] text-foreground md:text-[1.75rem]">
            What can I help you with?
          </h1>
          <p className="mt-2 text-sm font-normal text-muted-foreground">
            Start with a question, an idea, or something you want to make.
          </p>
        </div>

        <ChatComposer
          disabled={disabled}
          textareaRef={textareaRef}
          value={text}
          onValueChange={setText}
          onSubmit={onSubmit}
        />

        <Suggestions onSelect={selectShortcut}>
          <SuggestionList className="mt-4 justify-center">
            {promptShortcuts.map(({ icon: Icon, label, prompt }) => (
              <Suggestion key={label} variant="outline" value={prompt}>
                <Icon className="size-3.5 stroke-[1.7]" aria-hidden="true" />
                <span>{label}</span>
              </Suggestion>
            ))}
          </SuggestionList>
        </Suggestions>
      </div>
    </div>
  )
}
