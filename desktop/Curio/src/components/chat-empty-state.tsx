import { useState } from "react";
import {
  Clock3,
  Code2,
  GraduationCap,
  Lightbulb,
  Pencil,
  Plus,
  Send,
  Sparkles,
  type LucideIcon,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";

type ChatEmptyStateProps = {
  className?: string;
  disabled?: boolean;
  onSubmit?: (text: string) => void;
};

const promptShortcuts: Array<{ id: string; icon: LucideIcon; label: string }> = [
  { id: "write", icon: Pencil, label: "Write" },
  { id: "learn", icon: GraduationCap, label: "Learn" },
  { id: "code", icon: Code2, label: "Code" },
  { id: "life", icon: Lightbulb, label: "Life stuff" },
  { id: "choice", icon: Sparkles, label: "Surprise me" },
];

export function ChatEmptyState({ className, disabled = false, onSubmit }: ChatEmptyStateProps) {
  const [draft, setDraft] = useState("");

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const nextDraft = draft.trim();
    if (!nextDraft || disabled) {
      return;
    }

    onSubmit?.(nextDraft);
    setDraft("");
  };

  return (
    <div
      className={cn(
        "mx-auto flex h-full w-full max-w-[110rem] flex-col items-center justify-center px-4 pb-16 pt-8 md:px-8",
        className,
      )}
    >
      <div className="flex h-full max-h-[20rem] w-full max-w-[96rem] flex-col items-center justify-center space-y-12">
        <form
          onSubmit={handleSubmit}
          className="h-full w-full max-w-[64rem] rounded-[2.85rem] border border-primary/18 bg-[linear-gradient(180deg,rgba(255,255,255,0.42),rgba(124,153,130,0.08))] shadow-[0_18px_60px_rgba(55,72,58,0.10)] backdrop-blur-xl"
        >
          <div className="grid grid-cols-[auto_1fr_auto] items-start gap-4 px-6 py-6 md:px-11 md:py-8">
            <Button
              type="button"
              variant="ghost"
              size="icon-lg"
              aria-label="Attach files"
              className="mt-1 rounded-2xl text-muted-foreground hover:translate-y-0 hover:bg-white/45 hover:text-foreground"
            >
              <Plus className="size-8 stroke-[1.75]" />
            </Button>

            <Textarea
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              placeholder="How can I help you today?"
              rows={2}
              disabled={disabled}
              className="min-h-24 resize-none border-0 bg-transparent px-0 py-1 text-base leading-tight font-semibold text-foreground shadow-none outline-none ring-0 focus-visible:border-transparent focus-visible:ring-0 md:min-h-28 md:text-[1rem]"
            />

            <Button
              type="button"
              variant="ghost"
              size="icon-lg"
              aria-label="Prompt history"
              className="mt-1 rounded-2xl text-muted-foreground hover:translate-y-0 hover:bg-white/45 hover:text-foreground"
            >
              <Clock3 className="size-8 stroke-[1.75]" />
            </Button>
          </div>

          <div className="mx-6 h-px bg-primary/14 md:mx-11" />

          <div className="flex items-center justify-between gap-4 px-2 py-2 md:px-4 md:py-4">
            <span className="flex h-14 min-w-44 items-center rounded-[1rem] border border-primary/15 bg-white/40 px-4 text-lg font-sans font-semibold text-foreground md:min-w-52">
              Curio
            </span>

            <Button
              type="submit"
              size="icon-lg"
              aria-label="Send message"
              disabled={!draft.trim() || disabled}
              className="size-11 rounded-[1.15rem] bg-[linear-gradient(180deg,rgba(143,164,145,0.95),rgba(172,190,173,0.95))] text-primary-foreground shadow-[0_10px_24px_rgba(97,124,102,0.22)]"
            >
              <Send className="size-7 stroke-[2]" />
            </Button>
          </div>
        </form>

        <div className="flex flex-wrap items-center justify-center gap-5">
          {promptShortcuts.map((shortcut) => {
            const Icon = shortcut.icon;

            return (
              <Button
                key={shortcut.id}
                type="button"
                variant="ghost"
                className="h-auto rounded-[1.35rem] border border-border/70 bg-white/65 px-6 py-3 text-lg font-sans font-semibold text-foreground shadow-[0_10px_28px_rgba(37,42,52,0.08)] backdrop-blur-md hover:bg-white/80"
              >
                <Icon className="size-5 text-foreground" strokeWidth={1.9} aria-hidden="true" />
                <span>{shortcut.label}</span>
              </Button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
