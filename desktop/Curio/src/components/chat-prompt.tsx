import { Clock3, Plus, Send } from "lucide-react";
import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";

type ChatPromptProps = {
  className?: string;
  placeholder?: string;
};

export function ChatPrompt({
  className,
  placeholder = "How can I help you today?",
}: ChatPromptProps) {
  const [text, setText] = useState("");
  const [model, setModel] = useState("sonnet-4.5");

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!text.trim()) {
      return;
    }

    setText("");
  };

  return (
    <form
      onSubmit={handleSubmit}
      className={cn(
        "w-full rounded-[2.5rem] border border-primary/25 bg-primary/10 p-5 shadow-xl backdrop-blur-2xl md:p-7",
        className,
      )}
    >
      <div className="w-full space-y-5">
        <div className="grid grid-cols-[auto_1fr_auto] items-start gap-3 md:gap-4">
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="mt-1 size-10 rounded-xl text-muted-foreground hover:bg-background/40 hover:text-primary"
            aria-label="Attach files"
          >
            <Plus className="size-6" />
          </Button>

          <Textarea
            value={text}
            onChange={(event) => setText(event.target.value)}
            placeholder={placeholder}
            className="min-h-[88px] max-h-56 resize-none border-0 bg-transparent px-0 py-1 text-xl leading-tight font-medium placeholder:text-foreground/45 shadow-none outline-none ring-0 focus-visible:border-transparent focus-visible:ring-0 md:text-3xl"
          />

          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="mt-1 size-10 rounded-xl text-muted-foreground hover:bg-background/40 hover:text-primary"
            aria-label="Prompt history"
          >
            <Clock3 className="size-6" />
          </Button>
        </div>

        <div className="h-px w-full bg-primary/20" />

        <div className="flex items-center justify-between gap-3">
          <Select value={model} onValueChange={setModel}>
            <SelectTrigger className="h-12 rounded-xl border-primary/25 bg-background/70 px-4 text-base font-semibold text-foreground shadow-none md:min-w-48">
              <SelectValue placeholder="Select model" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="sonnet-4.5">Sonnet 4.5</SelectItem>
              <SelectItem value="haiku-3.5">Haiku 3.5</SelectItem>
              <SelectItem value="opus-4">Opus 4</SelectItem>
            </SelectContent>
          </Select>

          <Button
            type="submit"
            disabled={!text.trim()}
            className="size-12 rounded-xl bg-gradient-to-br from-primary to-accent text-primary-foreground shadow-md hover:from-primary/90 hover:to-accent/90"
          >
            <Send className="size-5" />
          </Button>
        </div>
      </div>
    </form>
  );
}
