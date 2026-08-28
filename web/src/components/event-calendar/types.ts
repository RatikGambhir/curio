import type { ReactNode } from "react"

export type CalendarView = "month" | "week" | "day" | "agenda"

/**
 * The canonical event record the calendar consumes.
 *
 * `startDate` / `endDate` carry a **string-format contract, not a Date
 * contract**: a bare `YYYY-MM-DD` marks an all-day event, a full ISO timestamp
 * marks a timed one. Serializing an all-day event as a timestamp silently
 * renders it as a midnight-anchored block, so the format has to survive every
 * round trip — including edits.
 */
export type TaskItem = {
  id: string
  title: string
  description?: string
  status?: string
  priority?: string
  labels?: string[]
  /** `YYYY-MM-DD` for all-day, full ISO timestamp for timed. */
  startDate: string
  /** Inclusive for all-day events. Absent on a timestamp means a milestone. */
  endDate?: string
  assignee?: { name: string; avatar?: string }
}

export type CalendarEventKind = "all-day" | "timed" | "milestone"

export type CalendarOption = {
  value: string
  label: string
}

/** Gates which mutations the editing slice offers. Start restrictive. */
export type TaskPermissions = {
  create?: boolean
  edit?: boolean
  move?: boolean
  resize?: boolean
  delete?: boolean
  copy?: boolean
}

export type CalendarRange = {
  start: Date
  end: Date
  view: CalendarView
}

/** Where a create gesture landed. `allDay` decides the serialized format. */
export type CalendarDraftRange = {
  start: Date
  end: Date
  allDay: boolean
}

/**
 * The mutation slice. Kept behind an interface so the base calendar stays
 * presentational and the editing bundle is opt-in.
 */
export type CalendarEditing = {
  moveTask: (task: TaskItem, dayDelta: number, minuteDelta: number) => TaskItem
  resizeTask: (
    task: TaskItem,
    edge: "start" | "end",
    dayDelta: number,
    minuteDelta: number,
  ) => TaskItem
  createTask: (range: CalendarDraftRange) => TaskItem
  duplicateTask: (task: TaskItem) => TaskItem
}

export type EventCalendarProps = {
  data: TaskItem[]
  statusOptions?: CalendarOption[]
  priorityOptions?: CalendarOption[]
  labelOptions?: CalendarOption[]
  /** Status value to CSS colour. Use theme tokens, never raw hex. */
  statusColors?: Record<string, string>
  defaultView?: CalendarView
  /**
   * Anchor for "today". Pass a value with stable identity — an inline
   * `new Date()` gets a fresh identity every render and drives
   * `onRangeChange` into a loop.
   */
  now?: Date
  showMiniNav?: boolean
  onTaskClick?: (task: TaskItem) => void
  onRangeChange?: (range: CalendarRange) => void
  editable?: boolean
  editing?: CalendarEditing
  onChange?: (data: TaskItem[]) => void
  permissions?: TaskPermissions
  renderTooltip?: (task: TaskItem) => ReactNode
  /** First classification layer: return undefined to fall through to format. */
  classifyEvent?: (task: TaskItem) => CalendarEventKind | undefined
  className?: string
}
