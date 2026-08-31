import { format, isSameDay, isSameYear } from "date-fns";

import {
  labelOptions,
  priorityOptions,
  statusOptions,
} from "@/components/calendar/calendar.config";
import type { CurioCalendarEvent } from "@/components/calendar/calendar.types";
import type {
  KanbanCardData,
  KanbanData,
  KanbanPaletteSwatch,
} from "@/components/kanban-board";

const DATE_ONLY = /^(\d{4})-(\d{2})-(\d{2})$/;
const KANBAN_RENDERER_ID = "kanban-card";

const STATUS_PALETTE_IDS: Record<string, string> = {
  scheduled: "scheduled",
  "in-progress": "in-progress",
  blocked: "blocked",
  done: "done",
  cancelled: "cancelled",
};

const PRIORITY_LABELS = new Map(
  priorityOptions.map((option) => [option.value, option.label]),
);
const TASK_LABELS = new Map(
  labelOptions.map((option) => [option.value, option.label]),
);
const KNOWN_STATUSES = new Set(statusOptions.map((option) => option.value));

export const taskKanbanPalette: KanbanPaletteSwatch[] = [
  { id: "scheduled", label: "Scheduled", cssVar: "--chart-1" },
  { id: "in-progress", label: "In progress", cssVar: "--chart-2" },
  { id: "blocked", label: "Blocked", cssVar: "--destructive" },
  { id: "done", label: "Done", cssVar: "--chart-3" },
  { id: "cancelled", label: "Cancelled", cssVar: "--chart-4" },
];

function parseTaskDate(value: string): Date | null {
  const dateOnly = DATE_ONLY.exec(value);
  const parsed = dateOnly
    ? new Date(
        Number(dateOnly[1]),
        Number(dateOnly[2]) - 1,
        Number(dateOnly[3]),
      )
    : new Date(value);

  return Number.isNaN(parsed.getTime()) ? null : parsed;
}

function formatDateRange(start: Date, end: Date): string {
  if (isSameDay(start, end)) return format(start, "MMM d, yyyy");
  if (isSameYear(start, end)) {
    return `${format(start, "MMM d")}–${format(end, "MMM d, yyyy")}`;
  }
  return `${format(start, "MMM d, yyyy")}–${format(end, "MMM d, yyyy")}`;
}

export function formatTaskSchedule(task: CurioCalendarEvent): string {
  const startValue = task.startAt ?? task.setAt;
  const start = parseTaskDate(startValue);
  if (!start) return "Unscheduled";

  const end = task.expireAt ? parseTaskDate(task.expireAt) : null;
  const isAllDay = DATE_ONLY.test(startValue);

  if (isAllDay) {
    return `${end ? formatDateRange(start, end) : format(start, "MMM d, yyyy")} · All day`;
  }

  if (!end) return format(start, "MMM d, yyyy · h:mm a");
  if (isSameDay(start, end)) {
    return `${format(start, "MMM d, yyyy")} · ${format(start, "h:mm a")}–${format(end, "h:mm a")}`;
  }

  return `${format(start, "MMM d, yyyy · h:mm a")}–${format(end, "MMM d, yyyy · h:mm a")}`;
}

function toKanbanCardData(task: CurioCalendarEvent): KanbanCardData {
  const priority = task.priority
    ? (PRIORITY_LABELS.get(task.priority) ?? task.priority)
    : undefined;
  const tags = (task.labels ?? []).map((label) => ({
    label: TASK_LABELS.get(label) ?? label,
  }));

  if (priority) tags.unshift({ label: priority });

  return {
    title: task.name,
    description: task.description,
    tags: tags.length > 0 ? tags : undefined,
    assignees: task.targetPerson
      ? [
          {
            id: task.targetPerson.id,
            name: task.targetPerson.name,
            avatarUrl: task.targetPerson.avatar,
          },
        ]
      : undefined,
    meta: [
      {
        key: "schedule",
        label: "When",
        value: formatTaskSchedule(task),
      },
    ],
  };
}

export function buildTaskKanbanData(tasks: CurioCalendarEvent[]): KanbanData {
  const grouped = new Map<string, CurioCalendarEvent[]>(
    statusOptions.map((option) => [option.value, []]),
  );

  for (const task of tasks) {
    const status = KNOWN_STATUSES.has(task.status) ? task.status : "scheduled";
    grouped.get(status)?.push(task);
  }

  return {
    columns: statusOptions.map((status) => ({
      id: status.value,
      title: status.label,
      color: STATUS_PALETTE_IDS[status.value],
      items: (grouped.get(status.value) ?? []).map((task) => ({
        id: task.id,
        rendererId: KANBAN_RENDERER_ID,
        data: toKanbanCardData(task),
      })),
    })),
  };
}
