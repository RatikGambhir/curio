import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { ChatSidebar } from "@/components/chat-sidebar";
import { ChatView } from "@/components/chat-view";
import { ChatPrompt } from "@/components/chat-prompt";

const Chat = () => {
  const messages: number[] = [];
  //TODO: Implement chat functionality with cloudlfare worker
  return (
    <SidebarProvider className="flex w-full h-screen">
      <ChatSidebar />

      <SidebarInset className="ml-[var(--sidebar-width)] w-[calc(100vw-var(--sidebar-width))]">
        <div className="relative flex flex-col items-center justify-center w-full">
          {messages.length > 0 ? (
            <div>
              <ChatView />
              <div className="absolute bottom-0 inset-x-0 mx-auto w-full max-w-3xl border-2 border-blue-500 rounded-lg p-4">
                <h1 className="text-2xl font-bold">Welcome to Curio!</h1>
                <p className="text-lg">
                  Start a conversation with our AI assistant.
                </p>
              </div>
            </div>
          ) : (
            <div className="backdrop-blur-2xl bg-white/10 border rounded-3xl shadow-2xl p-6 mb-6 absolute items-center justify-center w-full max-w-3xl mx-auto">
              <ChatPrompt />
            </div>
          )}
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Chat;
