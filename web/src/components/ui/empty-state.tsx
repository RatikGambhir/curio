import type { ReactNode } from "react"
import type { LucideIcon } from "lucide-react"

import { cn } from "@/lib/utils"

/* The one shape for "nothing here yet": a quiet glyph, a serif line saying what
   is missing, a sentence on why, and at most one way forward. */
export function EmptyState({
  icon: Icon,
  title,
  children,
  action,
  headingLevel = 2,
  className,
}: {
  icon?: LucideIcon
  title: ReactNode
  children?: ReactNode
  action?: ReactNode
  headingLevel?: 2 | 3
  className?: string
}) {
  const Heading = headingLevel === 2 ? "h2" : "h3"

  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center px-6 py-12 text-center",
        className,
      )}
    >
      {Icon ? (
        <Icon
          className="mb-4 size-6 text-muted-foreground"
          strokeWidth={1.5}
          aria-hidden="true"
        />
      ) : null}
      <Heading className="font-display text-display-sm text-foreground">
        {title}
      </Heading>
      {children ? (
        <div className="mt-2 max-w-sm text-sm leading-relaxed text-muted-foreground">
          {children}
        </div>
      ) : null}
      {action ? <div className="mt-5">{action}</div> : null}
    </div>
  )
}
