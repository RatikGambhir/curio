import { CalendarDays } from "lucide-react";
import { Link } from "react-router-dom";

import type { UpcomingItem } from "@/components/home/home.types";
import { cn } from "@/lib/utils";

type UpcomingCardProps = {
  items: readonly UpcomingItem[];
};

export function UpcomingCard({ items }: UpcomingCardProps) {
  return (
    <aside
      aria-labelledby="upcoming-title"
      className="min-w-0 rounded-[1rem] border border-border bg-card p-6 shadow-sm sm:p-7"
    >
      <div className="flex items-center gap-3">
        <CalendarDays className="size-5 text-destructive" aria-hidden="true" />
        <h2
          id="upcoming-title"
          className="text-base font-semibold text-foreground"
        >
          Upcoming
        </h2>
      </div>

      <ul className="mt-5 space-y-2">
        {items.map((item) => (
          <li key={item.title}>
            <Link
              to="/calendar"
              className="group -mx-2 flex gap-3 rounded-md px-2 py-1.5 outline-none transition-colors hover:bg-accent/50 focus-visible:ring-2 focus-visible:ring-ring"
            >
              <span
                className={cn(
                  "mt-1.5 size-1.5 shrink-0 rounded-full",
                  item.dotClassName,
                )}
                aria-hidden="true"
              />
              <span className="min-w-0">
                <span className="block truncate text-sm font-semibold text-foreground">
                  {item.title}
                </span>
                <span className="mt-0.5 block text-xs text-muted-foreground">
                  <time dateTime={item.dateTime}>{item.time}</time>
                  <span aria-hidden="true"> · </span>
                  {item.duration}
                </span>
              </span>
            </Link>
          </li>
        ))}
      </ul>
    </aside>
  );
}
