import { lazy, Suspense } from "react";
import { CircleAlert } from "lucide-react";

import CalendarTaskList from "@/components/calendar/calendar-task-list";
import type { TaskLayout } from "@/components/calendar/calendar-view-switcher";
import type { CurioCalendarEvent } from "@/components/calendar/calendar.types";
import { Spinner } from "@/components/ui/spinner";

const CalendarTaskKanban = lazy(
  () => import("@/components/calendar/calendar-task-kanban"),
);

function TaskViewLoading({ label = "Loading tasks" }: { label?: string }) {
  return (
    <div
      role="status"
      className="flex h-full items-center justify-center gap-2 bg-background text-sm text-muted-foreground"
    >
      <Spinner aria-hidden="true" />
      <span>{label}…</span>
    </div>
  );
}

export default function CalendarTaskViews({
  layout,
  tasks,
  isLoading,
  errorMessage,
}: {
  layout: TaskLayout;
  tasks: CurioCalendarEvent[];
  isLoading: boolean;
  errorMessage?: string;
}) {
  if (isLoading) return <TaskViewLoading />;

  if (errorMessage) {
    return (
      <div
        role="alert"
        className="flex h-full flex-col items-center justify-center gap-2 bg-background px-6 text-center"
      >
        <span className="flex size-10 items-center justify-center rounded-full bg-destructive/10 text-destructive">
          <CircleAlert className="size-4" aria-hidden="true" />
        </span>
        <p className="text-sm font-medium text-foreground">
          Tasks couldn&apos;t be loaded
        </p>
        <p className="max-w-md text-xs text-muted-foreground">{errorMessage}</p>
      </div>
    );
  }

  if (layout === "list") return <CalendarTaskList tasks={tasks} />;

  return (
    <Suspense fallback={<TaskViewLoading label="Loading board" />}>
      <CalendarTaskKanban tasks={tasks} />
    </Suspense>
  );
}
