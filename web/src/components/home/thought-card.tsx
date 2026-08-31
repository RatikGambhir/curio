import { NotebookPen } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router-dom";

import {
  HOME_THOUGHT_MAX_LENGTH,
  readHomeThought,
  writeHomeThought,
} from "@/features/home/thought-storage";

export function ThoughtCard({ userId }: { userId: string }) {
  const [thought, setThought] = useState(() =>
    readHomeThought(window.localStorage, userId),
  );

  return (
    <section
      aria-labelledby="seedling-thoughts-title"
      className="flex min-h-[14.5rem] min-w-0 flex-col rounded-[1rem] border border-primary/55 bg-card p-6 shadow-sm sm:p-7"
    >
      <div className="flex items-center justify-between gap-4">
        <div className="flex min-w-0 items-center gap-3">
          <NotebookPen
            className="size-5 shrink-0 text-primary"
            aria-hidden="true"
          />
          <h2
            id="seedling-thoughts-title"
            className="truncate text-base font-semibold text-foreground"
          >
            Seedling Thoughts
          </h2>
        </div>
        <Link
          to="/notes"
          className="shrink-0 rounded-md px-1.5 py-1 text-xs font-semibold text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring"
        >
          Expand
        </Link>
      </div>

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
        className="mt-4 min-h-[7rem] w-full flex-1 resize-none bg-transparent text-base leading-7 text-foreground outline-none placeholder:text-muted-foreground/55 focus-visible:placeholder:text-muted-foreground/35 sm:text-[1.0625rem]"
      />
      <p
        id="seedling-thoughts-storage-note"
        className="mt-2 text-xs text-muted-foreground"
      >
        Saved on this device
      </p>
    </section>
  );
}
