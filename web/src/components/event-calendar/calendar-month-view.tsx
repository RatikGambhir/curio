import { Fragment, useMemo } from "react"
import { useDroppable } from "@dnd-kit/core"
import {
  addDays,
  endOfDay,
  format,
  isSameDay,
  isSameMonth,
  startOfDay,
} from "date-fns"

import { cn } from "@/lib/utils"

import { CalendarEvent } from "./calendar-event"
import { useCalendar } from "./calendar-context"
import type { ResolvedTask } from "./classify"
import { buildWeekSegments } from "./layout"

const MAX_LANES = 3
const MAX_CHIPS = 2
const LANE_HEIGHT = 22
const CHIP_HEIGHT = 21
const HEADER_HEIGHT = 28
const MIN_CHIP_ROWS = 1

function DayCell({
  day,
  children,
  className,
}: {
  day: Date
  children: React.ReactNode
  className?: string
}) {
  const { createTaskAt, editable, permissions } = useCalendar()
  const { isOver, setNodeRef } = useDroppable({
    id: `day:${day.toISOString()}`,
    data: { day: day.toISOString(), allDay: true },
  })

  return (
    <div
      ref={setNodeRef}
      onDoubleClick={() => {
        if (editable && permissions.create) {
          createTaskAt({ start: startOfDay(day), end: endOfDay(day), allDay: true })
        }
      }}
      className={cn(
        "flex min-w-0 flex-col overflow-hidden border-r border-b border-border last:border-r-0",
        isOver && "bg-accent/40",
        className,
      )}
    >
      {children}
    </div>
  )
}

export function CalendarMonthView() {
  const { anchorDate, now, range, resolved, setAnchorDate, setView } =
    useCalendar()

  const weeks = useMemo(() => {
    const result: Date[][] = []
    let cursor = range.start

    while (cursor < range.end) {
      result.push(
        Array.from({ length: 7 }, (_, index) => addDays(cursor, index)),
      )
      cursor = addDays(cursor, 7)
    }

    return result
  }, [range.end, range.start])

  const spanning = useMemo(
    () => resolved.filter((entry) => entry.kind === "all-day"),
    [resolved],
  )
  const pointEvents = useMemo(
    () => resolved.filter((entry) => entry.kind !== "all-day"),
    [resolved],
  )

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div className="grid shrink-0 grid-cols-7 border-b border-border">
        {weeks[0]?.map((day) => (
          <div
            key={day.toISOString()}
            className="border-r border-border px-2 py-1.5 text-[11px] font-medium tracking-wide text-muted-foreground uppercase last:border-r-0"
          >
            {format(day, "EEE")}
          </div>
        ))}
      </div>

      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
        {weeks.map((week) => {
          const segments = buildWeekSegments(week[0], spanning)
          const visible = segments.filter((segment) => segment.lane < MAX_LANES)
          const lanesUsed = Math.min(
            MAX_LANES,
            segments.reduce((max, segment) => Math.max(max, segment.lane + 1), 0),
          )

          // Rows are sized from their own content. Without this the grid gives
          // every row an equal share and busy weeks spill into the next one.
          const chipRows = week.reduce((max, day) => {
            const dayPoints = pointEvents.filter((entry) => overlapsDay(entry, day))
            const shown = Math.min(dayPoints.length, MAX_CHIPS)
            const overflow =
              dayPoints.length > shown ||
              segments.some(
                (segment) =>
                  segment.lane >= MAX_LANES && overlapsDay(segment.resolved, day),
              )
            return Math.max(max, shown + (overflow ? 1 : 0))
          }, MIN_CHIP_ROWS)

          return (
            <div
              key={week[0].toISOString()}
              data-week-row
              className="relative grid flex-1 grid-cols-7"
              style={{
                minHeight:
                  HEADER_HEIGHT + lanesUsed * LANE_HEIGHT + chipRows * CHIP_HEIGHT + 8,
              }}
            >
              {week.map((day) => {
                const dayPoints = pointEvents.filter((entry) =>
                  overlapsDay(entry, day),
                )
                const hiddenSpans = segments.filter(
                  (segment) =>
                    segment.lane >= MAX_LANES && overlapsDay(segment.resolved, day),
                ).length
                const visibleChips = dayPoints.slice(0, MAX_CHIPS)
                const hidden = hiddenSpans + (dayPoints.length - visibleChips.length)

                return (
                  <DayCell key={day.toISOString()} day={day}>
                    <div className="flex items-center justify-between px-1.5 pt-1">
                      <button
                        type="button"
                        onClick={() => {
                          setAnchorDate(day)
                          setView("day")
                        }}
                        className={cn(
                          "flex size-6 items-center justify-center rounded-full text-xs tabular-nums transition-colors hover:bg-accent",
                          isSameMonth(day, anchorDate)
                            ? "text-foreground"
                            : "text-muted-foreground/60",
                          isSameDay(day, now) &&
                            "bg-primary font-semibold text-primary-foreground hover:bg-primary",
                        )}
                      >
                        {format(day, "d")}
                      </button>
                    </div>

                    <div style={{ height: lanesUsed * LANE_HEIGHT }} />

                    <div className="flex min-h-0 flex-col gap-0.5 px-1 pb-1">
                      {visibleChips.map((entry) => (
                        <CalendarEvent
                          key={entry.task.id}
                          resolved={entry}
                          variant="chip"
                        />
                      ))}
                      {hidden > 0 ? (
                        <button
                          type="button"
                          onClick={() => {
                            setAnchorDate(day)
                            setView("day")
                          }}
                          className="px-1 text-left text-[11px] text-muted-foreground hover:text-foreground"
                        >
                          +{hidden} more
                        </button>
                      ) : null}
                    </div>
                  </DayCell>
                )
              })}

              <div
                className="pointer-events-none absolute inset-x-0"
                style={{ top: HEADER_HEIGHT }}
              >
                {visible.map((segment) => (
                  <Fragment key={`${segment.resolved.task.id}-${segment.lane}`}>
                    <div
                      className="pointer-events-auto absolute px-1"
                      style={{
                        left: `${(segment.startIndex / 7) * 100}%`,
                        width: `${(segment.span / 7) * 100}%`,
                        top: segment.lane * LANE_HEIGHT,
                      }}
                    >
                      <CalendarEvent
                        resolved={segment.resolved}
                        variant="bar"
                        continuesBefore={segment.continuesBefore}
                        continuesAfter={segment.continuesAfter}
                      />
                    </div>
                  </Fragment>
                ))}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

function overlapsDay(entry: ResolvedTask, day: Date): boolean {
  return entry.start < endOfDay(day) && entry.end > startOfDay(day)
}
