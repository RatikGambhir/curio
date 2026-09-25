import { lazy, Suspense, useCallback, useState, type FormEvent } from "react";

import type { CalendarView } from "@/api/calendar";
import {
  calendarPermissions,
  labelOptions,
  priorityOptions,
  statusColors,
  statusOptions,
} from "@/components/calendar/calendar.config";
import type {
  CalendarPriority,
  CalendarStatus,
  CurioCalendarEvent,
} from "@/components/calendar/calendar.types";
import {
  CalendarViewSwitcher,
  type CalendarPageView,
  type TaskLayout,
} from "@/components/calendar/calendar-view-switcher";
import type {
  CalendarQuickComposerRenderer,
  TaskItem,
  TaskItemAddedEvent,
} from "@/components/event-calendar";
import { EventCalendar } from "@/components/event-calendar";
import { calendarEditing } from "@/components/event-calendar/features/editing";
import { PageHeader } from "@/components/page-header";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Notice } from "@/components/ui/notice";
import { Spinner } from "@/components/ui/spinner";
import {
  useCalendarEvents,
  useCreateCalendarEvent,
  type VisibleCalendarRange,
} from "@/hooks/useCalendarEvents";

const DATE_ONLY = /^\d{4}-\d{2}-\d{2}$/;
const EMPTY_EVENTS: CurioCalendarEvent[] = [];
const EVENT_DATE_FORMATTER = new Intl.DateTimeFormat(undefined, {
  weekday: "short",
  month: "short",
  day: "numeric",
});
const EVENT_TIME_FORMATTER = new Intl.DateTimeFormat(undefined, {
  hour: "numeric",
  minute: "2-digit",
});

const CalendarTaskViews = lazy(
  () => import("@/components/calendar/calendar-task-views"),
);

type CalendarCreateFormProps = Parameters<CalendarQuickComposerRenderer>[0];

function CalendarCreateForm({
  date,
  allDay,
  defaultEnd,
  commit,
  cancel,
}: CalendarCreateFormProps) {
  const [title, setTitle] = useState("");
  const [error, setError] = useState<string | null>(null);
  const when = allDay
    ? `${EVENT_DATE_FORMATTER.format(date)} · all day`
    : `${EVENT_DATE_FORMATTER.format(date)} · ${EVENT_TIME_FORMATTER.format(date)}–${EVENT_TIME_FORMATTER.format(defaultEnd)}`;

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedTitle = title.trim();
    if (!normalizedTitle) {
      setError("Enter a title for the event.");
      return;
    }
    commit({ name: normalizedTitle });
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-2">
      <p className="text-xs text-muted-foreground">{when}</p>
      <Input
        autoFocus
        value={title}
        onChange={(event) => {
          setTitle(event.target.value);
          setError(null);
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            cancel();
          }
        }}
        aria-invalid={Boolean(error)}
        aria-describedby={error ? "calendar-create-error" : undefined}
        placeholder="Event title"
        className="h-8"
      />
      {error ? (
        <p
          id="calendar-create-error"
          role="alert"
          className="text-xs text-destructive"
        >
          {error}
        </p>
      ) : null}
      <div className="flex justify-end gap-1.5">
        <Button type="button" size="sm" variant="ghost" onClick={cancel}>
          Cancel
        </Button>
        <Button type="submit" size="sm">
          Create
        </Button>
      </div>
    </form>
  );
}

const isHighPriority = (item: TaskItem) => item.priority === "high";
const isCalendarStatus = (value: string): value is CalendarStatus =>
  statusOptions.some((option) => option.value === value);
const isCalendarPriority = (
  value: string | undefined,
): value is CalendarPriority =>
  value !== undefined &&
  priorityOptions.some((option) => option.value === value);

