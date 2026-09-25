import { ArrowUpRight } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router-dom";

import {
  HOME_THOUGHT_MAX_LENGTH,
  readHomeThought,
  writeHomeThought,
} from "@/features/home/thought-storage";

/* Quick capture, drawn as a page from the notebook: ruled lines the text sits
   on and a margin rule in the theme's ink. The line height and the rule gap
   are the same value so every line of writing lands on a rule. */
export function ThoughtPad({ userId }: { userId: string }) {
  const [thought, setThought] = useState(() =>
    readHomeThought(window.localStorage, userId),
  );

  return (
    <section aria-labelledby="seedling-thoughts-title" className="min-w-0">
      <div className="flex items-baseline justify-between gap-4">
        <h2 id="seedling-thoughts-title" className="eyebrow text-muted-foreground">
          Seedling thoughts
        </h2>
        <Link
          to="/notes"
          className="focus-ring group inline-flex items-center gap-1 rounded-sm text-sm font-medium text-primary"
        >
          Open notes
          <ArrowUpRight
            className="size-3.5 transition-transform duration-150 group-hover:-translate-y-px group-hover:translate-x-px"
            aria-hidden="true"
          />
        </Link>
      </div>

      <div className="relative mt-3 overflow-hidden rounded-lg border border-border bg-card shadow-2xs transition-[border-color,box-shadow] focus-within:border-ring/60 focus-within:shadow-sm">
        <span
          aria-hidden="true"
          className="pointer-events-none absolute inset-y-0 left-10 w-px bg-accent-brand/40 sm:left-12"
        />
        <textarea
          aria-labelledby="seedling-thoughts-title"
          aria-describedby="seedling-thoughts-storage-note"
          placeholder="Jot down a quick thought…"
          value={thought}
          onChange={(event) => {
            const nextThought = event.target.value;
            setThought(nextThought);
            writeHomeThought(window.localStorage, userId, nextThought);
          }}
          maxLength={HOME_THOUGHT_MAX_LENGTH}
          rows={6}
          className="ruled-paper block min-h-[13.5rem] w-full resize-none bg-transparent pb-4 pl-14 pr-5 pt-2.5 [--rule-gap:2rem] font-display text-[1.1875rem] leading-8 text-foreground outline-none placeholder:text-muted-foreground sm:pl-16"
        />
      </div>

      <div
        id="seedling-thoughts-storage-note"
        className="mt-2 flex items-center justify-between gap-4 text-xs text-muted-foreground"
      >
        <span>Saved on this device as you type</span>
        <span className="font-mono tabular-nums">
          {thought.length}/{HOME_THOUGHT_MAX_LENGTH}
        </span>
      </div>
    </section>
  );
}
