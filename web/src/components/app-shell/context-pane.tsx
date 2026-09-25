import type { ReactNode } from "react"
import { PanelLeftClose, PanelLeftOpen, type LucideIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetTitle,
} from "@/components/ui/sheet"
import type { ContextPaneState } from "@/hooks/useContextPane"
import { cn } from "@/lib/utils"

/* A page-owned secondary column — chat history, note folders — that lives
   inside the content sheet next to the spine rather than replacing it, so
   global navigation never disappears when a page has a list of its own. */
export function ContextPane({
  pane,
  title,
  description,
  action,
  children,
}: {
  pane: ContextPaneState
  title: string
  /** Read to assistive technology when the pane opens as a sheet. */
  description: string
  action?: ReactNode
  children: ReactNode
}) {
  const header = (
    <div className="flex h-(--app-header-height) shrink-0 items-center gap-2 border-b border-border pl-4 pr-3">
      <span className="eyebrow min-w-0 flex-1 truncate text-muted-foreground">
        {title}
      </span>
      {action}
    </div>
  )

  if (!pane.isInline) {
    return (
      <Sheet open={pane.open} onOpenChange={pane.setOpen}>
        <SheetContent
          side="left"
          className="w-[min(20rem,88vw)] gap-0 p-0 [&>[data-slot=sheet-close]]:hidden"
        >
          <SheetTitle className="sr-only">{title}</SheetTitle>
          <SheetDescription className="sr-only">{description}</SheetDescription>
          {header}
          <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>
        </SheetContent>
      </Sheet>
    )
  }

  if (!pane.open) {
    return null
  }

  return (
    <aside
      aria-label={title}
      className="flex w-[17rem] shrink-0 flex-col border-r border-border bg-background xl:w-[18.5rem]"
    >
      {header}
      <div className="min-h-0 flex-1 overflow-y-auto">{children}</div>
    </aside>
  )
}

export function ContextPaneToggle({
  pane,
  label,
  overlayIcon,
  className,
}: {
  pane: ContextPaneState
  /** What the pane holds, e.g. "conversations". */
  label: string
  /** Shown when the pane opens as a sheet, where a panel glyph would read as
      a second copy of the navigation toggle beside it. */
  overlayIcon: LucideIcon
  className?: string
}) {
  const Icon = !pane.isInline
    ? overlayIcon
    : pane.open
      ? PanelLeftClose
      : PanelLeftOpen
  const action = pane.open ? `Hide ${label}` : `Show ${label}`

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-sm"
      onClick={pane.toggle}
      aria-expanded={pane.open}
      aria-label={action}
      title={action}
      className={cn("text-muted-foreground", className)}
    >
      <Icon className="size-4" aria-hidden="true" />
    </Button>
  )
}

/* One row in a pane list. Selection is a highlighter tint plus the theme's ink
   bar at the leading edge, matching the spine's "you are here". */
export function ContextPaneItem({
  isActive,
  onSelect,
  children,
  className,
}: {
  isActive: boolean
  onSelect: () => void
  children: ReactNode
  className?: string
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      aria-current={isActive ? "true" : undefined}
      className={cn(
        "focus-ring relative flex w-full min-w-0 flex-col gap-0.5 rounded-md px-3 py-2 text-left transition-colors duration-150",
        "hover:bg-accent",
        "before:absolute before:inset-y-2 before:left-0 before:w-0.5 before:rounded-full before:bg-primary before:opacity-0 before:transition-opacity",
        isActive && "bg-accent-subtle/70 before:opacity-100 hover:bg-accent-subtle",
        className,
      )}
    >
      {children}
    </button>
  )
}
