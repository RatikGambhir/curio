import { ChatComposer } from "@/components/chat-composer"

type ChatPromptProps = {
  className?: string
  disabled?: boolean
  placeholder?: string
  onSubmit?: (text: string) => void
}

export function ChatPrompt({
  className,
  disabled = false,
  placeholder = "Ask a follow-up",
  onSubmit,
}: ChatPromptProps) {
  return (
    <ChatComposer
      className={className}
      compact
      disabled={disabled}
      placeholder={placeholder}
      onSubmit={onSubmit}
    />
  )
}
