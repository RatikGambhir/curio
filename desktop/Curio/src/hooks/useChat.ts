import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useState } from "react";
import type { Dispatch, SetStateAction } from "react";

import type { ChatMessage } from "@/mocks/chats";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";

type MessagesByChatId = Record<string, ChatMessage[]>;

type StreamParams = {
  chatId: string;
  text: string;
  setMessagesByChatId: Dispatch<SetStateAction<MessagesByChatId>>;
  userMessageId?: string;
  assistantMessageId?: string;
};

type StreamTokenPayload = {
  conversationId: string;
  messageId: string;
  token: string;
};

type StreamErrorPayload = {
  conversationId: string;
  messageId: string;
  code: string;
  message: string;
};

type StreamDonePayload = {
  conversationId: string;
  messageId: string;
  responseId: string;
};

function createMessage(
  from: ChatMessage["from"],
  value: string,
  id: string = crypto.randomUUID(),
): ChatMessage {
  return { id, from, value };
}

function updateAssistantMessage(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  update: (message: ChatMessage) => ChatMessage,
) {
  setMessagesByChatId((currentMessages) => ({
    ...currentMessages,
    [chatId]: (currentMessages[chatId] ?? []).map((message) =>
      message.id === assistantMessageId ? update(message) : message,
    ),
  }));
}

function appendAssistantToken(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  token: string,
) {
  updateAssistantMessage(setMessagesByChatId, chatId, assistantMessageId, (message) => ({
    ...message,
    value: message.value + token,
  }));
}

function setAssistantError(
  setMessagesByChatId: StreamParams["setMessagesByChatId"],
  chatId: string,
  assistantMessageId: string,
  message: string,
) {
  updateAssistantMessage(setMessagesByChatId, chatId, assistantMessageId, (current) => ({
    ...current,
    value: message,
    status: "error",
  }));
}

function isCorrelated(
  payload: { conversationId: string; messageId: string },
  chatId: string,
  assistantMessageId: string,
) {
  return payload.conversationId === chatId && payload.messageId === assistantMessageId;
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
  onError: (message: string) => void;
}) {
  const unlisteners: UnlistenFn[] = [];

  try {
    unlisteners.push(
      await listen<StreamTokenPayload>("chat-stream-token", ({ payload }) => {
        if (!isCorrelated(payload, chatId, assistantMessageId)) {
          return;
        }

        appendAssistantToken(
          setMessagesByChatId,
          chatId,
          assistantMessageId,
          payload.token,
        );
      }),
    );

    unlisteners.push(
      await listen<StreamErrorPayload>("chat-stream-error", ({ payload }) => {
        if (!isCorrelated(payload, chatId, assistantMessageId)) {
          return;
        }

        onError(payload.message);
        setAssistantError(
          setMessagesByChatId,
          chatId,
          assistantMessageId,
          payload.message,
        );
      }),
    );

    unlisteners.push(
      await listen<StreamDonePayload>("chat-stream-done", ({ payload }) => {
        if (!isCorrelated(payload, chatId, assistantMessageId)) {
          return;
        }
      }),
    );
  } catch (error) {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    throw error;
  }

  return () => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  };
}

export function useChat() {
  const userId = useAuthenticatedUser().user?.id;
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamError, setStreamError] = useState<string | null>(null);

  const sendMessage = useCallback(
    async ({
      chatId,
      text,
      setMessagesByChatId,
      userMessageId,
      assistantMessageId,
    }: StreamParams) => {
      if (!userId) {
        setStreamError("Sign in with an email before starting a chat.");
        return;
      }

      setStreamError(null);
      setIsStreaming(true);

      const userMessage = createMessage("user", text, userMessageId);
      const assistantMessage = createMessage("assistant", "", assistantMessageId);
      setMessagesByChatId((currentMessages) => ({
        ...currentMessages,
        [chatId]: [...(currentMessages[chatId] ?? []), userMessage, assistantMessage],
      }));

      let receivedStreamError = false;
      let removeListeners: (() => void) | null = null;

      try {
        removeListeners = await registerStreamListeners({
          chatId,
          assistantMessageId: assistantMessage.id,
          setMessagesByChatId,
          onError: (message) => {
            receivedStreamError = true;
            setStreamError(message);
          },
        });

        await invoke("stream_chat", {
          request: {
            userId,
            conversationId: chatId,
            userMessageId: userMessage.id,
            assistantMessageId: assistantMessage.id,
            prompt: text,
          },
        });
      } catch (error) {
        if (!receivedStreamError) {
          const message = error instanceof Error ? error.message : String(error);
          setStreamError(message);
          setAssistantError(setMessagesByChatId, chatId, assistantMessage.id, message);
        }
      } finally {
        removeListeners?.();
        setIsStreaming(false);
      }
    },
    [userId],
  );

  return { isStreaming, sendMessage, streamError };
}
