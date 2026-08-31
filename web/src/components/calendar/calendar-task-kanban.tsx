import { useMemo } from "react";

import {
  buildTaskKanbanData,
  taskKanbanPalette,
} from "@/components/calendar/calendar-task-view-data";
import type { CurioCalendarEvent } from "@/components/calendar/calendar.types";
import { KanbanBoard, kanbanCardRenderer } from "@/components/kanban-board";

const TASK_RENDERERS = [kanbanCardRenderer];

export default function CalendarTaskKanban({
  tasks,
}: {
  tasks: CurioCalendarEvent[];
}) {
  const data = useMemo(() => buildTaskKanbanData(tasks), [tasks]);
  const dataVersion = useMemo(() => JSON.stringify(tasks), [tasks]);

  return (
    <section className="h-full min-h-0 bg-background" aria-label="Task board">
      <KanbanBoard
        key={dataVersion}
        defaultData={data}
        renderers={TASK_RENDERERS}
        palette={taskKanbanPalette}
        readOnly
        aria-label="Tasks grouped by status"
        className="h-full"
      />
    </section>
  );
}
