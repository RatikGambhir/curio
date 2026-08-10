import { useState } from "react";

import { ChatEmptyState } from "@/components/chat-empty-state";
import { ChatPrompt } from "@/components/chat-prompt";
import { ChatSidebar } from "@/components/chat-sidebar";
import { PageHeader } from "@/components/page-header";
import { ChatView } from "@/components/chat-view";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { useChat } from "@/hooks/useChat";
import { mockChats, mockMessagesByChatId } from "@/mocks/chats";
import type { ChatListItem } from "@/mocks/chats";

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
  const [selectedChatId, setSelectedChatId] = useState<string | null>(null);
  const isNewChat = selectedChatId === null;
  const { isStreaming, sendMessage } = useChat();
  const messages = selectedChatId ? messagesByChatId[selectedChatId] ?? [] : [];

  const upsertChatMeta = (chatId: string, text: string) => {
    setChats((currentChats) => {
      const existingChat = currentChats.find((chat) => chat.id === chatId);
      const nextMeta: ChatListItem = {
        id: chatId,
        title: existingChat?.title ?? buildChatTitle(text),
        updatedAt: "Just now",
        preview: buildChatPreview(text),
      };

      return [nextMeta, ...currentChats.filter((chat) => chat.id !== chatId)];
    });
  };

  const handleCreateChat = async (text: string) => {
    if (isStreaming) {
      return;
    }

    const conversationId = crypto.randomUUID();
    upsertChatMeta(conversationId, text);
    setSelectedChatId(conversationId);
    await sendMessage({
      chatId: conversationId,
      text,
      setMessagesByChatId,
    });
  };

  const handleSendMessage = async (text: string) => {
    if (!selectedChatId || isStreaming) {
      return;
    }

    upsertChatMeta(selectedChatId, text);
    await sendMessage({ chatId: selectedChatId, text, setMessagesByChatId });
  };

  return (
    <SidebarProvider className="h-screen w-full font-sans">
      <ChatSidebar
        chats={chats}
        selectedChatId={selectedChatId}
        isNewChat={isNewChat}
        onSelectChat={setSelectedChatId}
        onStartNewChat={() => setSelectedChatId(null)}
      />
      <SidebarInset className="bg-background">
        <div className="flex h-full w-full flex-col bg-background">
          <PageHeader />
          <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden bg-background px-4 py-5 md:px-8 md:py-6">
          <div className="relative z-10 flex h-full w-full flex-col">
            {isNewChat ? (
              <ChatEmptyState disabled={isStreaming} onSubmit={handleCreateChat} />
            ) : (
              <div className="mx-auto flex h-full w-full max-w-4xl flex-col gap-3">
                <div className="min-h-0 flex-1 overflow-hidden">
                  <ChatView messages={messages} />
                </div>
                <ChatPrompt disabled={isStreaming} onSubmit={handleSendMessage} />
              </div>
            )}
          </div>
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

export default ChatPage;
