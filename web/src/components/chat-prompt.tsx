import { ChatComposer } from "@/components/chat-composer"

type ChatPromptProps = {
  className?: string
  disabled?: boolean
  isStreaming?: boolean
  placeholder?: string
  onStop?: () => void
  onSubmit?: (text: string) => void
}

export function ChatPrompt({
  className,
  disabled = false,
  isStreaming = false,
  placeholder = "Ask a follow-up",
  onStop,
  onSubmit,
}: ChatPromptProps) {
  return (
    <ChatComposer
      className={className}
      disabled={disabled}
      isStreaming={isStreaming}
      placeholder={placeholder}
      onStop={onStop}
      onSubmit={onSubmit}
    />
  )
}
