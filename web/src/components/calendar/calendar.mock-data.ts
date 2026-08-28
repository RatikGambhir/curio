import { addDays, addHours, format, startOfWeek } from "date-fns"

import type { CurioCalendarEvent } from "./calendar.types"

const DATE_ONLY = "yyyy-MM-dd"

/** All-day events serialize date-only. Never an ISO timestamp. */
const allDay = (date: Date) => format(date, DATE_ONLY)

/** Timed events serialize a full timestamp. */
const timed = (date: Date, hour: number, minute = 0) => {
  const value = new Date(date)
  value.setHours(hour, minute, 0, 0)
  return value.toISOString()
}

/**
 * Mock events anchored to a reference date, so week and day views are never
 * empty on load. Taking the reference as an argument keeps this deterministic
 * for tests instead of reading the clock at module scope.
 */
export function buildMockCalendarEvents(
  reference: Date,
): CurioCalendarEvent[] {
  const weekStart = startOfWeek(reference)
  const day = (offset: number) => addDays(weekStart, offset)

  return [
    // All-day, single day.
    {
      id: "evt-reading-day",
      title: "Reading day",
      status: "scheduled",
      priority: "low",
      labels: ["research"],
      startDate: allDay(day(1)),
      source: "user",
    },
    // All-day, multi-day span — exercises the month row's spanning bars.
    {
      id: "evt-vault-migration",
      title: "Vault migration window",
      description: "Backfill and re-index every vault document.",
      status: "in-progress",
      priority: "high",
      labels: ["ops"],
      startDate: allDay(day(2)),
      endDate: allDay(day(5)),
      source: "vault",
    },
    // A second span that overlaps the first, forcing a second lane.
    {
      id: "evt-freeze",
      title: "Release freeze",
      status: "blocked",
      priority: "high",
      startDate: allDay(day(3)),
      endDate: allDay(day(4)),
      source: "user",
    },
    // Timed events, two of them overlapping to exercise lane packing.
    {
      id: "evt-standup",
      title: "Standup",
      status: "scheduled",
      priority: "medium",
      startDate: timed(day(1), 9, 30),
      endDate: timed(day(1), 9, 45),
      source: "user",
    },
    {
      id: "evt-design-review",
      title: "Design review",
      status: "scheduled",
      priority: "medium",
      labels: ["review"],
      startDate: timed(day(2), 14),
      endDate: timed(day(2), 15, 30),
      source: "user",
    },
    {
      id: "evt-pairing",
      title: "Pairing on the notes editor",
      status: "in-progress",
      priority: "medium",
      startDate: timed(day(2), 14, 30),
      endDate: timed(day(2), 16),
      source: "chat",
    },
    {
      id: "evt-retro",
      title: "Retro",
      status: "scheduled",
      priority: "low",
      startDate: timed(day(4), 16),
      endDate: timed(day(4), 17),
      source: "user",
    },
    // Milestone: a timestamp with no end date.
    {
      id: "evt-ship",
      title: "Ship calendar page",
      status: "scheduled",
      priority: "high",
      startDate: timed(day(5), 12),
      source: "user",
    },
    // A crowded day, to trigger the month view's "+N more" overflow.
    ...Array.from({ length: 5 }, (_, index) => ({
      id: `evt-interview-${index + 1}`,
      title: `Research interview ${index + 1}`,
      status: index % 2 === 0 ? "scheduled" : "done",
      priority: "medium",
      labels: ["research"],
      startDate: timed(day(3), 9 + index),
      endDate: timed(day(3), 10 + index),
      source: "user" as const,
    })),
    // Next week and last week, so period navigation lands on something.
    {
      id: "evt-next-week-planning",
      title: "Cycle planning",
      status: "scheduled",
      priority: "high",
      startDate: timed(addDays(weekStart, 8), 11),
      endDate: timed(addDays(weekStart, 8), 12, 30),
      source: "user",
    },
    {
      id: "evt-last-week-offsite",
      title: "Team offsite",
      status: "done",
      priority: "low",
      startDate: allDay(addDays(weekStart, -4)),
      endDate: allDay(addDays(weekStart, -3)),
      source: "user",
    },
    {
      id: "evt-long-block",
      title: "Deep work",
      status: "in-progress",
      priority: "medium",
      startDate: timed(addHours(day(6), 0), 10),
      endDate: timed(addHours(day(6), 0), 15),
      source: "user",
    },
  ]
}
