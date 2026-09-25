import { Link } from "react-router-dom";

import type { UpcomingItem } from "@/components/home/home.types";
import { cn } from "@/lib/utils";

/* Today's agenda as a timetable: times in a mono column so they align,
   titles beside them, one coloured mark per item. */
export function UpcomingAgenda({ items }: { items: readonly UpcomingItem[] }) {
  return (
    <section aria-labelledby="upcoming-title" className="min-w-0">
      <div className="flex items-baseline justify-between gap-4">
        <h2 id="upcoming-title" className="eyebrow text-muted-foreground">
          Upcoming today
        </h2>
        <Link
          to="/calendar"
          className="focus-ring rounded-sm text-sm font-medium text-primary"
        >
          Calendar
        </Link>
      </div>

      {items.length === 0 ? (
        <p className="mt-3 border-t border-border pt-3 text-sm text-muted-foreground">
          Nothing scheduled. A clear day.
        </p>
      ) : (
        <ul className="mt-3 border-t border-border">
          {items.map((item) => (
            <li key={item.title} className="border-b border-border">
              <Link
                to="/calendar"
                className="focus-ring group grid grid-cols-[3.25rem_minmax(0,1fr)] items-baseline gap-3 rounded-sm py-3 transition-colors"
              >
                <time
                  dateTime={item.dateTime}
                  className="font-mono text-[0.8125rem] tabular-nums text-muted-foreground"
                >
                  {item.time}
                </time>
                <span className="min-w-0">
                  <span className="flex items-center gap-2">
                    <span
                      className={cn("size-1.5 shrink-0 rounded-full", item.dotClassName)}
                      aria-hidden="true"
                    />
                    <span className="truncate text-sm font-medium text-foreground underline-offset-4 group-hover:underline">
                      {item.title}
                    </span>
                  </span>
                  <span className="mt-0.5 block pl-3.5 text-xs text-muted-foreground">
                    {item.duration}
                  </span>
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
