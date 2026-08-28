import { addDays, addMinutes } from "date-fns"

import {
  formatTaskDate,
  isDateOnly,
  parseTaskDate,
  shiftTask,
} from "../classify"
import type { CalendarDraftRange, CalendarEditing, TaskItem } from "../types"

function nextId(): string {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `task-${Date.now()}-${Math.round(Math.random() * 1e6)}`
}

/**
 * The opt-in mutation slice.
 *
 * Every operation preserves the all-day vs timed serialization of the task it
 * touches — that string format is what the calendar classifies on, so losing it
 * silently converts an all-day event into a midnight block.
 */
export const calendarEditing: CalendarEditing = {
  moveTask(task, dayDelta, minuteDelta) {
    return shiftTask(task, dayDelta, minuteDelta)
  },

  resizeTask(task, edge, dayDelta, minuteDelta) {
    const allDay = isDateOnly(task.startDate)
    const start = parseTaskDate(task.startDate)
    const end = task.endDate
      ? parseTaskDate(task.endDate)
      : addMinutes(start, allDay ? 0 : 60)

    const shift = (date: Date) =>
      addMinutes(addDays(date, dayDelta), allDay ? 0 : minuteDelta)

    if (edge === "start") {
      const nextStart = shift(start)
      if (nextStart >= end) {
        return task
      }
      return {
        ...task,
        startDate: formatTaskDate(nextStart, allDay),
      }
    }

    const nextEnd = shift(end)
    if (nextEnd <= start) {
      return task
    }

    return {
      ...task,
      endDate: formatTaskDate(nextEnd, isDateOnly(task.endDate ?? task.startDate)),
    }
  },

  createTask(range: CalendarDraftRange): TaskItem {
    return {
      id: nextId(),
      title: "New event",
      startDate: formatTaskDate(range.start, range.allDay),
      endDate: formatTaskDate(range.end, range.allDay),
    }
  },

  duplicateTask(task) {
    return { ...task, id: nextId(), title: `${task.title} (copy)` }
  },
}
