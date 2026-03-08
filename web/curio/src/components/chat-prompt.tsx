import { Button } from "@/components/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Textarea } from "@/components/ui/textarea"
import { cn } from "@/lib/utils"
import { Clock3, Plus, Send } from "lucide-react"
import { type FormEvent, useState } from "react"

type ChatPromptProps = {
  className?: string
  placeholder?: string
}

export function ChatPrompt({
  className,
  placeholder = "How can I help you today?",
}: ChatPromptProps) {
  const [text, setText] = useState("")
  const [model, setModel] = useState("sonnet-4.5")

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!text.trim()) {
      return
    }
    setText("")
  }

  return (
    <form
      onSubmit={handleSubmit}
      className={cn(
        "w-full rounded-[2rem] border border-primary/25 bg-primary/10 p-4 shadow-xl backdrop-blur-2xl md:p-5",
        className,
      )}
    >
      <div className="w-full space-y-4">
        <div className="grid grid-cols-[auto_1fr_auto] items-start gap-2 md:gap-3">
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="mt-1 size-9 rounded-lg text-muted-foreground hover:bg-background/40 hover:text-primary"
            aria-label="Attach files"
          >
            <Plus className="size-5" />
          </Button>

          <Textarea
            value={text}
            onChange={(event) => setText(event.target.value)}
            placeholder={placeholder}
            className="min-h-[64px] max-h-44 resize-none border-0 bg-transparent px-0 py-1 text-base leading-snug font-medium placeholder:text-sm placeholder:text-foreground/45 shadow-none outline-none ring-0 focus-visible:border-transparent focus-visible:ring-0 md:min-h-[72px] md:text-lg md:placeholder:text-base"
          />

          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="mt-1 size-9 rounded-lg text-muted-foreground hover:bg-background/40 hover:text-primary"
            aria-label="Prompt history"
          >
            <Clock3 className="size-5" />
          </Button>
        </div>

        <div className="h-px w-full bg-primary/20" />

        <div className="flex items-center justify-between gap-3">
          <Select value={model} onValueChange={setModel}>
            <SelectTrigger className="h-10 rounded-lg border-primary/25 bg-background/70 px-3 text-sm font-semibold text-foreground shadow-none md:min-w-44">
              <SelectValue placeholder="Select model" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="sonnet-4.5">Sonnet 4.5</SelectItem>
              <SelectItem value="haiku-3.5">Haiku 3.5</SelectItem>
              <SelectItem value="opus-4">Opus 4</SelectItem>
            </SelectContent>
          </Select>

          <Button
            type="submit"
            disabled={!text.trim()}
            className="size-10 rounded-lg bg-gradient-to-br from-primary to-accent text-primary-foreground shadow-md hover:from-primary/90 hover:to-accent/90"
          >
            <Send className="size-4.5" />
          </Button>
        </div>
      </div>
    </form>
  )
}
