import {
  PromptInput,
  PromptInputHeader,
  PromptInputAttachments,
  PromptInputAttachment,
  PromptInputBody,
  PromptInputTextarea,
  PromptInputFooter,
  PromptInputButton,
} from "@/components/ai-elements/prompt-input";

import type { PromptInputMessage } from "@/components/ai-elements/prompt-input";

import { type FormEvent, useRef, useState } from "react";

export function ChatPrompt() {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [text, setText] = useState("");

  //const { messages, status, sendMessage } = useChat();

  const askQuestion = async (
    message: PromptInputMessage,
    event: FormEvent<HTMLFormElement>,
  ) => {
    event.preventDefault();
    if (!message.text.trim() && message.files.length === 0) {
      return;
    }
    setText("");
  };

  return (
    <PromptInput
      onSubmit={askQuestion}
      globalDrop
      multiple
    >
      <PromptInputHeader>
        <PromptInputAttachments>
          {(attachment) => <PromptInputAttachment data={attachment} />}
        </PromptInputAttachments>
      </PromptInputHeader>
      <PromptInputBody>
        <PromptInputTextarea
          onChange={(e) => setText(e.target.value)}
          ref={textareaRef}
          value={text}
        />
      </PromptInputBody>
      <PromptInputFooter>
        <PromptInputButton type="submit" disabled={!text}>
          Send
        </PromptInputButton>
      </PromptInputFooter>
    </PromptInput>
  );
}
