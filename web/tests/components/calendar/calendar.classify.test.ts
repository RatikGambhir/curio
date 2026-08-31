import { describe, expect, it } from "vitest";

import type { CalendarEventRecord } from "@/api/calendar";
import { calendarPermissions } from "@/components/calendar/calendar.config";
import { buildMockCalendarEvents } from "@/components/calendar/calendar.mock-data";
import type { TaskItem } from "@/components/calendar/calendar.types";
import {
  classify,
  effectiveEnd,
  effectiveStart,
  parseDateValue,
} from "@/components/event-calendar/lib/classify";
import {
  formatDateValue,
  setWindow,
} from "@/components/event-calendar/features/editing/lib/edit-mutations";
import { calendarRecordToEvent } from "@/hooks/useCalendarEvents";

const allDayTask: TaskItem = {
  id: "all-day",
  name: "Reading day",
  status: "scheduled",
  active: true,
  setAt: "2026-06-22",
};

const spanTask: TaskItem = {
  id: "span",
  name: "Migration window",
  status: "in-progress",
  active: true,
  setAt: "2026-06-22",
  expireAt: "2026-06-25",
};

const timedTask: TaskItem = {
  id: "timed",
  name: "Design review",
  status: "scheduled",
  active: true,
  setAt: "2026-06-22T14:00:00.000Z",
  expireAt: "2026-06-22T15:30:00.000Z",
};

const milestoneTask: TaskItem = {
  id: "milestone",
  name: "Ship it",
  status: "scheduled",
  active: true,
  setAt: "2026-06-22T12:00:00.000Z",
};

describe("event classification", () => {
  it("reads date-only strings as all-day and timestamps as timed", () => {
    expect(classify(allDayTask)).toBe("all-day");
    expect(classify(spanTask)).toBe("all-day");
    expect(classify(timedTask)).toBe("timed");
  });

  it("treats a timestamp with no end date as a milestone", () => {
    expect(classify(milestoneTask)).toBe("milestone");
  });

  it("lets an explicit predicate win over the stored format", () => {
    expect(classify(allDayTask, () => "timed")).toBe("timed");
    expect(classify(allDayTask, () => undefined)).toBe("all-day");
  });

  it("parses a date-only string at local midnight, not UTC", () => {
    const parsed = new Date(parseDateValue("2026-06-22").ms);

    expect(parsed.getFullYear()).toBe(2026);
    expect(parsed.getMonth()).toBe(5);
    expect(parsed.getDate()).toBe(22);
    expect(parsed.getHours()).toBe(0);
  });

  it("reports whether a parsed value is date-only", () => {
    expect(parseDateValue("2026-06-22").dateOnly).toBe(true);
    expect(parseDateValue("2026-06-22T14:00:00.000Z").dateOnly).toBe(false);
    expect(parseDateValue(undefined).dateOnly).toBe(false);
  });

  it("classifies an API record without normalizing its date strings", () => {
    const record: CalendarEventRecord = {
      id: "api-all-day",
      userId: "user-1",
      title: "Offsite",
      description: null,
      status: "scheduled",
      priority: "medium",
      allDay: true,
      startDate: "2026-06-22",
      endDate: "2026-06-25",
      createdAt: "2026-06-01 00:00:00",
      updatedAt: "2026-06-01 00:00:00",
    };

    const event = calendarRecordToEvent(record);

    expect(event.setAt).toBe("2026-06-22");
    expect(event.expireAt).toBe("2026-06-25");
    expect(classify(event)).toBe("all-day");
  });
});

describe("calendar persistence permissions", () => {
  it("keeps unsupported updates and deletes read-only", () => {
    expect(calendarPermissions.default).toEqual({
      edit: false,
      remove: false,
      addChildren: false,
      drag: false,
      toggleActive: false,
      overrideColor: false,
    });
  });
});

