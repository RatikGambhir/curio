import { describe, expect, it } from "vitest";

import { statusOptions } from "@/components/calendar/calendar.config";
import {
  buildTaskKanbanData,
  formatTaskSchedule,
  taskKanbanPalette,
} from "@/components/calendar/calendar-task-view-data";
import type { CurioCalendarEvent } from "@/components/calendar/calendar.types";
import type { KanbanCardData } from "@/components/kanban-board";

function task(
  id: string,
  overrides: Partial<CurioCalendarEvent> = {},
): CurioCalendarEvent {
  return {
    id,
    name: `Task ${id}`,
    status: "scheduled",
    active: true,
    setAt: "2026-08-31",
    ...overrides,
  };
}

describe("task schedule formatting", () => {
  it("keeps date-only values on their local calendar day", () => {
    expect(formatTaskSchedule(task("one"))).toBe("Aug 31, 2026 · All day");
    expect(
      formatTaskSchedule(
        task("range", {
          expireAt: "2026-09-02",
        }),
      ),
    ).toBe("Aug 31–Sep 2, 2026 · All day");
  });

  it("formats timed tasks and same-day ranges compactly", () => {
    expect(
      formatTaskSchedule(
        task("timed", {
          setAt: "2026-08-31T14:00:00",
          expireAt: "2026-08-31T15:30:00",
        }),
      ),
    ).toBe("Aug 31, 2026 · 2:00 PM–3:30 PM");
  });
});

describe("task kanban data", () => {
  it("builds the configured status columns in order and groups tasks", () => {
    const data = buildTaskKanbanData([
      task("blocked", { status: "blocked" }),
      task("done", { status: "done" }),
      task("unknown", { status: "future-status" }),
    ]);

    expect(data.columns.map((column) => column.id)).toEqual(
      statusOptions.map((status) => status.value),
    );
    expect(
      data.columns.find((column) => column.id === "blocked")?.items[0]?.id,
    ).toBe("blocked");
    expect(
      data.columns.find((column) => column.id === "done")?.items[0]?.id,
    ).toBe("done");
    expect(
      data.columns.find((column) => column.id === "scheduled")?.items[0]?.id,
    ).toBe("unknown");
  });

  it("maps task details into the block's compact card model", () => {
    const data = buildTaskKanbanData([
      task("research", {
        name: "Review research",
        description: "Pull the strongest findings together.",
        priority: "high",
        labels: ["research", "custom"],
      }),
    ]);
    const card = data.columns[0]?.items[0]?.data as KanbanCardData;

    expect(card).toMatchObject({
      title: "Review research",
      description: "Pull the strongest findings together.",
      tags: [{ label: "High" }, { label: "Research" }, { label: "custom" }],
      meta: [
        {
          key: "schedule",
          label: "When",
          value: "Aug 31, 2026 · All day",
        },
      ],
    });
  });

  it("uses semantic theme tokens for every column accent", () => {
    expect(taskKanbanPalette.map((swatch) => swatch.id)).toEqual(
      statusOptions.map((status) => status.value),
    );
    expect(
      taskKanbanPalette.every((swatch) => swatch.cssVar.startsWith("--")),
    ).toBe(true);
  });
});
