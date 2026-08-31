import { Inbox } from "lucide-react";

import {
  priorityOptions,
  statusOptions,
} from "@/components/calendar/calendar.config";
import { formatTaskSchedule } from "@/components/calendar/calendar-task-view-data";
import type { CurioCalendarEvent } from "@/components/calendar/calendar.types";
import { DataTable, type DataTableColumn } from "@/components/data-table";
import { Badge } from "@/components/ui/badge";

const STATUS_OPTIONS = new Map(
  statusOptions.map((option) => [option.value, option]),
);
const PRIORITY_OPTIONS = new Map(
  priorityOptions.map((option) => [option.value, option]),
);

const SOURCE_LABELS: Record<
  NonNullable<CurioCalendarEvent["source"]>,
  string
> = {
  user: "Personal",
  vault: "Vault",
  chat: "Chat",
};

const TASK_COLUMNS: DataTableColumn<CurioCalendarEvent>[] = [
  {
    id: "task",
    header: "Task",
    width: "42%",
    accessor: (task) => (
      <div className="min-w-56 max-w-[36rem]">
        <p className="truncate font-medium text-foreground">{task.name}</p>
        {task.description ? (
          <p className="mt-0.5 truncate text-xs text-muted-foreground">
            {task.description}
          </p>
        ) : null}
      </div>
    ),
  },
  {
    id: "status",
    header: "Status",
    accessor: (task) => {
      const status = STATUS_OPTIONS.get(task.status);
      return (
        <Badge variant={status?.variant ?? "outline"}>
          {status?.label ?? task.status}
        </Badge>
      );
    },
  },
  {
    id: "priority",
    header: "Priority",
    accessor: (task) => {
      const priority = task.priority
        ? PRIORITY_OPTIONS.get(task.priority)
        : undefined;

      return priority ? (
        <Badge
          variant="outline"
          style={{
            borderColor: priority.color,
            color: priority.color,
          }}
        >
          {priority.label}
        </Badge>
      ) : (
        <span className="text-muted-foreground">—</span>
      );
    },
  },
  {
    id: "schedule",
    header: "Schedule",
    accessor: (task) => (
      <span className="text-muted-foreground">{formatTaskSchedule(task)}</span>
    ),
  },
  {
    id: "source",
    header: "Source",
    accessor: (task) => (
      <span className="text-muted-foreground">
        {task.source ? SOURCE_LABELS[task.source] : "Personal"}
      </span>
    ),
  },
];

export default function CalendarTaskList({
  tasks,
}: {
  tasks: CurioCalendarEvent[];
}) {
  return (
    <section
      aria-label="Task list"
      className="h-full overflow-y-auto bg-background p-3 sm:p-4"
    >
      <DataTable
        columns={TASK_COLUMNS}
        rows={tasks}
        rowKey={(task) => task.id}
        caption={`${tasks.length} ${tasks.length === 1 ? "task" : "tasks"} in the current calendar range`}
        emptyState={
          <div className="flex flex-col items-center gap-2 py-4">
            <span className="flex size-10 items-center justify-center rounded-full bg-muted">
              <Inbox className="size-4" aria-hidden="true" />
            </span>
            <span>No tasks in the current calendar range.</span>
          </div>
        }
      />
    </section>
  );
}
