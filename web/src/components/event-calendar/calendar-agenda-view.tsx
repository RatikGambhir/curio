import { useMemo } from "react"
import { addDays, format, isSameDay } from "date-fns"

import { cn } from "@/lib/utils"

import { CalendarEvent } from "./calendar-event"
import { useCalendar } from "./calendar-context"
import { tasksForDay } from "./layout"

export function CalendarAgendaView() {
  const { now, range, resolved } = useCalendar()

  const days = useMemo(() => {
    const result: Date[] = []
    let cursor = range.start

    while (cursor < range.end) {
      result.push(cursor)
      cursor = addDays(cursor, 1)
    }

    return result
  }, [range.end, range.start])

  const populated = days
    .map((day) => ({ day, entries: tasksForDay(resolved, day) }))
    .filter((group) => group.entries.length > 0)

  if (populated.length === 0) {
    return (
      <div className="flex min-h-0 flex-1 items-center justify-center p-6 text-sm text-muted-foreground">
        Nothing scheduled in this range.
      </div>
    )
  }

  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <ul className="divide-y divide-border">
        {populated.map(({ day, entries }) => (
          <li key={day.toISOString()} className="flex gap-4 px-4 py-3">
            <div className="w-24 shrink-0">
              <div
                className={cn(
                  "text-sm font-medium",
                  isSameDay(day, now) ? "text-primary" : "text-foreground",
                )}
              >
                {format(day, "EEE d MMM")}
              </div>
              <div className="text-[11px] text-muted-foreground">
                {entries.length} {entries.length === 1 ? "event" : "events"}
              </div>
            </div>
            <div className="flex min-w-0 flex-1 flex-col gap-1">
              {entries.map((entry) => (
                <CalendarEvent
                  key={entry.task.id}
                  resolved={entry}
                  variant="row"
                  className="hover:bg-accent/50"
                />
              ))}
            </div>
          </li>
        ))}
      </ul>
    </div>
  )
}
