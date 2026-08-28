import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import type { ReactNode } from "react"
import {
  addDays,
  addMonths,
  differenceInCalendarDays,
  endOfMonth,
  endOfWeek,
  startOfDay,
  startOfMonth,
  startOfWeek,
} from "date-fns"

import { parseTaskDate, resolveTasks, shiftTask } from "./classify"
import { CalendarContext } from "./calendar-context"
import type { CalendarClipboard, CalendarContextValue } from "./calendar-context"
import type {
  CalendarDraftRange,
  CalendarRange,
  CalendarView,
  EventCalendarProps,
  TaskItem,
} from "./types"

const AGENDA_DAYS = 30

const defaultStatusColors: Record<string, string> = {}

function rangeForView(view: CalendarView, anchor: Date): CalendarRange {
  if (view === "month") {
    return {
      view,
      start: startOfWeek(startOfMonth(anchor)),
      end: addDays(endOfWeek(endOfMonth(anchor)), 1),
    }
  }

  if (view === "week") {
    return {
      view,
      start: startOfWeek(anchor),
      end: addDays(endOfWeek(anchor), 1),
    }
  }

  if (view === "day") {
    return { view, start: startOfDay(anchor), end: addDays(startOfDay(anchor), 1) }
  }

  return {
    view,
    start: startOfDay(anchor),
    end: addDays(startOfDay(anchor), AGENDA_DAYS),
  }
}

function stepAnchor(view: CalendarView, anchor: Date, direction: -1 | 1): Date {
  if (view === "month") {
    return addMonths(anchor, direction)
  }
  if (view === "week") {
    return addDays(anchor, 7 * direction)
  }
  if (view === "day") {
    return addDays(anchor, direction)
  }
  return addDays(anchor, AGENDA_DAYS * direction)
}

export type EventCalendarRootProps = EventCalendarProps & {
  children: ReactNode
  onRequestEdit?: (task: TaskItem) => void
  onRequestDelete?: (task: TaskItem) => void
}

/**
 * Headless state container. It owns the view, the anchor date, selection and
 * mutation plumbing; every view component reads it through `useCalendar`.
 */
export function EventCalendarRoot({
  children,
  classifyEvent,
  data,
  defaultView = "month",
  editable = false,
  editing,
  labelOptions = [],
  now,
  onChange,
  onRangeChange,
  onRequestDelete,
  onRequestEdit,
  onTaskClick,
  permissions = {},
  priorityOptions = [],
  renderTooltip,
  showMiniNav = false,
  statusColors = defaultStatusColors,
  statusOptions = [],
}: EventCalendarRootProps) {
  // A stable "today" — an inline `new Date()` would take a fresh identity on
  // every render and push onRangeChange into a loop.
  const [fallbackNow] = useState(() => new Date())
  const resolvedNow = now ?? fallbackNow

  const [view, setView] = useState<CalendarView>(defaultView)
  const [anchorDate, setAnchorDate] = useState<Date>(() => resolvedNow)
  const [focusedTaskId, setFocusedTaskId] = useState<string | null>(null)
  const [clipboard, setClipboard] = useState<CalendarClipboard | null>(null)

  const range = useMemo(() => rangeForView(view, anchorDate), [anchorDate, view])
  const resolved = useMemo(
    () => resolveTasks(data, classifyEvent),
    [classifyEvent, data],
  )

  const rangeCallback = useRef(onRangeChange)
  rangeCallback.current = onRangeChange

  useEffect(() => {
    rangeCallback.current?.({ start: range.start, end: range.end, view })
    // Primitive deps: the range object is rebuilt each render.
  }, [range.start, range.end, view])

  const commit = useCallback(
    (next: TaskItem[]) => {
      onChange?.(next)
    },
    [onChange],
  )

  const updateTask = useCallback(
    (task: TaskItem) => {
      commit(data.map((entry) => (entry.id === task.id ? task : entry)))
    },
    [commit, data],
  )

  const removeTask = useCallback(
    (taskId: string) => {
      commit(data.filter((entry) => entry.id !== taskId))
    },
    [commit, data],
  )

  const createTaskAt = useCallback(
    (draft: CalendarDraftRange) => {
      if (!editing || !permissions.create) {
        return
      }
      commit([...data, editing.createTask(draft)])
    },
    [commit, data, editing, permissions.create],
  )

  const pasteAt = useCallback(
    (day: Date) => {
      if (!clipboard || !editing || !permissions.create) {
        return
      }

      const source = clipboard.task
      const dayDelta = differenceInCalendarDays(
        startOfDay(day),
        startOfDay(parseTaskDate(source.startDate)),
      )
      const moved = shiftTask(source, dayDelta, 0)

      if (clipboard.cut) {
        commit(
          data.map((entry) => (entry.id === source.id ? moved : entry)),
        )
        setClipboard(null)
        return
      }

      commit([...data, editing.duplicateTask(moved)])
    },
    [clipboard, commit, data, editing, permissions.create],
  )

  const colorForTask = useCallback(
    (task: TaskItem) =>
      (task.status && statusColors[task.status]) || "var(--chart-1)",
    [statusColors],
  )

  const value: CalendarContextValue = useMemo(
    () => ({
      view,
      setView,
      anchorDate,
      setAnchorDate,
      goToToday: () => setAnchorDate(resolvedNow),
      step: (direction: -1 | 1) =>
        setAnchorDate((current) => stepAnchor(view, current, direction)),
      now: resolvedNow,
      range,
      data,
      resolved,
      statusColors,
      statusOptions,
      priorityOptions,
      labelOptions,
      showMiniNav,
      editable: editable && Boolean(editing),
      editing,
      permissions,
      renderTooltip,
      focusedTaskId,
      setFocusedTaskId,
      selectTask: (task: TaskItem) => {
        setFocusedTaskId(task.id)
        onTaskClick?.(task)
      },
      updateTask,
      removeTask,
      createTaskAt,
      clipboard,
      setClipboard,
      pasteAt,
      requestEdit: (task: TaskItem) => onRequestEdit?.(task),
      requestDelete: (task: TaskItem) => onRequestDelete?.(task),
      colorForTask,
    }),
    [
      anchorDate,
      clipboard,
      colorForTask,
      createTaskAt,
      data,
      editable,
      editing,
      focusedTaskId,
      labelOptions,
      onRequestDelete,
      onRequestEdit,
      onTaskClick,
      pasteAt,
      permissions,
      priorityOptions,
      range,
      removeTask,
      renderTooltip,
      resolved,
      resolvedNow,
      showMiniNav,
      statusColors,
      statusOptions,
      updateTask,
      view,
    ],
  )

  return (
    <CalendarContext.Provider value={value}>{children}</CalendarContext.Provider>
  )
}
