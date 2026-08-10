import { useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import {
  ArrowUp,
  Code2,
  GraduationCap,
  Lightbulb,
  Paperclip,
  Pencil,
  Sparkles,
  type LucideIcon,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";

type ChatComposerProps = {
  className?: string;
  compact?: boolean;
  disabled?: boolean;
  placeholder?: string;
  showShortcuts?: boolean;
  onSubmit?: (text: string) => void;
};

const promptShortcuts: Array<{
  icon: LucideIcon;
  label: string;
  prompt: string;
}> = [
  { icon: Pencil, label: "Write", prompt: "Help me write " },
  { icon: GraduationCap, label: "Learn", prompt: "Teach me about " },
  { icon: Code2, label: "Code", prompt: "Help me build " },
  { icon: Lightbulb, label: "Think", prompt: "Help me think through " },
  {
    icon: Sparkles,
    label: "Surprise me",
    prompt: "Give me something interesting to explore",
  },
];

export function ChatComposer({
  className,
  compact = false,
  disabled = false,
  placeholder = "Ask anything",
  showShortcuts = false,
  onSubmit,
}: ChatComposerProps) {
  const [text, setText] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const submitText = () => {
    const nextText = text.trim();
    if (!nextText || disabled) {
      return;
    }

    onSubmit?.(nextText);
    setText("");
  };

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    submitText();
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      event.currentTarget.form?.requestSubmit();
    }
  };

  const selectShortcut = (prompt: string) => {
    setText(prompt);
    requestAnimationFrame(() => textareaRef.current?.focus());
  };

  return (
    <div className={cn("w-full", className)}>
      <form
        onSubmit={handleSubmit}
        className="w-full rounded-[1.35rem] border border-foreground/10 bg-card/90 p-2 shadow-[0_1px_2px_rgba(24,31,27,0.05),0_12px_32px_rgba(24,31,27,0.055)] transition-[border-color,box-shadow,background-color] duration-200 focus-within:border-primary/45 focus-within:bg-card focus-within:shadow-[0_1px_2px_rgba(24,31,27,0.05),0_16px_40px_rgba(24,31,27,0.075)]"
      >
        <Textarea
          ref={textareaRef}
          value={text}
          onChange={(event) => setText(event.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          rows={compact ? 2 : 3}
          disabled={disabled}
          aria-label="Message Curio"
          className={cn(
            "max-h-48 resize-none border-0 bg-transparent px-3 text-[15px] font-normal leading-6 tracking-[-0.006em] text-foreground shadow-none outline-none ring-0 placeholder:font-normal placeholder:text-muted-foreground/65 focus-visible:border-transparent focus-visible:ring-0 md:text-[15px]",
            compact ? "min-h-14 py-2" : "min-h-24 py-3",
          )}
        />

        <div className="flex items-center justify-between gap-3 px-1 pb-1 pt-1">
          <div className="flex min-w-0 items-center gap-1.5">
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label="Attach files"
              className="size-8 rounded-lg text-muted-foreground shadow-none hover:translate-y-0 hover:bg-muted hover:text-foreground"
            >
              <Paperclip className="size-4 stroke-[1.7]" />
            </Button>
            <span className="truncate text-xs font-normal text-muted-foreground">Curio</span>
          </div>

          <div className="flex items-center gap-2">
            <span className="hidden text-[11px] font-normal text-muted-foreground/60 sm:inline">
              Enter to send
            </span>
            <Button
              type="submit"
              size="icon-sm"
              aria-label="Send message"
              disabled={!text.trim() || disabled}
              className="size-8 rounded-lg bg-foreground text-background shadow-none hover:translate-y-0 hover:bg-foreground/88 disabled:opacity-30"
            >
              <ArrowUp className="size-4 stroke-2" />
            </Button>
          </div>
        </div>
      </form>

      {showShortcuts ? (
        <div className="mt-4 flex flex-wrap items-center justify-center gap-2">
          {promptShortcuts.map(({ icon: Icon, label, prompt }) => (
            <Button
              key={label}
              type="button"
              variant="ghost"
              onClick={() => selectShortcut(prompt)}
              className="h-9 rounded-full border border-foreground/10 bg-background/75 px-3.5 text-[13px] font-normal text-muted-foreground shadow-none backdrop-blur-sm hover:translate-y-0 hover:border-foreground/15 hover:bg-card hover:text-foreground"
            >
              <Icon className="size-3.5 stroke-[1.7]" aria-hidden="true" />
              <span>{label}</span>
            </Button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
