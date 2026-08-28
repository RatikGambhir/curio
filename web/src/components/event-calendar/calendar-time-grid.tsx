import { useMemo } from "react"
import { useDroppable } from "@dnd-kit/core"
import { addMinutes, format, isSameDay, startOfDay } from "date-fns"

import { cn } from "@/lib/utils"

import { CalendarEvent, HOUR_HEIGHT, SNAP_MINUTES } from "./calendar-event"
import { useCalendar } from "./calendar-context"
import { buildWeekSegments, dayFraction, layoutTimedTasks } from "./layout"

const HOURS = Array.from({ length: 24 }, (_, hour) => hour)
const GUTTER = "3.5rem"

function DayColumn({ day, isLast }: { day: Date; isLast: boolean }) {
  const { createTaskAt, editable, now, permissions, resolved } = useCalendar()
  const { isOver, setNodeRef } = useDroppable({
    id: `slot:${day.toISOString()}`,
    data: { day: day.toISOString(), allDay: false },
  })

  const layouts = useMemo(
    () => layoutTimedTasks(day, resolved.filter((entry) => entry.kind !== "all-day")),
    [day, resolved],
  )

  const showNow = isSameDay(day, now)

  return (
    <div
      ref={setNodeRef}
      onDoubleClick={(event) => {
        if (!editable || !permissions.create) {
          return
        }

        const bounds = event.currentTarget.getBoundingClientRect()
        const minutes = (event.clientY - bounds.top) / (HOUR_HEIGHT / 60)
        const snapped = Math.round(minutes / SNAP_MINUTES) * SNAP_MINUTES
        const start = addMinutes(startOfDay(day), snapped)

        createTaskAt({ start, end: addMinutes(start, 60), allDay: false })
      }}
      className={cn(
        "relative min-w-0 flex-1 border-r border-border",
        isLast && "border-r-0",
        isOver && "bg-accent/30",
      )}
      style={{ height: 24 * HOUR_HEIGHT }}
    >
      {HOURS.map((hour) => (
        <div
          key={hour}
          className="border-b border-border/60"
          style={{ height: HOUR_HEIGHT }}
        />
      ))}

      {layouts.map((layout) => (
        <div
          key={layout.resolved.task.id}
          className="absolute px-0.5"
          style={{
            top: `${layout.topPercent}%`,
            height: `${layout.heightPercent}%`,
            left: `${(layout.columnIndex / layout.columnCount) * 100}%`,
            width: `${(1 / layout.columnCount) * 100}%`,
          }}
        >
          <CalendarEvent resolved={layout.resolved} variant="block" />
        </div>
      ))}

      {showNow ? (
        <div
          aria-hidden
          className="pointer-events-none absolute inset-x-0 z-30 border-t-2 border-destructive"
          style={{ top: `${dayFraction(now) * 100}%` }}
        >
          <span className="absolute -top-1 -left-1 size-2 rounded-full bg-destructive" />
        </div>
      ) : null}
    </div>
  )
}

export function CalendarTimeGrid({ days }: { days: Date[] }) {
  const { now, resolved, setAnchorDate, setView } = useCalendar()

  const allDay = useMemo(
    () => resolved.filter((entry) => entry.kind === "all-day"),
    [resolved],
  )
  const segments = useMemo(
    () => buildWeekSegments(days[0], allDay).filter((s) => s.startIndex < days.length),
    [allDay, days],
  )
  const laneCount = segments.reduce(
    (max, segment) => Math.max(max, segment.lane + 1),
    0,
  )

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div className="flex shrink-0 border-b border-border">
        <div className="shrink-0 border-r border-border" style={{ width: GUTTER }} />
        <div className="flex flex-1">
          {days.map((day, index) => (
            <button
              key={day.toISOString()}
              type="button"
              onClick={() => {
                setAnchorDate(day)
                setView("day")
              }}
              className={cn(
                "min-w-0 flex-1 border-r border-border px-2 py-1.5 text-left last:border-r-0 hover:bg-accent/40",
                index === days.length - 1 && "border-r-0",
              )}
            >
              <span className="block text-[11px] tracking-wide text-muted-foreground uppercase">
                {format(day, "EEE")}
              </span>
              <span
                className={cn(
                  "mt-0.5 inline-flex size-6 items-center justify-center rounded-full text-sm tabular-nums",
                  isSameDay(day, now) &&
                    "bg-primary font-semibold text-primary-foreground",
                )}
              >
                {format(day, "d")}
              </span>
            </button>
          ))}
        </div>
      </div>

      {laneCount > 0 ? (
        <div className="flex shrink-0 border-b border-border">
          <div
            className="shrink-0 border-r border-border py-1 pr-2 text-right text-[10px] text-muted-foreground"
            style={{ width: GUTTER }}
          >
            all-day
          </div>
          <div
            data-week-row
            className="relative flex-1"
            style={{ height: laneCount * 22 + 4 }}
          >
            {segments.map((segment) => (
              <div
                key={`${segment.resolved.task.id}-${segment.lane}`}
                className="absolute px-1"
                style={{
                  left: `${(segment.startIndex / days.length) * 100}%`,
                  width: `${(Math.min(segment.span, days.length - segment.startIndex) / days.length) * 100}%`,
                  top: segment.lane * 22 + 2,
                }}
              >
                <CalendarEvent
                  resolved={segment.resolved}
                  variant="bar"
                  continuesBefore={segment.continuesBefore}
                  continuesAfter={segment.continuesAfter}
                />
              </div>
            ))}
          </div>
        </div>
      ) : null}

      <div className="flex min-h-0 flex-1 overflow-y-auto">
        <div className="shrink-0 border-r border-border" style={{ width: GUTTER }}>
          {HOURS.map((hour) => (
            <div
              key={hour}
              className="relative pr-2 text-right text-[10px] text-muted-foreground tabular-nums"
              style={{ height: HOUR_HEIGHT }}
            >
              <span className="absolute top-0 right-2 -translate-y-1/2">
                {hour === 0 ? "" : format(addMinutes(startOfDay(now), hour * 60), "HH:mm")}
              </span>
            </div>
          ))}
        </div>
        <div className="flex flex-1">
          {days.map((day, index) => (
            <DayColumn
              key={day.toISOString()}
              day={day}
              isLast={index === days.length - 1}
            />
          ))}
        </div>
      </div>
    </div>
  )
}
