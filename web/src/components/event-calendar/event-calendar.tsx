import { useCallback, useEffect, useState } from "react"
import {
  DndContext,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core"
import type { DragEndEvent } from "@dnd-kit/core"
import { differenceInCalendarDays, startOfDay } from "date-fns"

import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { cn } from "@/lib/utils"

import { CalendarAgendaView } from "./calendar-agenda-view"
import { useCalendar } from "./calendar-context"
import { CalendarDayView } from "./calendar-day-view"
import { CalendarMonthView } from "./calendar-month-view"
import { CalendarToolbar } from "./calendar-toolbar"
import { CalendarWeekView } from "./calendar-week-view"
import { PIXELS_PER_MINUTE, SNAP_MINUTES } from "./calendar-event"
import { EventCalendarRoot } from "./calendar-root"
import { parseTaskDate } from "./classify"
import type { EventCalendarProps, TaskItem } from "./types"

/** Single-letter shortcuts must never fire while someone is typing. */
function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false
  }

  return (
    target.isContentEditable ||
    ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName)
  )
}

function CalendarKeyboard() {
  const {
    clipboard,
    data,
    editable,
    editing,
    focusedTaskId,
    goToToday,
    anchorDate,
    pasteAt,
    permissions,
    requestDelete,
    requestEdit,
    setClipboard,
    setView,
    step,
    updateTask,
  } = useCalendar()

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.defaultPrevented || isTypingTarget(event.target)) {
        return
      }

      const focused = focusedTaskId
        ? data.find((task) => task.id === focusedTaskId)
        : undefined

      if (event.metaKey || event.ctrlKey) {
        const key = event.key.toLowerCase()
        if (focused && key === "c" && permissions.copy) {
          setClipboard({ task: focused, cut: false })
          event.preventDefault()
        }
        if (focused && key === "x" && permissions.copy && permissions.move) {
          setClipboard({ task: focused, cut: true })
          event.preventDefault()
        }
        if (clipboard && key === "v") {
          pasteAt(anchorDate)
          event.preventDefault()
        }
        return
      }

      if (event.altKey) {
        return
      }

      const key = event.key

      if (focused && editable && editing) {
        if (key === "Enter" || key === "F2") {
          if (permissions.edit) {
            requestEdit(focused)
            event.preventDefault()
          }
          return
        }

        if (key === "Delete" || key === "Backspace") {
          if (permissions.delete) {
            requestDelete(focused)
            event.preventDefault()
          }
          return
        }

        const horizontal = key === "ArrowLeft" ? -1 : key === "ArrowRight" ? 1 : 0
        const vertical = key === "ArrowUp" ? -1 : key === "ArrowDown" ? 1 : 0

        if (horizontal !== 0 || vertical !== 0) {
          if (event.shiftKey) {
            if (permissions.resize) {
              updateTask(
                editing.resizeTask(
                  focused,
                  "end",
                  horizontal,
                  vertical * SNAP_MINUTES,
                ),
              )
              event.preventDefault()
            }
            return
          }

          if (permissions.move) {
            updateTask(
              editing.moveTask(focused, horizontal, vertical * SNAP_MINUTES),
            )
            event.preventDefault()
          }
          return
        }
      }

      switch (key.toLowerCase()) {
        case "m":
          setView("month")
          break
        case "w":
          setView("week")
          break
        case "d":
          setView("day")
          break
        case "a":
          setView("agenda")
          break
        case "t":
          goToToday()
          break
        default:
          break
      }

      if (key === "ArrowLeft" || key === "PageUp") {
        step(-1)
      }
      if (key === "ArrowRight" || key === "PageDown") {
        step(1)
      }
    }

    document.addEventListener("keydown", handler)
    return () => document.removeEventListener("keydown", handler)
  }, [
    anchorDate,
    clipboard,
    data,
    editable,
    editing,
    focusedTaskId,
    goToToday,
    pasteAt,
    permissions,
    requestDelete,
    requestEdit,
    setClipboard,
    setView,
    step,
    updateTask,
  ])

  return null
}

