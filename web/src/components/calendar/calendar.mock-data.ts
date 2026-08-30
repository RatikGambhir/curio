import { addDays, format, startOfWeek } from "date-fns";

import type { CurioCalendarEvent } from "./calendar.types";

const DATE_ONLY = "yyyy-MM-dd";

/** All-day events serialize date-only. Never an ISO timestamp. */
const allDay = (date: Date) => format(date, DATE_ONLY);

/** Timed events serialize a full timestamp. */
const timed = (date: Date, hour: number, minute = 0) => {
  const value = new Date(date);
  value.setHours(hour, minute, 0, 0);
  return value.toISOString();
};

/**
 * Mock events anchored to a reference date, so week and day views are never
 * empty on load. Taking the reference as an argument keeps this deterministic
 * for tests instead of reading the clock at module scope.
 */
export function buildMockCalendarEvents(reference: Date): CurioCalendarEvent[] {
  const weekStart = startOfWeek(reference);
  const day = (offset: number) => addDays(weekStart, offset);

  return [
    // All-day, single day.
    {
      id: "evt-reading-day",
      name: "Reading day",
      status: "scheduled",
      active: true,
      priority: "low",
      labels: ["research"],
      setAt: allDay(day(1)),
      source: "user",
    },
    // All-day, multi-day span — exercises the month row's spanning bars.
    {
      id: "evt-vault-migration",
      name: "Vault migration window",
      description: "Backfill and re-index every vault document.",
      status: "in-progress",
      active: true,
      priority: "high",
      labels: ["ops"],
      setAt: allDay(day(2)),
      expireAt: allDay(day(5)),
      source: "vault",
    },
    // A second span that overlaps the first, forcing a second lane.
    {
      id: "evt-freeze",
      name: "Release freeze",
      status: "blocked",
      active: true,
      priority: "high",
      setAt: allDay(day(3)),
      expireAt: allDay(day(4)),
      source: "user",
    },
    // Timed events, two of them overlapping to exercise lane packing.
    {
      id: "evt-standup",
      name: "Standup",
      status: "scheduled",
      active: true,
      priority: "medium",
      setAt: timed(day(1), 9, 30),
      expireAt: timed(day(1), 9, 45),
      source: "user",
    },
    {
      id: "evt-design-review",
      name: "Design review",
      status: "scheduled",
      active: true,
      priority: "medium",
      labels: ["review"],
      setAt: timed(day(2), 14),
      expireAt: timed(day(2), 15, 30),
      source: "user",
    },
    {
      id: "evt-pairing",
      name: "Pairing on the notes editor",
      status: "in-progress",
      active: true,
      priority: "medium",
      setAt: timed(day(2), 14, 30),
      expireAt: timed(day(2), 16),
      source: "chat",
    },
    {
      id: "evt-retro",
      name: "Retro",
      status: "scheduled",
      active: true,
      priority: "low",
      setAt: timed(day(4), 16),
      expireAt: timed(day(4), 17),
      source: "user",
    },
    // Milestone: a timestamp with no end date.
    {
      id: "evt-ship",
      name: "Ship calendar page",
      status: "scheduled",
      active: true,
      priority: "high",
      setAt: timed(day(5), 12),
      source: "user",
    },
    // A crowded day, to trigger the month view's "+N more" overflow.
    ...Array.from({ length: 5 }, (_, index) => ({
      id: `evt-interview-${index + 1}`,
      name: `Research interview ${index + 1}`,
      status: index % 2 === 0 ? "scheduled" : "done",
      active: true,
      priority: "medium",
      labels: ["research"],
      setAt: timed(day(3), 9 + index),
      expireAt: timed(day(3), 10 + index),
      source: "user" as const,
    })),
    // Next week and last week, so period navigation lands on something.
    {
      id: "evt-next-week-planning",
      name: "Cycle planning",
      status: "scheduled",
      active: true,
      priority: "high",
      setAt: timed(addDays(weekStart, 8), 11),
      expireAt: timed(addDays(weekStart, 8), 12, 30),
      source: "user",
    },
    {
      id: "evt-last-week-offsite",
      name: "Team offsite",
      status: "done",
      active: true,
      priority: "low",
      setAt: allDay(addDays(weekStart, -4)),
      expireAt: allDay(addDays(weekStart, -3)),
      source: "user",
    },
    {
      id: "evt-long-block",
      name: "Deep work",
      status: "in-progress",
      active: true,
      priority: "medium",
      setAt: timed(day(6), 10),
      expireAt: timed(day(6), 15),
      source: "user",
    },
  ];
}
