import {
  PromptInput,
  PromptInputBody,
  PromptInputTextarea,
  PromptInputButton,
} from "@/components/ai-elements/prompt-input"
import type { PromptInputMessage } from "@/components/ai-elements/prompt-input"
import { Button } from "@/components/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { cn } from "@/lib/utils"
import { Clock3, Plus, Send } from "lucide-react"
import { type FormEvent, useRef, useState } from "react"

type ChatPromptProps = {
  className?: string
  placeholder?: string
}

export function ChatPrompt({
  className,
  placeholder = "How can I help you today?",
}: ChatPromptProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const [text, setText] = useState("")
  const [model, setModel] = useState("sonnet-4.5")

  //const { messages, status, sendMessage } = useChat();
  const askQuestion = async (
    message: PromptInputMessage,
    event: FormEvent<HTMLFormElement>,
  ) => {
    event.preventDefault()
    if (!message.text.trim() && message.files.length === 0) {
      return
    }
    setText("")
  }

  return (
    <PromptInput
      onSubmit={askQuestion}
      className={cn(
        "w-full rounded-[2.5rem] border border-primary/25 bg-primary/10 p-5 shadow-xl backdrop-blur-2xl md:p-7",
        className,
      )}
    >
      <div className="w-full space-y-5">
        <div className="grid grid-cols-[auto_1fr_auto] items-start gap-3 md:gap-4">
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="mt-1 size-10 rounded-xl text-muted-foreground hover:bg-background/40 hover:text-primary"
            aria-label="Attach files"
          >
            <Plus className="size-6" />
          </Button>
          <PromptInputBody className="min-w-0 flex-1">
            <PromptInputTextarea
              onChange={(e) => setText(e.target.value)}
              ref={textareaRef}
              value={text}
              placeholder={placeholder}
              className="min-h-[88px] max-h-56 px-0 py-1 text-xl leading-tight font-medium placeholder:text-foreground/45 md:text-3xl"
            />
          </PromptInputBody>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="mt-1 size-10 rounded-xl text-muted-foreground hover:bg-background/40 hover:text-primary"
            aria-label="Prompt history"
          >
            <Clock3 className="size-6" />
          </Button>
        </div>

        <div className="h-px w-full bg-primary/20" />

        <div className="flex items-center justify-between gap-3">
          <Select value={model} onValueChange={setModel}>
            <SelectTrigger className="h-12 rounded-xl border-primary/25 bg-background/70 px-4 text-base font-semibold text-foreground shadow-none md:min-w-48">
              <SelectValue placeholder="Select model" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="sonnet-4.5">Sonnet 4.5</SelectItem>
              <SelectItem value="haiku-3.5">Haiku 3.5</SelectItem>
              <SelectItem value="opus-4">Opus 4</SelectItem>
            </SelectContent>
          </Select>

          <PromptInputButton
            type="submit"
            disabled={!text.trim()}
            className="size-12 rounded-xl bg-gradient-to-br from-primary to-accent text-primary-foreground shadow-md hover:from-primary/90 hover:to-accent/90"
          >
            <Send className="size-5" />
          </PromptInputButton>
        </div>
      </div>
    </PromptInput>
  )
}
