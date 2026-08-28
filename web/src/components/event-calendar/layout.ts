import {
  addDays,
  differenceInCalendarDays,
  differenceInMinutes,
  endOfDay,
  startOfDay,
} from "date-fns"

import type { ResolvedTask } from "./classify"

export const MINUTES_PER_DAY = 24 * 60

/** One all-day/multi-day bar clipped to a single week row. */
export type WeekSegment = {
  resolved: ResolvedTask
  /** Column index within the week, 0-6. */
  startIndex: number
  span: number
  lane: number
  continuesBefore: boolean
  continuesAfter: boolean
}

function overlapsDay(resolved: ResolvedTask, day: Date): boolean {
  return resolved.start < endOfDay(day) && resolved.end > startOfDay(day)
}

export function tasksForDay(
  resolved: readonly ResolvedTask[],
  day: Date,
): ResolvedTask[] {
  return resolved.filter((entry) => overlapsDay(entry, day))
}

/**
 * Packs spanning events into lanes across one week row. Lanes are assigned
 * greedily so a bar keeps the same row for its whole span.
 */
export function buildWeekSegments(
  weekStart: Date,
  resolved: readonly ResolvedTask[],
): WeekSegment[] {
  const weekEnd = addDays(weekStart, 7)
  const laneEnds: number[] = []

  return resolved
    .filter((entry) => entry.start < weekEnd && entry.end > weekStart)
    .sort((a, b) => {
      const byStart = a.start.getTime() - b.start.getTime()
      if (byStart !== 0) {
        return byStart
      }
      return b.end.getTime() - a.end.getTime()
    })
    .map((entry) => {
      const startIndex = Math.max(
        0,
        differenceInCalendarDays(startOfDay(entry.start), weekStart),
      )
      const endIndex = Math.min(
        6,
        differenceInCalendarDays(startOfDay(addDays(entry.end, -1)), weekStart),
      )
      const span = Math.max(1, endIndex - startIndex + 1)

      let lane = laneEnds.findIndex((end) => end <= startIndex)
      if (lane === -1) {
        lane = laneEnds.length
      }
      laneEnds[lane] = startIndex + span

      return {
        resolved: entry,
        startIndex,
        span,
        lane,
        continuesBefore: entry.start < weekStart,
        continuesAfter: entry.end > weekEnd,
      }
    })
}

export type TimedLayout = {
  resolved: ResolvedTask
  topPercent: number
  heightPercent: number
  columnIndex: number
  columnCount: number
}

/**
 * Positions timed events inside one day column, splitting overlapping runs
 * into side-by-side columns.
 */
export function layoutTimedTasks(
  day: Date,
  resolved: readonly ResolvedTask[],
): TimedLayout[] {
  const dayStart = startOfDay(day)
  const dayEnd = addDays(dayStart, 1)

  const entries = resolved
    .filter((entry) => entry.start < dayEnd && entry.end > dayStart)
    .sort((a, b) => a.start.getTime() - b.start.getTime())

  const layouts: TimedLayout[] = []
  let cluster: ResolvedTask[] = []
  let clusterEnd = 0

  const flush = () => {
    if (cluster.length === 0) {
      return
    }

    const columnEnds: number[] = []
    const assigned = cluster.map((entry) => {
      let column = columnEnds.findIndex((end) => end <= entry.start.getTime())
      if (column === -1) {
        column = columnEnds.length
      }
      columnEnds[column] = entry.end.getTime()
      return { entry, column }
    })

    const columnCount = columnEnds.length
    for (const { entry, column } of assigned) {
      const startMinutes = Math.max(
        0,
        differenceInMinutes(entry.start, dayStart),
      )
      const endMinutes = Math.min(
        MINUTES_PER_DAY,
        differenceInMinutes(entry.end, dayStart),
      )

      layouts.push({
        resolved: entry,
        topPercent: (startMinutes / MINUTES_PER_DAY) * 100,
        heightPercent:
          (Math.max(20, endMinutes - startMinutes) / MINUTES_PER_DAY) * 100,
        columnIndex: column,
        columnCount,
      })
    }

    cluster = []
    clusterEnd = 0
  }

  for (const entry of entries) {
    if (cluster.length > 0 && entry.start.getTime() >= clusterEnd) {
      flush()
    }
    cluster.push(entry)
    clusterEnd = Math.max(clusterEnd, entry.end.getTime())
  }
  flush()

  return layouts
}

/** Fraction of the day a timestamp sits at, for the now indicator. */
export function dayFraction(date: Date): number {
  return differenceInMinutes(date, startOfDay(date)) / MINUTES_PER_DAY
}
