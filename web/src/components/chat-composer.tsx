import { useState, type Ref } from "react"
import { ArrowUp, Paperclip, Square } from "lucide-react"

import {
  PromptInput,
  PromptInputAction,
  PromptInputActionGroup,
  PromptInputActions,
  PromptInputTextarea,
} from "@/components/nexus-ui/prompt-input"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"

type ChatComposerProps = {
  className?: string
  /** Blocks sending. The textarea stays editable so a follow-up can be drafted. */
  disabled?: boolean
  isStreaming?: boolean
  placeholder?: string
  textareaRef?: Ref<HTMLTextAreaElement>
  /** Optional controlled value, so callers can prefill from a suggestion. */
  value?: string
  onValueChange?: (value: string) => void
  onStop?: () => void
  onSubmit?: (text: string) => void
}

export function ChatComposer({
  className,
  disabled = false,
  isStreaming = false,
  placeholder = "Ask anything",
  textareaRef,
  value,
  onValueChange,
  onStop,
  onSubmit,
}: ChatComposerProps) {
  const [uncontrolledValue, setUncontrolledValue] = useState("")
  const isControlled = value !== undefined
  const text = isControlled ? value : uncontrolledValue

  const setText = (nextText: string) => {
    if (!isControlled) {
      setUncontrolledValue(nextText)
    }
    onValueChange?.(nextText)
  }

  const submitText = () => {
    const nextText = text.trim()
    if (!nextText || disabled) {
      return
    }

    onSubmit?.(nextText)
    setText("")
  }

  return (
    <PromptInput
      onSubmit={submitText}
      className={cn(
        "shadow-sm transition-[border-color,box-shadow] focus-within:border-ring/60 focus-within:shadow-md",
        className,
      )}
    >
      <PromptInputTextarea
        ref={textareaRef}
        aria-label="Message Curio"
        placeholder={placeholder}
        value={text}
        onChange={(event) => setText(event.target.value)}
      />

      <PromptInputActions>
        <PromptInputActionGroup>
          <PromptInputAction tooltip="Attach files" asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label="Attach files"
              className="rounded-lg text-muted-foreground hover:translate-y-0 hover:text-foreground"
            >
              <Paperclip className="size-4 stroke-[1.7]" />
            </Button>
          </PromptInputAction>
        </PromptInputActionGroup>

        <PromptInputActionGroup>
          {isStreaming && onStop ? (
            <PromptInputAction tooltip="Stop generating" asChild>
              <Button
                type="button"
                size="icon-sm"
                aria-label="Stop generating"
                onClick={onStop}
                className="rounded-lg bg-foreground text-background hover:translate-y-0 hover:bg-foreground/85"
              >
                <Square className="size-3.5 fill-current" />
              </Button>
            </PromptInputAction>
          ) : (
            <PromptInputAction
              tooltip={{ content: "Send message", shortcut: "Enter" }}
              asChild
            >
              <Button
                type="button"
                size="icon-sm"
                aria-label="Send message"
                disabled={!text.trim() || disabled}
                onClick={submitText}
                className="rounded-lg bg-foreground text-background hover:translate-y-0 hover:bg-foreground/85 disabled:opacity-30"
              >
                <ArrowUp className="size-4 stroke-2" />
              </Button>
            </PromptInputAction>
          )}
        </PromptInputActionGroup>
      </PromptInputActions>
    </PromptInput>
  )
}