function CalendarViewport({ className }: { className?: string }) {
  const { data, editable, editing, permissions, updateTask, view } = useCalendar()
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
  )

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const dropData = event.over?.data.current as
        | { day: string; allDay: boolean }
        | undefined

      if (!dropData || !editing || !permissions.move) {
        return
      }

      const task = data.find((entry) => entry.id === event.active.id)
      if (!task) {
        return
      }

      const dayDelta = differenceInCalendarDays(
        startOfDay(new Date(dropData.day)),
        startOfDay(parseTaskDate(task.startDate)),
      )
      const rawMinutes = event.delta.y / PIXELS_PER_MINUTE
      const minuteDelta = dropData.allDay
        ? 0
        : Math.round(rawMinutes / SNAP_MINUTES) * SNAP_MINUTES

      if (dayDelta === 0 && minuteDelta === 0) {
        return
      }

      updateTask(editing.moveTask(task, dayDelta, minuteDelta))
    },
    [data, editing, permissions.move, updateTask],
  )

  const surface = (
    <div
      className={cn(
        "flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-border bg-card",
        className,
      )}
    >
      <CalendarToolbar />
      {view === "month" ? <CalendarMonthView /> : null}
      {view === "week" ? <CalendarWeekView /> : null}
      {view === "day" ? <CalendarDayView /> : null}
      {view === "agenda" ? <CalendarAgendaView /> : null}
    </div>
  )

  if (!editable) {
    return surface
  }

  return (
    <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
      {surface}
    </DndContext>
  )
}

function EditTaskDialog({
  task,
  onClose,
}: {
  task: TaskItem | null
  onClose: () => void
}) {
  const { priorityOptions, statusOptions, updateTask } = useCalendar()
  const [draft, setDraft] = useState<TaskItem | null>(task)

  useEffect(() => {
    setDraft(task)
  }, [task])

  return (
    <Dialog open={Boolean(task)} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Edit event</DialogTitle>
          <DialogDescription>
            Changes are kept in memory only — there is no events API yet.
          </DialogDescription>
        </DialogHeader>

        {draft ? (
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="calendar-event-title">Title</Label>
              <Input
                id="calendar-event-title"
                value={draft.title}
                onChange={(event) =>
                  setDraft({ ...draft, title: event.target.value })
                }
              />
            </div>

            {statusOptions.length > 0 ? (
              <div className="flex flex-col gap-2">
                <Label>Status</Label>
                <Select
                  value={draft.status ?? ""}
                  onValueChange={(value) => setDraft({ ...draft, status: value })}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select a status" />
                  </SelectTrigger>
                  <SelectContent>
                    {statusOptions.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            ) : null}

            {priorityOptions.length > 0 ? (
              <div className="flex flex-col gap-2">
                <Label>Priority</Label>
                <Select
                  value={draft.priority ?? ""}
                  onValueChange={(value) => setDraft({ ...draft, priority: value })}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select a priority" />
                  </SelectTrigger>
                  <SelectContent>
                    {priorityOptions.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            ) : null}
          </div>
        ) : null}

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={() => {
              if (draft) {
                updateTask(draft)
              }
              onClose()
            }}
          >
            Save
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function DeleteTaskDialog({
  task,
  onClose,
}: {
  task: TaskItem | null
  onClose: () => void
}) {
  const { removeTask } = useCalendar()

  return (
    <Dialog open={Boolean(task)} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Delete event?</DialogTitle>
          <DialogDescription>
            {/* Deliberate: the keyboard path deletes on a single Delete press
                and nothing here can be undone, so it routes through a confirm. */}
            “{task?.title}” will be removed. There is no undo.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            onClick={() => {
              if (task) {
                removeTask(task.id)
              }
              onClose()
            }}
          >
            Delete
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function CalendarSurface({
  className,
  editingTask,
  deletingTask,
  onCloseEdit,
  onCloseDelete,
}: {
  className?: string
  editingTask: TaskItem | null
  deletingTask: TaskItem | null
  onCloseEdit: () => void
  onCloseDelete: () => void
}) {
  return (
    <>
      <CalendarKeyboard />
      <CalendarViewport className={className} />
      <EditTaskDialog task={editingTask} onClose={onCloseEdit} />
      <DeleteTaskDialog task={deletingTask} onClose={onCloseDelete} />
    </>
  )
}

export function EventCalendar(props: EventCalendarProps) {
  const [editingTask, setEditingTask] = useState<TaskItem | null>(null)
  const [deletingTask, setDeletingTask] = useState<TaskItem | null>(null)

  /**
   * Opened on the next tick.
   *
   * A context-menu item that opens a dialog in the same tick leaves the menu
   * and the dialog fighting over the same `body { pointer-events: none }` lock,
   * and the release order can end with the lock still applied — the page then
   * ignores every click. Letting the menu unmount first avoids the race.
   */
  const openLater = (setter: (task: TaskItem | null) => void) => (task: TaskItem) => {
    window.setTimeout(() => setter(task), 0)
  }

  return (
    <EventCalendarRoot
      {...props}
      onRequestEdit={openLater(setEditingTask)}
      onRequestDelete={openLater(setDeletingTask)}
    >
      <CalendarSurface
        className={props.className}
        editingTask={editingTask}
        deletingTask={deletingTask}
        onCloseEdit={() => setEditingTask(null)}
        onCloseDelete={() => setDeletingTask(null)}
      />
    </EventCalendarRoot>
  )
}
