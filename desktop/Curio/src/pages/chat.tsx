import { useMemo, useState } from "react";

import { ChatEmptyState } from "@/components/chat-empty-state";
import { ChatPrompt } from "@/components/chat-prompt";
import { ChatSidebar } from "@/components/chat-sidebar";
import { ChatView } from "@/components/chat-view";
import { mockChats, mockMessagesByChatId } from "@/mocks/chats";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";

const linenNoise =
  "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 400 400' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E\")";

function ChatPage() {
  const chats = useMemo(() => mockChats, []);
  const [selectedChatId, setSelectedChatId] = useState<string | null>(
    chats[0]?.id ?? null,
  );

  const messages = selectedChatId ? mockMessagesByChatId[selectedChatId] ?? [] : [];

  return (
    <SidebarProvider className="h-screen w-full font-sans">
      <ChatSidebar
        chats={chats}
        selectedChatId={selectedChatId}
        onSelectChat={setSelectedChatId}
        className="bg-background"
      />
      <SidebarInset className="bg-background">
        <div className="relative flex h-full w-full flex-col overflow-hidden bg-background px-4 py-6 md:px-8">
          <div
            aria-hidden="true"
            className="pointer-events-none absolute inset-0 opacity-[0.03]"
            style={{ backgroundImage: linenNoise, backgroundRepeat: "repeat" }}
          />
          <div className="relative z-10 flex h-full w-full flex-col">
            {messages.length > 0 ? (
              <div className="mx-auto flex h-full w-full max-w-4xl flex-col gap-4">
                <div className="min-h-0 flex-1 overflow-y-auto rounded-xl border border-border bg-card">
                  <ChatView messages={messages} />
                </div>
                <ChatPrompt />
              </div>
            ) : (
              <ChatEmptyState />
            )}
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

export default ChatPage;
