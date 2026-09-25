import { CalendarDays, Columns3, List, ListTodo } from "lucide-react";

import {
  SegmentedControl,
  type SegmentedOption,
} from "@/components/ui/segmented-control";
import { cn } from "@/lib/utils";

export type CalendarPageView = "calendar" | "tasks";
export type TaskLayout = "list" | "kanban";

const PAGE_VIEWS: SegmentedOption<CalendarPageView>[] = [
  { value: "calendar", label: "Calendar", icon: CalendarDays },
  { value: "tasks", label: "Tasks", icon: ListTodo },
];

const TASK_LAYOUTS: SegmentedOption<TaskLayout>[] = [
  { value: "list", label: "List", icon: List },
  { value: "kanban", label: "Board", icon: Columns3 },
];

export function CalendarViewSwitcher({
  pageView,
  taskLayout,
  onPageViewChange,
  onTaskLayoutChange,
  className,
}: {
  pageView: CalendarPageView;
  taskLayout: TaskLayout;
  onPageViewChange: (view: CalendarPageView) => void;
  onTaskLayoutChange: (layout: TaskLayout) => void;
  className?: string;
}) {
  return (
    <div className={cn("flex min-w-0 shrink-0 items-center gap-2", className)}>
      {pageView === "tasks" ? (
        <SegmentedControl
          label="Task layout"
          options={TASK_LAYOUTS}
          value={taskLayout}
          onValueChange={onTaskLayoutChange}
          hideLabelsBelow="md"
        />
      ) : null}
      <SegmentedControl
        label="Page view"
        options={PAGE_VIEWS}
        value={pageView}
        onValueChange={onPageViewChange}
        hideLabelsBelow="sm"
      />
    </div>
  );
}
