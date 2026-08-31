import {
  CalendarDays,
  Columns3,
  List,
  ListTodo,
  type LucideIcon,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import { cn } from "@/lib/utils";

export type CalendarPageView = "calendar" | "tasks";
export type TaskLayout = "list" | "kanban";

type ViewOption<TValue extends string> = {
  value: TValue;
  label: string;
  icon: LucideIcon;
};

const PAGE_VIEWS: ViewOption<CalendarPageView>[] = [
  { value: "calendar", label: "Calendar", icon: CalendarDays },
  { value: "tasks", label: "Tasks", icon: ListTodo },
];

const TASK_LAYOUTS: ViewOption<TaskLayout>[] = [
  { value: "list", label: "List", icon: List },
  { value: "kanban", label: "Kanban", icon: Columns3 },
];

function SegmentedControl<TValue extends string>({
  label,
  options,
  value,
  onValueChange,
}: {
  label: string;
  options: ViewOption<TValue>[];
  value: TValue;
  onValueChange: (value: TValue) => void;
}) {
  return (
    <ButtonGroup aria-label={label}>
      {options.map((option) => {
        const Icon = option.icon;
        const selected = value === option.value;

        return (
          <Button
            key={option.value}
            type="button"
            size="sm"
            variant={selected ? "default" : "outline"}
            aria-pressed={selected}
            aria-label={option.label}
            title={option.label}
            onClick={() => onValueChange(option.value)}
            className={cn(
              "h-7 gap-1.5 px-2.5 shadow-none hover:translate-y-0",
              !selected && "text-muted-foreground",
            )}
          >
            <Icon className="size-3.5" aria-hidden="true" />
            <span className="hidden sm:inline">{option.label}</span>
          </Button>
        );
      })}
    </ButtonGroup>
  );
}

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
    <div
      className={cn(
        "ml-auto flex min-w-0 shrink-0 items-center gap-2",
        className,
      )}
    >
      {pageView === "tasks" ? (
        <SegmentedControl
          label="Task layout"
          options={TASK_LAYOUTS}
          value={taskLayout}
          onValueChange={onTaskLayoutChange}
        />
      ) : null}
      <SegmentedControl
        label="Page view"
        options={PAGE_VIEWS}
        value={pageView}
        onValueChange={onPageViewChange}
      />
    </div>
  );
}
