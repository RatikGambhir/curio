import type {
  TaskLabelOption,
  TaskPermissions,
  TaskPriorityOption,
  TaskStatusOption,
} from "@/components/event-calendar";

export const statusOptions: TaskStatusOption[] = [
  {
    value: "scheduled",
    label: "Scheduled",
    tone: "active",
    variant: "outline",
  },
  {
    value: "in-progress",
    label: "In progress",
    tone: "active",
    variant: "secondary",
  },
  {
    value: "blocked",
    label: "Blocked",
    tone: "blocked",
    variant: "destructive",
    icon: "⛔",
  },
  {
    value: "done",
    label: "Done",
    tone: "done",
    variant: "secondary",
    icon: "✓",
  },
  { value: "cancelled", label: "Cancelled", tone: "done", variant: "outline" },
];

export const priorityOptions: TaskPriorityOption[] = [
  { value: "low", label: "Low", color: "var(--muted-foreground)" },
  { value: "medium", label: "Medium", color: "var(--chart-2)" },
  { value: "high", label: "High", color: "var(--destructive)" },
];

export const labelOptions: TaskLabelOption[] = [
  { value: "research", label: "Research", color: "var(--chart-1)" },
  { value: "writing", label: "Writing", color: "var(--chart-2)" },
  { value: "review", label: "Review", color: "var(--chart-3)" },
  { value: "ops", label: "Ops", color: "var(--chart-4)" },
];

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
};

/** Demo policy using the shared task-card permission matrix. */
export const calendarPermissions: TaskPermissions = {
  default: {
    edit: true,
    remove: true,
    addChildren: true,
    drag: true,
    toggleActive: true,
    overrideColor: true,
  },
};
