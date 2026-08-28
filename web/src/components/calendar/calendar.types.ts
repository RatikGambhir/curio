import type { TaskItem } from "@/components/event-calendar"

/**
 * Re-exported so no app file imports out of `src/components/event-calendar/**`
 * directly — that indirection is what lets the vendored tree be regenerated
 * without touching page code.
 */
export type { TaskItem }

/**
 * A Curio calendar entry. Narrow extension of the calendar's canonical record.
 *
 * `startDate` / `endDate` keep the component's string-format contract: a bare
 * `YYYY-MM-DD` is all-day, a full ISO timestamp is timed, and a timestamp with
 * no end is a milestone. Anything that writes these — mock data now, the events
 * API later — has to preserve that distinction rather than normalising
 * everything to ISO.
 */
export type CurioCalendarEvent = TaskItem & {
  /** Where the entry came from, once events can be derived. */
  source?: "user" | "vault" | "chat"
}

export type CalendarStatus =
  | "scheduled"
  | "in-progress"
  | "blocked"
  | "done"
  | "cancelled"

export type CalendarPriority = "low" | "medium" | "high"
