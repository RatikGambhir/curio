import { describe, expect, it } from "vitest"

import {
  classifyTask,
  formatTaskDate,
  isDateOnly,
  parseTaskDate,
} from "@/components/event-calendar"
import { calendarEditing } from "@/components/event-calendar"
import { buildMockCalendarEvents } from "@/components/calendar/calendar.mock-data"
import type { TaskItem } from "@/components/calendar/calendar.types"

const allDayTask: TaskItem = {
  id: "all-day",
  title: "Reading day",
  startDate: "2026-06-22",
}

const spanTask: TaskItem = {
  id: "span",
  title: "Migration window",
  startDate: "2026-06-22",
  endDate: "2026-06-25",
}

const timedTask: TaskItem = {
  id: "timed",
  title: "Design review",
  startDate: "2026-06-22T14:00:00.000Z",
  endDate: "2026-06-22T15:30:00.000Z",
}

const milestoneTask: TaskItem = {
  id: "milestone",
  title: "Ship it",
  startDate: "2026-06-22T12:00:00.000Z",
}

describe("event classification", () => {
  it("reads date-only strings as all-day and timestamps as timed", () => {
    expect(classifyTask(allDayTask)).toBe("all-day")
    expect(classifyTask(spanTask)).toBe("all-day")
    expect(classifyTask(timedTask)).toBe("timed")
  })

  it("treats a timestamp with no end date as a milestone", () => {
    expect(classifyTask(milestoneTask)).toBe("milestone")
  })

  it("lets an explicit predicate win over the format", () => {
    expect(classifyTask(allDayTask, () => "timed")).toBe("timed")
    // Falling through returns undefined, so format still decides.
    expect(classifyTask(allDayTask, () => undefined)).toBe("all-day")
  })

  it("parses a date-only string at local midnight, not UTC", () => {
    const parsed = parseTaskDate("2026-06-22")

    expect(parsed.getFullYear()).toBe(2026)
    expect(parsed.getMonth()).toBe(5)
    expect(parsed.getDate()).toBe(22)
    expect(parsed.getHours()).toBe(0)
  })

  it("recognises which strings are date-only", () => {
    expect(isDateOnly("2026-06-22")).toBe(true)
    expect(isDateOnly("2026-06-22T14:00:00.000Z")).toBe(false)
    expect(isDateOnly(undefined)).toBe(false)
  })
})

describe("editing preserves the all-day contract", () => {
  it("keeps a moved all-day event date-only", () => {
    const moved = calendarEditing.moveTask(allDayTask, 3, 0)

    expect(moved.startDate).toBe("2026-06-25")
    expect(isDateOnly(moved.startDate)).toBe(true)
    expect(classifyTask(moved)).toBe("all-day")
  })

  it("ignores a minute delta on an all-day event", () => {
    // A vertical drag or ArrowDown must not push an all-day event to 00:15.
    const nudged = calendarEditing.moveTask(allDayTask, 0, 15)

    expect(nudged.startDate).toBe("2026-06-22")
    expect(classifyTask(nudged)).toBe("all-day")
  })

  it("keeps a moved timed event a timestamp", () => {
    const moved = calendarEditing.moveTask(timedTask, 1, 30)

    expect(isDateOnly(moved.startDate)).toBe(false)
    expect(classifyTask(moved)).toBe("timed")
    expect(parseTaskDate(moved.startDate).getTime()).toBe(
      parseTaskDate(timedTask.startDate).getTime() + 24.5 * 60 * 60 * 1000,
    )
  })

  it("keeps both ends of a span date-only when resized", () => {
    const resized = calendarEditing.resizeTask(spanTask, "end", 2, 0)

    expect(resized.startDate).toBe("2026-06-22")
    expect(resized.endDate).toBe("2026-06-27")
    expect(classifyTask(resized)).toBe("all-day")
  })

  it("refuses a resize that would invert the event", () => {
    expect(calendarEditing.resizeTask(spanTask, "end", -10, 0)).toBe(spanTask)
    expect(calendarEditing.resizeTask(spanTask, "start", 10, 0)).toBe(spanTask)
  })

  it("creates all-day and timed events in the matching format", () => {
    const start = new Date(2026, 5, 22, 9, 0, 0)
    const end = new Date(2026, 5, 22, 10, 0, 0)

    expect(calendarEditing.createTask({ start, end, allDay: true }).startDate).toBe(
      "2026-06-22",
    )
    expect(
      isDateOnly(calendarEditing.createTask({ start, end, allDay: false }).startDate),
    ).toBe(false)
  })

  it("gives a duplicate a fresh id", () => {
    const copy = calendarEditing.duplicateTask(timedTask)

    expect(copy.id).not.toBe(timedTask.id)
    expect(copy.startDate).toBe(timedTask.startDate)
  })

  it("round-trips through formatTaskDate", () => {
    const date = new Date(2026, 5, 22, 13, 45, 0)

    expect(formatTaskDate(date, true)).toBe("2026-06-22")
    expect(isDateOnly(formatTaskDate(date, false))).toBe(false)
  })
})

describe("mock calendar data", () => {
  const events = buildMockCalendarEvents(new Date(2026, 5, 22))

  it("is deterministic for a given reference date", () => {
    expect(buildMockCalendarEvents(new Date(2026, 5, 22))).toEqual(events)
  })

  it("covers every shape the views have to render", () => {
    const kinds = new Set(events.map((event) => classifyTask(event)))

    expect(kinds).toContain("all-day")
    expect(kinds).toContain("timed")
    expect(kinds).toContain("milestone")
    expect(
      events.some(
        (event) => event.endDate && isDateOnly(event.startDate) && event.endDate !== event.startDate,
      ),
    ).toBe(true)
  })

  it("keeps every id unique", () => {
    const ids = events.map((event) => event.id)
    expect(new Set(ids).size).toBe(ids.length)
  })

  it("never serializes an all-day event as a timestamp", () => {
    for (const event of events) {
      if (classifyTask(event) === "all-day") {
        expect(isDateOnly(event.startDate)).toBe(true)
        if (event.endDate) {
          expect(isDateOnly(event.endDate)).toBe(true)
        }
      }
    }
  })
})
