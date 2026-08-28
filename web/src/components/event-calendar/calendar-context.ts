import { createContext, useContext } from "react"
import type { ReactNode } from "react"

import type { ResolvedTask } from "./classify"
import type {
  CalendarDraftRange,
  CalendarEditing,
  CalendarOption,
  CalendarRange,
  CalendarView,
  TaskItem,
  TaskPermissions,
} from "./types"

export type CalendarClipboard = {
  task: TaskItem
  cut: boolean
}

export type CalendarContextValue = {
  view: CalendarView
  setView: (view: CalendarView) => void
  anchorDate: Date
  setAnchorDate: (date: Date) => void
  goToToday: () => void
  step: (direction: -1 | 1) => void
  now: Date
  range: CalendarRange
  data: readonly TaskItem[]
  resolved: readonly ResolvedTask[]
  statusColors: Record<string, string>
  statusOptions: readonly CalendarOption[]
  priorityOptions: readonly CalendarOption[]
  labelOptions: readonly CalendarOption[]
  showMiniNav: boolean
  editable: boolean
  editing?: CalendarEditing
  permissions: TaskPermissions
  renderTooltip?: (task: TaskItem) => ReactNode
  focusedTaskId: string | null
  setFocusedTaskId: (id: string | null) => void
  selectTask: (task: TaskItem) => void
  updateTask: (task: TaskItem) => void
  removeTask: (taskId: string) => void
  createTaskAt: (range: CalendarDraftRange) => void
  clipboard: CalendarClipboard | null
  setClipboard: (clipboard: CalendarClipboard | null) => void
  pasteAt: (day: Date) => void
  requestEdit: (task: TaskItem) => void
  requestDelete: (task: TaskItem) => void
  colorForTask: (task: TaskItem) => string
}

export const CalendarContext = createContext<CalendarContextValue | null>(null)

export function useCalendar(): CalendarContextValue {
  const value = useContext(CalendarContext)

  if (!value) {
    throw new Error("useCalendar must be used inside <EventCalendarRoot>")
  }

  return value
}