describe("editing preserves the all-day contract", () => {
  it("keeps a moved all-day event date-only", () => {
    const moved = setWindow([allDayTask], allDayTask.id, {
      startMs: new Date(2026, 5, 25).getTime(),
      allDay: true,
    })[0];

    expect(moved.startAt).toBe("2026-06-25");
    expect(parseDateValue(moved.startAt).dateOnly).toBe(true);
    expect(classify(moved)).toBe("all-day");
  });

  it("does not turn an all-day event into a timed event for a minute nudge", () => {
    const startMs = effectiveStart(allDayTask).ms;
    const nudged = setWindow([allDayTask], allDayTask.id, {
      startMs: startMs + 15 * 60 * 1000,
      allDay: true,
    })[0];

    expect(nudged.startAt).toBe("2026-06-22");
    expect(classify(nudged)).toBe("all-day");
  });

  it("keeps a moved timed event a timestamp", () => {
    const startMs = effectiveStart(timedTask).ms;
    const endMs = effectiveEnd(timedTask, startMs).ms;
    expect(endMs).not.toBeNull();

    const moved = setWindow([timedTask], timedTask.id, {
      startMs: startMs + 24.5 * 60 * 60 * 1000,
      endMs: (endMs ?? startMs) + 24.5 * 60 * 60 * 1000,
      allDay: false,
    })[0];

    expect(parseDateValue(moved.startAt).dateOnly).toBe(false);
    expect(classify(moved)).toBe("timed");
    expect(parseDateValue(moved.startAt).ms).toBe(
      startMs + 24.5 * 60 * 60 * 1000,
    );
  });

  it("keeps both ends of an all-day span date-only when resized", () => {
    const resized = setWindow([spanTask], spanTask.id, {
      endMs: new Date(2026, 5, 27).getTime(),
      allDay: true,
    })[0];

    expect(resized.setAt).toBe("2026-06-22");
    expect(resized.expireAt).toBe("2026-06-27");
    expect(classify(resized)).toBe("all-day");
  });

  it("enforces a positive duration for timed resizes", () => {
    const startMs = effectiveStart(timedTask).ms;
    const resized = setWindow([timedTask], timedTask.id, {
      endMs: startMs - 60_000,
      allDay: false,
    })[0];

    expect(parseDateValue(resized.expireAt).ms).toBe(startMs + 60_000);
  });

  it("formats all-day and timed values in their matching shapes", () => {
    const date = new Date(2026, 5, 22, 13, 45, 0);

    expect(formatDateValue(date.getTime(), true)).toBe("2026-06-22");
    expect(
      parseDateValue(formatDateValue(date.getTime(), false)).dateOnly,
    ).toBe(false);
  });
});

describe("mock calendar data", () => {
  const events = buildMockCalendarEvents(new Date(2026, 5, 22));

  it("is deterministic for a given reference date", () => {
    expect(buildMockCalendarEvents(new Date(2026, 5, 22))).toEqual(events);
  });

  it("covers every shape the views have to render", () => {
    const kinds = new Set(events.map((event) => classify(event)));

    expect(kinds).toContain("all-day");
    expect(kinds).toContain("timed");
    expect(kinds).toContain("milestone");
    expect(
      events.some((event) => {
        const start = effectiveStart(event);
        const end = effectiveEnd(event, start.ms);
        return (
          end.ms !== null &&
          start.dateOnly &&
          end.dateOnly &&
          end.ms !== start.ms
        );
      }),
    ).toBe(true);
  });

  it("keeps every id unique", () => {
    const ids = events.map((event) => event.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("never serializes an all-day event as a timestamp", () => {
    for (const event of events) {
      if (classify(event) === "all-day") {
        const start = effectiveStart(event);
        const end = effectiveEnd(event, start.ms);
        expect(start.dateOnly).toBe(true);
        if (end.ms !== null) {
          expect(end.dateOnly).toBe(true);
        }
      }
    }
  });
});
