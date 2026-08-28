import { useEffect, useState } from "react"
import {
  addDays,
  addMonths,
  endOfWeek,
  format,
  isSameDay,
  isSameMonth,
  startOfMonth,
  startOfWeek,
} from "date-fns"
import { CalendarDays, ChevronLeft, ChevronRight } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"
import { cn } from "@/lib/utils"

import { useCalendar } from "./calendar-context"
import type { CalendarView } from "./types"

const views: { value: CalendarView; label: string; hotkey: string }[] = [
  { value: "month", label: "Month", hotkey: "M" },
  { value: "week", label: "Week", hotkey: "W" },
  { value: "day", label: "Day", hotkey: "D" },
  { value: "agenda", label: "Agenda", hotkey: "A" },
]

function periodTitle(view: CalendarView, anchor: Date): string {
  if (view === "month") {
    return format(anchor, "MMMM yyyy")
  }

  if (view === "week") {
    const start = startOfWeek(anchor)
    const end = endOfWeek(anchor)
    const sameMonth = isSameMonth(start, end)
    return `${format(start, "d MMM")} – ${format(end, sameMonth ? "d MMM yyyy" : "d MMM yyyy")}`
  }

  if (view === "day") {
    return format(anchor, "EEEE d MMMM yyyy")
  }

  return `${format(anchor, "d MMM")} – ${format(addDays(anchor, 29), "d MMM yyyy")}`
}

function MiniNav() {
  const { anchorDate, now, setAnchorDate } = useCalendar()
  const [visibleMonth, setVisibleMonth] = useState(() => startOfMonth(anchorDate))

  useEffect(() => {
    setVisibleMonth(startOfMonth(anchorDate))
  }, [anchorDate])

  const gridStart = startOfWeek(startOfMonth(visibleMonth))
  const days = Array.from({ length: 42 }, (_, index) => addDays(gridStart, index))

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="ghost" size="icon" aria-label="Jump to date">
          <CalendarDays className="size-4" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-auto p-3">
        <div className="mb-2 flex items-center justify-between gap-2">
          <Button
            variant="ghost"
            size="icon"
            aria-label="Previous month"
            onClick={() => setVisibleMonth((current) => addMonths(current, -1))}
          >
            <ChevronLeft className="size-4" />
          </Button>
          <span className="text-sm font-medium">
            {format(visibleMonth, "MMMM yyyy")}
          </span>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Next month"
            onClick={() => setVisibleMonth((current) => addMonths(current, 1))}
          >
            <ChevronRight className="size-4" />
          </Button>
        </div>

        <div className="grid grid-cols-7 gap-0.5">
          {days.slice(0, 7).map((day) => (
            <div
              key={`label-${day.toISOString()}`}
              className="pb-1 text-center text-[10px] text-muted-foreground"
            >
              {format(day, "EEEEE")}
            </div>
          ))}
          {days.map((day) => (
            <button
              key={day.toISOString()}
              type="button"
              onClick={() => setAnchorDate(day)}
              className={cn(
                "size-7 rounded-md text-xs tabular-nums transition-colors hover:bg-accent",
                isSameMonth(day, visibleMonth)
                  ? "text-foreground"
                  : "text-muted-foreground/50",
                isSameDay(day, anchorDate) &&
                  "bg-primary text-primary-foreground hover:bg-primary",
                isSameDay(day, now) &&
                  !isSameDay(day, anchorDate) &&
                  "ring-1 ring-primary",
              )}
            >
              {format(day, "d")}
            </button>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  )
}

export function CalendarToolbar() {
  const { anchorDate, goToToday, setView, showMiniNav, step, view } =
    useCalendar()

  return (
    <div className="flex shrink-0 flex-wrap items-center justify-between gap-2 border-b border-border px-3 py-2">
      <div className="flex items-center gap-1">
        <Button variant="outline" size="sm" onClick={goToToday} title="Today (T)">
          Today
        </Button>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Previous period"
          onClick={() => step(-1)}
        >
          <ChevronLeft className="size-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Next period"
          onClick={() => step(1)}
        >
          <ChevronRight className="size-4" />
        </Button>
        <h2 className="ml-2 text-sm font-semibold text-foreground">
          {periodTitle(view, anchorDate)}
        </h2>
      </div>

      <div className="flex items-center gap-1">
        <div className="flex items-center rounded-md border border-border p-0.5">
          {views.map((entry) => (
            <button
              key={entry.value}
              type="button"
              title={`${entry.label} (${entry.hotkey})`}
              aria-pressed={view === entry.value}
              onClick={() => setView(entry.value)}
              className={cn(
                "rounded-sm px-2.5 py-1 text-xs transition-colors",
                view === entry.value
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:text-foreground",
              )}
            >
              {entry.label}
            </button>
          ))}
        </div>
        {showMiniNav ? <MiniNav /> : null}
      </div>
    </div>
  )
}
