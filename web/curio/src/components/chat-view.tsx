import {
  Conversation,
  ConversationContent,
  ConversationEmptyState,
} from "./ai-elements/conversation";
import { MessageSquare } from "lucide-react";
import { Message, MessageContent } from "./ai-elements/message";

const messages: { key: string; value: string; from: "user" | "assistant" }[] = [
  {
    key: "1",
    value: "Hello, how are you?",
    from: "user",
  },
  {
    key: "2",
    value: "I'm good, thank you! How can I assist you today?",
    from: "assistant",
  },
];

const mockMessages = [];

export function ChatView() {
  //const { messages } = useChat();
  return (
    <div className="mx-auto p-4 max-h-250 w-full">
      <div className="flex flex-col">
        <Conversation>
          <ConversationContent>
            {mockMessages.length > 0 ? (
              messages.map(({ key, value, from }) => (
                <Message key={key} from={from}>
                  <MessageContent>{value}</MessageContent>
                </Message>
              ))
            ) : (
              <ConversationEmptyState
                icon={<MessageSquare className="size-12" />}
                title="Start a conversation"
                description="Type a message below to begin chatting"
              />
            )}
          </ConversationContent>
        </Conversation>
      </div>
    </div>
  );
}
