import { ChatComposer } from "@/components/chat-composer"

type ChatPromptProps = {
  className?: string
  placeholder?: string
  onSubmit?: (text: string) => void
}

export function ChatPrompt({
  className,
  placeholder = "Ask a follow-up",
  onSubmit,
}: ChatPromptProps) {
  return (
    <ChatComposer
      className={className}
      compact
      placeholder={placeholder}
      onSubmit={onSubmit}
    />
  )
}
