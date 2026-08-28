import { useRef } from "react"
import type { CSSProperties, PointerEvent as ReactPointerEvent } from "react"
import { useDraggable } from "@dnd-kit/core"
import { format } from "date-fns"

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@/components/ui/context-menu"
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card"
import { cn } from "@/lib/utils"

import { useCalendar } from "./calendar-context"
import type { ResolvedTask } from "./classify"

export const HOUR_HEIGHT = 48
export const PIXELS_PER_MINUTE = HOUR_HEIGHT / 60
/** Mutations snap to this, so dragging cannot produce 3-minute offsets. */
export const SNAP_MINUTES = 15

export type CalendarEventVariant = "bar" | "chip" | "block" | "row"

type CalendarEventProps = {
  resolved: ResolvedTask
  variant: CalendarEventVariant
  style?: CSSProperties
  className?: string
  continuesBefore?: boolean
  continuesAfter?: boolean
}

function EventBody({
  resolved,
  variant,
  continuesBefore,
  continuesAfter,
}: Pick<
  CalendarEventProps,
  "resolved" | "variant" | "continuesBefore" | "continuesAfter"
>) {
  const { colorForTask } = useCalendar()
  const color = colorForTask(resolved.task)
  const isMilestone = resolved.kind === "milestone"
  const showTime = resolved.kind === "timed" || isMilestone

  return (
    <>
      {variant === "chip" || variant === "row" ? (
        <span
          aria-hidden
          className={cn(
            "size-2 shrink-0",
            isMilestone ? "rotate-45" : "rounded-full",
          )}
          style={{ background: color }}
        />
      ) : null}
      <span className="min-w-0 flex-1 truncate">
        {continuesBefore ? "← " : null}
        {resolved.task.title}
        {continuesAfter ? " →" : null}
      </span>
      {showTime && variant !== "bar" ? (
        <span className="shrink-0 text-[10px] tabular-nums opacity-70">
          {format(resolved.start, "HH:mm")}
        </span>
      ) : null}
    </>
  )
}

