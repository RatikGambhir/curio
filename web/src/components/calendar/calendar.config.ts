import type { CalendarOption, TaskPermissions } from "@/components/event-calendar"

export const statusOptions: CalendarOption[] = [
  { value: "scheduled", label: "Scheduled" },
  { value: "in-progress", label: "In progress" },
  { value: "blocked", label: "Blocked" },
  { value: "done", label: "Done" },
  { value: "cancelled", label: "Cancelled" },
]

export const priorityOptions: CalendarOption[] = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
]

export const labelOptions: CalendarOption[] = [
  { value: "research", label: "Research" },
  { value: "writing", label: "Writing" },
  { value: "review", label: "Review" },
  { value: "ops", label: "Ops" },
]

/**
 * Theme tokens only — a raw hex would not flip with the dark block in
 * `src/index.css`. `--chart-1..5` is the existing oklch green/neutral ramp;
 * `--destructive` carries the one status that should read as a warning.
 */
export const statusColors: Record<string, string> = {
  scheduled: "var(--chart-1)",
  "in-progress": "var(--chart-2)",
  blocked: "var(--destructive)",
  done: "var(--chart-3)",
  cancelled: "var(--chart-4)",
}

/**
 * Restrictive by default, as the plan asks. Delete stays on but routes through
 * a confirm dialog, because nothing here is persisted and there is no undo.
 */
export const calendarPermissions: TaskPermissions = {
  create: true,
  edit: true,
  move: true,
  resize: true,
  copy: true,
  delete: true,
}
