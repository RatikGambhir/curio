import { MessageSquare } from "lucide-react";

import type { ChatMessage } from "@/mocks/chats";

type ChatViewProps = {
  messages: ChatMessage[];
};

export function ChatView({ messages }: ChatViewProps) {
  if (messages.length === 0) {
    return (
      <div className="flex h-full items-center justify-center p-8 text-center">
        <div className="space-y-3">
          <div className="flex justify-center text-muted-foreground">
            <MessageSquare className="size-12" />
          </div>
          <div className="space-y-1">
            <h3 className="font-medium text-sm">Start a conversation</h3>
            <p className="text-sm text-muted-foreground">
              Type a message below to begin chatting
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto flex h-full max-w-4xl flex-col gap-6 overflow-y-auto p-6">
      {messages.map((message) => (
        <div
          key={message.id}
          className={[
            "flex w-full",
            message.from === "user" ? "justify-end" : "justify-start",
          ].join(" ")}
        >
          <div
            className={[
              "max-w-[80%] rounded-2xl px-4 py-3 text-sm leading-6 shadow-sm",
              message.from === "user"
                ? "bg-secondary text-foreground"
                : "bg-primary/10 text-foreground",
            ].join(" ")}
          >
            {message.value}
          </div>
        </div>
      ))}
    </div>
  );
}
