import { AlertTriangle } from "lucide-react";
import { useEffect, useState } from "react";

import { cn } from "@/lib/utils";
import type { ChatMessage } from "@/mocks/chats";

type BaseMessageProps = {
  children: React.ReactNode;
  className?: string;
};

const thinkingTerms = ["Thinking", "Reading context", "Drafting", "Checking details"];

function MessageRow({ children, className }: BaseMessageProps) {
  return <div className={cn("flex w-full", className)}>{children}</div>;
}

function MessageBubble({ children, className }: BaseMessageProps) {
  return (
    <div
      className={cn(
        "max-w-[80%] rounded-2xl px-4 py-3 text-sm leading-6 shadow-sm",
        className,
      )}
    >
      {children}
    </div>
  );
}

export function UserMessage({ value }: Pick<ChatMessage, "value">) {
  return (
    <MessageRow className="justify-end">
      <MessageBubble className="bg-secondary text-foreground">{value}</MessageBubble>
    </MessageRow>
  );
}

export function AssistantMessage({ value }: Pick<ChatMessage, "value">) {
  return <AssistantMessageContent value={value} />;
}

function ThinkingIndicator() {
  const [termIndex, setTermIndex] = useState(0);

  useEffect(() => {
    const interval = window.setInterval(
      () => setTermIndex((current) => (current + 1) % thinkingTerms.length),
      1800,
    );
    return () => window.clearInterval(interval);
  }, []);

  return <span className="text-muted-foreground">{thinkingTerms[termIndex]}…</span>;
}

function AssistantMessageContent({
  value,
  status,
}: Pick<ChatMessage, "value" | "status">) {
  const isError = status === "error";

  return (
    <MessageRow className="justify-start">
      <MessageBubble
        className={
          isError
            ? "border border-red-300 bg-red-50 text-red-900"
            : "whitespace-pre-wrap bg-primary/10 text-foreground"
        }
      >
        {isError ? (
          <span className="flex items-start gap-2" role="alert">
            <AlertTriangle className="mt-0.5 size-4 shrink-0 text-red-600" />
            <span>{value}</span>
          </span>
        ) : value.trim() ? (
          value
        ) : (
          <ThinkingIndicator />
        )}
      </MessageBubble>
    </MessageRow>
  );
}

export function ChatMessageItem(message: ChatMessage) {
  if (message.from === "user") {
    return <UserMessage value={message.value} />;
  }

  return <AssistantMessageContent value={message.value} status={message.status} />;
}
