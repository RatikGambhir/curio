import { useState } from "react";

import { ChatEmptyState } from "@/components/chat-empty-state";
import { ChatPrompt } from "@/components/chat-prompt";
import { ChatSidebar } from "@/components/chat-sidebar";
import { ChatView } from "@/components/chat-view";
import { mockChats, mockMessagesByChatId } from "@/mocks/chats";
import type { ChatListItem } from "@/mocks/chats";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";
import { useChat } from "@/hooks/useChat";

const linenNoise =
  "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 400 400' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E\")";

function buildChatTitle(text: string) {
  const normalized = text.trim().replace(/\s+/g, " ");
  if (!normalized) {
    return "New chat";
  }

  return normalized.length > 32 ? `${normalized.slice(0, 32).trimEnd()}...` : normalized;
}

function buildChatPreview(text: string) {
  const normalized = text.trim().replace(/\s+/g, " ");
  return normalized.length > 52 ? `${normalized.slice(0, 52).trimEnd()}...` : normalized;
}

function ChatPage() {
  const [chats, setChats] = useState<ChatListItem[]>(mockChats);
  const [messagesByChatId, setMessagesByChatId] = useState(mockMessagesByChatId);
  const [selectedChatId, setSelectedChatId] = useState<string | null>(
    null,
  );
  const isNewChat = selectedChatId === null;
  const { isStreaming, sendMessage } = useChat();
  const { user } = useAuthenticatedUser();

  const messages = selectedChatId ? messagesByChatId[selectedChatId] ?? [] : [];

  const handleStartNewChat = () => {
    setSelectedChatId(null);
  };

  const upsertChatMeta = (chatId: string, text: string) => {
    setChats((currentChats) => {
      const existingChat = currentChats.find((chat) => chat.id === chatId);
      const nextMeta: ChatListItem = {
        id: chatId,
        title: existingChat?.title ?? buildChatTitle(text),
        updatedAt: "Just now",
        preview: buildChatPreview(text),
      };

      const otherChats = currentChats.filter((chat) => chat.id !== chatId);
      return [nextMeta, ...otherChats];
    });
  };

  const handleCreateChat = async (text: string) => {
    if (isStreaming) {
      return;
    }

    const threadId = crypto.randomUUID();
    const userMessageId = crypto.randomUUID();
    const assistantMessageId = crypto.randomUUID();

    upsertChatMeta(threadId, text);
    setSelectedChatId(threadId);
    await sendMessage({
      chatId: threadId,
      text,
      setMessagesByChatId,
      userId: user?.id,
      userMessageId,
      threadId,
      assistantMessageId,
    });
  };

  const handleSendMessage = async (text: string) => {
    if (!selectedChatId || isStreaming) {
      return;
    }

    upsertChatMeta(selectedChatId, text);
    await sendMessage({
      chatId: selectedChatId,
      text,
      setMessagesByChatId,
      userId: user?.id,
      threadId: selectedChatId,
    });
  };

  return (
    <SidebarProvider className="h-screen w-full font-sans">
      <ChatSidebar
        chats={chats}
        selectedChatId={selectedChatId}
        isNewChat={isNewChat}
        onSelectChat={setSelectedChatId}
        onStartNewChat={handleStartNewChat}
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
            {!isNewChat ? (
              <div className="mx-auto flex h-full w-full max-w-6xl flex-col gap-4 px-2 md:px-3">
                <div className="min-h-0 flex-1 overflow-hidden rounded-[1.75rem] border border-border bg-background">
                  <ChatView messages={messages} />
                </div>
                <ChatPrompt disabled={isStreaming} onSubmit={handleSendMessage} />
              </div>
            ) : (
              <ChatEmptyState disabled={isStreaming} onSubmit={handleCreateChat} />
            )}
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

export default ChatPage;
