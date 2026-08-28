import {
  addDays,
  addMinutes,
  differenceInCalendarDays,
  format,
  parse,
  parseISO,
  startOfDay,
} from "date-fns"

import type { CalendarEventKind, TaskItem } from "./types"

const DATE_ONLY = /^\d{4}-\d{2}-\d{2}$/

export const DATE_ONLY_FORMAT = "yyyy-MM-dd"

export function isDateOnly(value: string | undefined): boolean {
  return typeof value === "string" && DATE_ONLY.test(value)
}

/**
 * Parses either shape into local time.
 *
 * `parseISO("2026-06-22")` yields UTC midnight, which lands on the previous day
 * west of Greenwich, so date-only strings go through `parse` instead.
 */
export function parseTaskDate(value: string): Date {
  return isDateOnly(value)
    ? parse(value, DATE_ONLY_FORMAT, new Date())
    : parseISO(value)
}

export function formatTaskDate(date: Date, allDay: boolean): string {
  return allDay ? format(date, DATE_ONLY_FORMAT) : date.toISOString()
}

/**
 * Three layers, in the documented order: an explicit predicate, then the
 * string format, then a span heuristic.
 */
export function classifyTask(
  task: TaskItem,
  classifyEvent?: (task: TaskItem) => CalendarEventKind | undefined,
): CalendarEventKind {
  const explicit = classifyEvent?.(task)
  if (explicit) {
    return explicit
  }

  if (isDateOnly(task.startDate)) {
    return "all-day"
  }

  if (!task.endDate) {
    return "milestone"
  }

  // Span heuristic: a timestamped event covering whole days reads as all-day.
  const start = parseTaskDate(task.startDate)
  const end = parseTaskDate(task.endDate)
  const spansDays = differenceInCalendarDays(end, start) >= 1
  const startsAtMidnight =
    start.getHours() === 0 && start.getMinutes() === 0 && spansDays

  return startsAtMidnight ? "all-day" : "timed"
}

export function isAllDayKind(kind: CalendarEventKind): boolean {
  return kind === "all-day"
}

export type ResolvedTask = {
  task: TaskItem
  kind: CalendarEventKind
  start: Date
  /** Exclusive end used for layout. All-day `endDate` is inclusive in data. */
  end: Date
}

export function resolveTask(
  task: TaskItem,
  classifyEvent?: (task: TaskItem) => CalendarEventKind | undefined,
): ResolvedTask {
  const kind = classifyTask(task, classifyEvent)
  const start = parseTaskDate(task.startDate)

  if (kind === "milestone") {
    return { task, kind, start, end: addMinutes(start, 30) }
  }

  if (kind === "all-day") {
    const inclusiveEnd = task.endDate ? parseTaskDate(task.endDate) : start
    return {
      task,
      kind,
      start: startOfDay(start),
      end: addDays(startOfDay(inclusiveEnd), 1),
    }
  }

  const end = task.endDate ? parseTaskDate(task.endDate) : addMinutes(start, 60)
  return { task, kind, start, end }
}

export function resolveTasks(
  tasks: readonly TaskItem[],
  classifyEvent?: (task: TaskItem) => CalendarEventKind | undefined,
): ResolvedTask[] {
  return tasks
    .map((task) => resolveTask(task, classifyEvent))
    .sort((a, b) => a.start.getTime() - b.start.getTime())
}

/**
 * Shifts a task, preserving its all-day vs timed serialization. Dragging an
 * all-day event must not quietly convert it into a midnight timed block.
 */
export function shiftTask(
  task: TaskItem,
  dayDelta: number,
  minuteDelta: number,
): TaskItem {
  const allDay = isDateOnly(task.startDate)
  const shift = (value: string) => {
    const shifted = addMinutes(
      addDays(parseTaskDate(value), dayDelta),
      allDay ? 0 : minuteDelta,
    )
    return formatTaskDate(shifted, isDateOnly(value))
  }

  return {
    ...task,
    startDate: shift(task.startDate),
    endDate: task.endDate ? shift(task.endDate) : undefined,
  }
}