export function CalendarEvent({
  resolved,
  variant,
  style,
  className,
  continuesBefore,
  continuesAfter,
}: CalendarEventProps) {
  const {
    colorForTask,
    editable,
    focusedTaskId,
    permissions,
    renderTooltip,
    requestDelete,
    requestEdit,
    selectTask,
    setClipboard,
    updateTask,
    editing,
  } = useCalendar()

  const task = resolved.task
  const color = colorForTask(task)
  const isFocused = focusedTaskId === task.id
  const canDrag = editable && Boolean(permissions.move)
  const canResize = editable && Boolean(permissions.resize)

  const { attributes, listeners, setNodeRef, transform, isDragging } =
    useDraggable({
      id: task.id,
      disabled: !canDrag,
      data: { taskId: task.id, kind: resolved.kind },
    })

  const resizeState = useRef<{ edge: "start" | "end"; origin: number } | null>(
    null,
  )

  const beginResize = (
    event: ReactPointerEvent<HTMLSpanElement>,
    edge: "start" | "end",
  ) => {
    if (!canResize || !editing) {
      return
    }

    event.preventDefault()
    event.stopPropagation()
    const target = event.currentTarget
    target.setPointerCapture(event.pointerId)
    resizeState.current = {
      edge,
      origin: variant === "bar" ? event.clientX : event.clientY,
    }
  }

  const endResize = (event: ReactPointerEvent<HTMLSpanElement>) => {
    const state = resizeState.current
    resizeState.current = null

    if (!state || !editing) {
      return
    }

    if (variant === "bar") {
      const cellWidth =
        event.currentTarget.closest("[data-week-row]")?.clientWidth ?? 0
      const dayDelta = cellWidth
        ? Math.round(((event.clientX - state.origin) / cellWidth) * 7)
        : 0
      if (dayDelta !== 0) {
        updateTask(editing.resizeTask(task, state.edge, dayDelta, 0))
      }
      return
    }

    const rawMinutes = (event.clientY - state.origin) / PIXELS_PER_MINUTE
    const minuteDelta = Math.round(rawMinutes / SNAP_MINUTES) * SNAP_MINUTES
    if (minuteDelta !== 0) {
      updateTask(editing.resizeTask(task, state.edge, 0, minuteDelta))
    }
  }

  const dragStyle: CSSProperties = transform
    ? {
        transform: `translate3d(${transform.x}px, ${transform.y}px, 0)`,
        zIndex: 40,
      }
    : {}

  const content = (
    <button
      ref={setNodeRef}
      type="button"
      data-task-id={task.id}
      aria-label={task.title}
      onClick={() => selectTask(task)}
      onFocus={() => selectTask(task)}
      className={cn(
        "group/event relative flex w-full items-center gap-1.5 overflow-hidden rounded-sm border px-1.5 text-left text-xs text-foreground transition-shadow",
        variant === "block" && "h-full items-start py-1",
        variant === "bar" && "h-5",
        variant === "chip" && "h-5",
        variant === "row" && "h-8 gap-2 border-transparent bg-transparent px-2",
        isFocused && "ring-2 ring-ring ring-offset-1 ring-offset-background",
        isDragging && "opacity-70",
        canDrag && "cursor-grab active:cursor-grabbing",
        className,
      )}
      style={{
        borderColor: variant === "row" ? undefined : color,
        background:
          variant === "row"
            ? undefined
            : `color-mix(in oklch, ${color} 18%, var(--card))`,
        ...style,
        ...dragStyle,
      }}
      {...(canDrag ? listeners : {})}
      {...(canDrag ? attributes : {})}
    >
      <EventBody
        resolved={resolved}
        variant={variant}
        continuesBefore={continuesBefore}
        continuesAfter={continuesAfter}
      />

      {canResize && variant === "block" ? (
        <>
          <span
            role="presentation"
            onPointerDown={(event) => beginResize(event, "start")}
            onPointerUp={endResize}
            className="absolute inset-x-0 top-0 h-1.5 cursor-ns-resize opacity-0 group-hover/event:opacity-100"
            style={{ background: color }}
          />
          <span
            role="presentation"
            onPointerDown={(event) => beginResize(event, "end")}
            onPointerUp={endResize}
            className="absolute inset-x-0 bottom-0 h-1.5 cursor-ns-resize opacity-0 group-hover/event:opacity-100"
            style={{ background: color }}
          />
        </>
      ) : null}

      {canResize && variant === "bar" ? (
        <span
          role="presentation"
          onPointerDown={(event) => beginResize(event, "end")}
          onPointerUp={endResize}
          className="absolute inset-y-0 right-0 w-1.5 cursor-ew-resize opacity-0 group-hover/event:opacity-100"
          style={{ background: color }}
        />
      ) : null}
    </button>
  )

  const withTooltip = renderTooltip ? (
    <HoverCard openDelay={250}>
      <HoverCardTrigger asChild>{content}</HoverCardTrigger>
      <HoverCardContent className="w-64 text-sm">
        {renderTooltip(task)}
      </HoverCardContent>
    </HoverCard>
  ) : (
    content
  )

  if (!editable) {
    return withTooltip
  }

  return (
    <ContextMenu>
      {/* The trigger wraps rather than composes: two nested `asChild` triggers
          collapse onto one element and the outer one's handlers are lost. */}
      <ContextMenuTrigger asChild>
        <div className="h-full w-full">{withTooltip}</div>
      </ContextMenuTrigger>
      <ContextMenuContent className="w-44">
        <ContextMenuItem
          disabled={!permissions.edit}
          onSelect={() => requestEdit(task)}
        >
          Edit
          <ContextMenuShortcut>↵</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem
          disabled={!permissions.copy}
          onSelect={() => setClipboard({ task, cut: false })}
        >
          Copy
          <ContextMenuShortcut>⌘C</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem
          disabled={!permissions.copy || !permissions.move}
          onSelect={() => setClipboard({ task, cut: true })}
        >
          Cut
          <ContextMenuShortcut>⌘X</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem
          variant="destructive"
          disabled={!permissions.delete}
          onSelect={() => requestDelete(task)}
        >
          Delete
          <ContextMenuShortcut>⌫</ContextMenuShortcut>
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  )
}
