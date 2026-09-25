import type { LucideIcon } from "lucide-react"

import { cn } from "@/lib/utils"

export type SegmentedOption<TValue extends string> = {
  value: TValue
  label: string
  icon?: LucideIcon
}

/* A small set of mutually exclusive views — month/week, list/board,
   light/dark. Built on native radio inputs so arrow keys, form semantics and
   screen-reader announcements come for free; the selected segment is a raised
   sheet on a sunken track. */
export function SegmentedControl<TValue extends string>({
  label,
  name,
  options,
  value,
  onValueChange,
  hideLabelsBelow,
  size = "sm",
  className,
}: {
  /** Accessible name for the group. */
  label: string
  /** Unique radio-group name; defaults to the label. */
  name?: string
  options: readonly SegmentedOption<TValue>[]
  value: TValue
  onValueChange: (value: TValue) => void
  /** Collapse segments to icons below this breakpoint. */
  hideLabelsBelow?: "sm" | "md"
  size?: "sm" | "default"
  className?: string
}) {
  const groupName = name ?? `segmented-${label.toLowerCase().replace(/\W+/g, "-")}`

  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={cn(
        "inline-flex shrink-0 items-center gap-0.5 rounded-md bg-secondary p-0.5 shadow-[inset_0_0_0_1px_var(--border-subtle)]",
        className,
      )}
    >
      {options.map((option) => {
        const Icon = option.icon
        const selected = option.value === value

        return (
          <label
            key={option.value}
            title={option.label}
            className="relative cursor-pointer"
          >
            <input
              type="radio"
              name={groupName}
              value={option.value}
              checked={selected}
              onChange={() => onValueChange(option.value)}
              className="peer sr-only"
            />
            <span
              className={cn(
                "flex items-center gap-1.5 rounded-[calc(var(--radius)-2px)] font-medium text-muted-foreground transition-[background-color,color,box-shadow] duration-150",
                size === "sm" ? "h-7 px-2.5 text-[0.8125rem]" : "h-8 px-3 text-sm",
                "hover:text-foreground",
                "peer-checked:bg-card peer-checked:text-foreground peer-checked:shadow-sm",
                "peer-focus-visible:outline-2 peer-focus-visible:outline-offset-1 peer-focus-visible:outline-ring",
              )}
            >
              {Icon ? <Icon className="size-3.5 shrink-0" aria-hidden="true" /> : null}
              <span
                className={cn(
                  Icon && hideLabelsBelow === "sm" && "sr-only sm:not-sr-only",
                  Icon && hideLabelsBelow === "md" && "sr-only md:not-sr-only",
                )}
              >
                {option.label}
              </span>
            </span>
          </label>
        )
      })}
    </div>
  )
}