const Calendar = () => {
  // Stable identity: an inline `new Date()` would take a fresh identity every
  // render and drive onRangeChange into a loop.
  const [now] = useState(() => new Date());
  const [pageView, setPageView] = useState<CalendarPageView>("calendar");
  const [taskLayout, setTaskLayout] = useState<TaskLayout>("list");
  const [calendarView, setCalendarView] = useState<CalendarView>("month");
  const [calendarDate, setCalendarDate] = useState(() => now);
  const [visibleRange, setVisibleRange] = useState<VisibleCalendarRange | null>(
    null,
  );
  const calendarQuery = useCalendarEvents(visibleRange);
  const { mutate: createEvent } = useCreateCalendarEvent();
  const events = calendarQuery.data?.events ?? EMPTY_EVENTS;
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);
  const loadErrorMessage = calendarQuery.isError
    ? calendarQuery.error instanceof Error
      ? calendarQuery.error.message
      : "Please try again."
    : undefined;
  const taskErrorMessage = events.length === 0 ? loadErrorMessage : undefined;

  const handleTaskClick = useCallback((task: TaskItem) => {
    setSelectedEventId(task.id);
  }, []);

  const handleRangeChange = useCallback(
    (range: { view: CalendarView; start: Date; end: Date }) => {
      setVisibleRange({
        view: range.view,
        start: range.start.toISOString(),
        end: range.end.toISOString(),
      });
    },
    [],
  );

  const handleItemAdded = useCallback(
    ({ item }: TaskItemAddedEvent) => {
      const startDate = item.startAt ?? item.setAt;
      createEvent({
        id: item.id,
        title: item.name,
        description: item.description ?? null,
        status: isCalendarStatus(item.status) ? item.status : null,
        priority: isCalendarPriority(item.priority) ? item.priority : null,
        allDay: DATE_ONLY.test(startDate),
        startDate,
        endDate: item.expireAt ?? null,
      });
    },
    [createEvent],
  );

  const renderCreateComposer = useCallback<CalendarQuickComposerRenderer>(
    (props) => <CalendarCreateForm {...props} />,
    [],
  );

  const renderTooltip = useCallback(
    (task: TaskItem) => (
      <div className="flex flex-col gap-1">
        <span className="font-medium">{task.name}</span>
        {task.description ? (
          <span className="text-muted-foreground">{task.description}</span>
        ) : null}
        <span className="text-xs text-muted-foreground">
          {task.status ?? "unscheduled"}
          {task.priority ? ` · ${task.priority} priority` : ""}
        </span>
      </div>
    ),
    [],
  );

  return (
    <>
      <PageHeader
        title="Calendar"
        actions={
          <CalendarViewSwitcher
            pageView={pageView}
            taskLayout={taskLayout}
            onPageViewChange={setPageView}
            onTaskLayoutChange={setTaskLayout}
          />
        }
      />
      {pageView === "calendar" && loadErrorMessage ? (
        <Notice
          tone="error"
          title="Events could not be loaded"
          action={
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={calendarQuery.isFetching}
              onClick={() => void calendarQuery.refetch()}
            >
              {calendarQuery.isFetching ? "Retrying…" : "Retry"}
            </Button>
          }
          className="mx-3 mt-3 sm:mx-5"
        >
          {loadErrorMessage} Anything you add now may not be saved.
        </Notice>
      ) : null}
      <div className="min-h-0 min-w-0 flex-1 overflow-hidden">
        {pageView === "calendar" ? (
          <EventCalendar
            data={events}
            onItemAdded={handleItemAdded}
            statusOptions={statusOptions}
            priorityOptions={priorityOptions}
            labelOptions={labelOptions}
            statusColors={statusColors}
            flagPriority={isHighPriority}
            view={calendarView}
            onViewChange={setCalendarView}
            date={calendarDate}
            onDateChange={setCalendarDate}
            now={now}
            showMiniNav
            editable
            editing={calendarEditing}
            permissions={calendarPermissions}
            renderQuickComposer={renderCreateComposer}
            selectedId={selectedEventId}
            onSelect={setSelectedEventId}
            onTaskClick={handleTaskClick}
            onRangeChange={handleRangeChange}
            renderTooltip={renderTooltip}
            className="h-full rounded-none border-0"
          />
        ) : (
          <Suspense
            fallback={
              <div
                role="status"
                className="flex h-full items-center justify-center gap-2 text-sm text-muted-foreground"
              >
                <Spinner aria-hidden="true" />
                <span>Loading tasks…</span>
              </div>
            }
          >
            <CalendarTaskViews
              layout={taskLayout}
              tasks={events}
              isLoading={calendarQuery.isPending && events.length === 0}
              errorMessage={taskErrorMessage}
            />
          </Suspense>
        )}
      </div>
    </>
  );
};

export default Calendar;
