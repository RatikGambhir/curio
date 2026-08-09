import { Clock3, Plus, Send } from "lucide-react";
import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";

type ChatPromptProps = {
  className?: string;
  disabled?: boolean;
  placeholder?: string;
  onSubmit?: (text: string) => void;
};

export function ChatPrompt({
  className,
  disabled = false,
  placeholder = "How can I help you today?",
  onSubmit,
}: ChatPromptProps) {
  const [text, setText] = useState("");

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const nextText = text.trim();
    if (!nextText || disabled) {
      return;
    }

    onSubmit?.(nextText);
    setText("");
  };

  return (
    <form
      onSubmit={handleSubmit}
      className={cn(
        "w-full rounded-[2rem] border border-primary/25 bg-primary/10 p-4 shadow-xl backdrop-blur-2xl md:p-5",
        className,
      )}
    >
      <div className="w-full space-y-4">
        <div className="grid grid-cols-[auto_1fr_auto] items-start gap-2 md:gap-3">
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="mt-1 size-9 rounded-lg text-muted-foreground hover:bg-background/40 hover:text-primary"
            aria-label="Attach files"
          >
            <Plus className="size-5" />
          </Button>

          <Textarea
            value={text}
            onChange={(event) => setText(event.target.value)}
            placeholder={placeholder}
            disabled={disabled}
            className="min-h-[64px] max-h-44 resize-none border-0 bg-transparent px-0 py-1 text-base leading-snug font-medium placeholder:text-sm placeholder:text-foreground/45 shadow-none outline-none ring-0 focus-visible:border-transparent focus-visible:ring-0 md:min-h-[72px] md:text-lg md:placeholder:text-base"
          />

          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="mt-1 size-9 rounded-lg text-muted-foreground hover:bg-background/40 hover:text-primary"
            aria-label="Prompt history"
          >
            <Clock3 className="size-5" />
          </Button>
        </div>

        <div className="h-px w-full bg-primary/20" />

        <div className="flex items-center justify-between gap-3">
          <span className="flex h-10 items-center rounded-lg border border-primary/25 bg-background/70 px-3 text-sm font-semibold text-foreground md:min-w-44">
            Curio
          </span>

          <Button
            type="submit"
            disabled={!text.trim() || disabled}
            className="size-10 rounded-lg bg-gradient-to-br from-primary to-accent text-primary-foreground shadow-md hover:from-primary/90 hover:to-accent/90"
          >
            <Send className="size-4.5" />
          </Button>
        </div>
      </div>
    </form>
  );
}
