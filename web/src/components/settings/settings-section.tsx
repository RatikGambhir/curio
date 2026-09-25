import type { ReactNode } from "react"

import { cn } from "@/lib/utils"

/* The frame every settings section shares: a serif title, one sentence on
   what the section controls, then its content. */
export function SettingsSection({
  title,
  description,
  aside,
  children,
  className,
}: {
  title: string
  description?: ReactNode
  /** A control that applies to the whole section, aligned with the title. */
  aside?: ReactNode
  children: ReactNode
  className?: string
}) {
  return (
    <section className={cn("rise-in min-w-0", className)}>
      <header className="flex flex-wrap items-end justify-between gap-4 border-b border-border pb-4">
        <div className="min-w-0">
          <h2 className="font-display text-display-sm text-foreground">{title}</h2>
          {description ? (
            <p className="mt-2 max-w-xl text-sm leading-relaxed text-muted-foreground">
              {description}
            </p>
          ) : null}
        </div>
        {aside}
      </header>
      <div className="pt-6">{children}</div>
    </section>
  )
}
