import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useState } from "react";
import type { Dispatch, SetStateAction } from "react";

import type { ChatMessage } from "@/mocks/chats";

type MessagesByChatId = Record<string, ChatMessage[]>;

type StreamParams = {
  chatId: string;
  text: string;
  setMessagesByChatId: Dispatch<SetStateAction<MessagesByChatId>>;
  userId?: string | null;
  userMessageId?: string;
  threadId?: string;
  assistantMessageId?: string;
};

type StreamTokenPayload = {
  chatId: string;
  assistantMessageId: string;
  token: string;
};

type StreamErrorPayload = {
  chatId: string;
  assistantMessageId: string;
  error: string;
};

function createMessage(
  from: ChatMessage["from"],
  value: string,
  id: string = crypto.randomUUID(),
): ChatMessage {
  return {
    id,
    from,
    value,
  };
}

function appendTokenToAssistantMessage(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  token: string,
) {
  setMessagesByChatId((currentMessages) => {
    const chatMessages = currentMessages[chatId] ?? [];
    const nextChatMessages = chatMessages.map((message) => {
      if (message.id !== assistantMessageId) {
        return message;
      }

      return {
        ...message,
        value: message.value + token,
      };
    });

    return {
      ...currentMessages,
      [chatId]: nextChatMessages,
    };
  });
}

function setAssistantMessageValue(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  value: string,
) {
  setMessagesByChatId((currentMessages) => {
    const chatMessages = currentMessages[chatId] ?? [];
    const nextChatMessages = chatMessages.map((message) => {
      if (message.id !== assistantMessageId) {
        return message;
      }

      return {
        ...message,
        value,
      };
    });

    return {
      ...currentMessages,
      [chatId]: nextChatMessages,
    };
  });
}

async function registerStreamListeners({
  chatId,
  assistantMessageId,
  setMessagesByChatId,
  onError,
}: {
  chatId: string;
  assistantMessageId: string;
  setMessagesByChatId: StreamParams["setMessagesByChatId"];
  onError: (error: string) => void;
}) {
  const unlistenToken = await listen<StreamTokenPayload>(
    "chat-stream-token",
    ({ payload }) => {
      if (payload.chatId !== chatId || payload.assistantMessageId !== assistantMessageId) {
        return;
      }

      appendTokenToAssistantMessage(
        setMessagesByChatId,
        chatId,
        assistantMessageId,
        payload.token,
      );
    },
  );

  const unlistenError = await listen<StreamErrorPayload>(
    "chat-stream-error",
    ({ payload }) => {
      if (payload.chatId !== chatId || payload.assistantMessageId !== assistantMessageId) {
        return;
      }

      onError(payload.error);
      setAssistantMessageValue(
        setMessagesByChatId,
        chatId,
        assistantMessageId,
        "Something went wrong while streaming the response.",
      );
    },
  );

  return () => {
    unlistenToken();
    unlistenError();
  };
}

function unlistenSafely(unlisten: UnlistenFn | null) {
  if (unlisten) {
    unlisten();
  }
}

export function useChat() {
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamError, setStreamError] = useState<string | null>(null);

  const sendMessage = useCallback(
    async ({
      chatId,
      text,
      setMessagesByChatId,
      userId,
      userMessageId,
      threadId,
      assistantMessageId,
    }: StreamParams) => {
      setStreamError(null);
      setIsStreaming(true);

      const userMessage = createMessage("user", text, userMessageId);
      const assistantMessage = createMessage("assistant", "", assistantMessageId);

      setMessagesByChatId((currentMessages) => ({
        ...currentMessages,
        [chatId]: [...(currentMessages[chatId] ?? []), userMessage, assistantMessage],
      }));

      let unlisten: UnlistenFn | null = null;

      try {
        unlisten = await registerStreamListeners({
          chatId,
          assistantMessageId: assistantMessage.id,
          setMessagesByChatId,
          onError: setStreamError,
        });

        await invoke("stream_chat", {
          request: {
            chatId,
            text,
            userId,
            threadId,
            assistantMessageId: assistantMessage.id,
          },
        });
      } catch (error) {
        const nextError = error instanceof Error ? error.message : String(error);
        setStreamError(nextError);
        setAssistantMessageValue(
          setMessagesByChatId,
          chatId,
          assistantMessage.id,
          "Something went wrong while streaming the response.",
        );
      } finally {
        unlistenSafely(unlisten);
        setIsStreaming(false);
      }
    },
    [],
  );

  return {
    isStreaming,
    sendMessage,
    streamError,
  };
}
