import { ArrowRight, Plus } from "lucide-react";
import { Link } from "react-router-dom";

import type { Cultivation } from "@/components/home/home.types";
import { Badge } from "@/components/ui/badge";

/* Recent work as an index rather than a wall of cards: one ruled row per
   entry, the title in the reading serif, tags and recency as quiet marks. */
export function CultivationIndex({
  items,
}: {
  items: readonly Cultivation[];
}) {
  return (
    <section aria-labelledby="recent-cultivations-title" className="min-w-0">
      <div className="flex items-baseline justify-between gap-4">
        <h2
          id="recent-cultivations-title"
          className="font-display text-display-md text-foreground"
        >
          Recent cultivations
        </h2>
        <Link
          to="/vault"
          className="focus-ring shrink-0 rounded-sm text-sm font-medium text-primary"
        >
          View all
        </Link>
      </div>

      <ol className="mt-5 grid border-t border-border md:grid-cols-2 md:gap-x-10">
        {items.map((item, index) => {
          const Icon = item.icon;

          return (
            <li key={item.title} className="border-b border-border">
              <Link
                to="/vault"
                className="focus-ring group grid grid-cols-[1.75rem_minmax(0,1fr)_auto] items-start gap-3 rounded-sm py-4"
              >
                <span className="pt-1 font-mono text-2xs tabular-nums text-muted-foreground">
                  {String(index + 1).padStart(2, "0")}
                </span>
                <span className="min-w-0">
                  <span className="flex items-center gap-2">
                    <Icon
                      className="size-4 shrink-0 text-muted-foreground"
                      aria-hidden="true"
                    />
                    <span className="truncate font-display text-[1.1875rem] leading-snug text-foreground underline-offset-4 group-hover:underline">
                      {item.title}
                    </span>
                  </span>
                  <span className="mt-2 flex flex-wrap items-center gap-1.5">
                    {item.tags.map((tag) => (
                      <Badge key={tag} variant="tag">
                        {tag}
                      </Badge>
                    ))}
                  </span>
                </span>
                <span className="pt-1 text-xs text-muted-foreground">
                  {item.updated}
                </span>
              </Link>
            </li>
          );
        })}
        <li className="border-b border-border">
          <Link
            to="/notes"
            className="focus-ring group grid grid-cols-[1.75rem_minmax(0,1fr)_auto] items-center gap-3 rounded-sm py-4 text-muted-foreground transition-colors hover:text-foreground"
          >
            <Plus className="size-4" aria-hidden="true" />
            <span className="font-display text-[1.1875rem] italic leading-snug">
              Plant a new seed
            </span>
            <ArrowRight
              className="size-4 transition-transform duration-150 group-hover:translate-x-0.5"
              aria-hidden="true"
            />
          </Link>
        </li>
      </ol>
    </section>
  );
}
