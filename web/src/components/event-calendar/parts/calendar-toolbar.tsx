"use client";

import { ChevronLeft, ChevronRight } from "lucide-react";
import { format } from "date-fns";
import { Button } from "@/components/ui/button";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { cn } from "@/lib/utils";
import { useCalendar } from "../hooks/use-calendar-context";
import type { CalendarView } from "../types";

const VIEW_LABELS: Record<CalendarView, string> = {
  month: "Month",
  week: "Week",
  day: "Day",
  agenda: "Agenda",
};

function periodLabel(
  view: CalendarView,
  focusDate: Date,
  range: { start: Date; end: Date },
): string {
  if (view === "month") return format(focusDate, "MMMM yyyy");
  if (view === "day") return format(focusDate, "EEEE, MMMM d, yyyy");
  // week + agenda → a range. Same year: show the year once, on the end
  // ("Jun 1 – Jun 7, 2026"). Cross-year: show it on both ("Dec 28, 2025 – Jan 3, 2026").
  const sameYear = range.start.getFullYear() === range.end.getFullYear();
  return `${format(range.start, sameYear ? "MMM d" : "MMM d, yyyy")} – ${format(
    range.end,
    "MMM d, yyyy",
  )}`;
}

/** Toolbar (Tier B): period nav + label + view switch. */
export function CalendarToolbar({ className }: { className?: string }) {
  const {
    view,
    focusDate,
    visibleRange,
    availableViews,
    setView,
    next,
    prev,
    goToToday,
  } = useCalendar();

  return (
    <div
      className={cn(
        "flex min-w-0 flex-col items-stretch gap-2 border-b border-border px-3 py-2 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between sm:px-5",
        className,
      )}
    >
      <div className="flex min-w-0 items-center gap-1">
        <Button variant="outline" size="sm" onClick={goToToday}>
          Today
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Previous period"
          onClick={prev}
        >
          <ChevronLeft className="size-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label="Next period"
          onClick={next}
        >
          <ChevronRight className="size-4" />
        </Button>
        <span
          aria-live="polite"
          className="ml-2 min-w-0 truncate font-display text-[1.125rem] leading-none text-foreground"
        >
          {periodLabel(view, focusDate, visibleRange)}
        </span>
      </div>

      {availableViews.length > 1 ? (
        <SegmentedControl
          label="Calendar view"
          name="event-calendar-view"
          options={availableViews.map((v) => ({ value: v, label: VIEW_LABELS[v] }))}
          value={view}
          onValueChange={setView}
          className="self-start sm:self-auto"
        />
      ) : null}
    </div>
  );
}
