import { CalendarDays, NotebookPen, Sparkles } from "lucide-react"
import type { LucideIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  SCALE_STEPS,
  SCALE_USAGE,
  scaleHexes,
} from "@/features/theme/palette"
import { themeSeed } from "@/features/theme/theme"
import { useTheme } from "@/hooks/useTheme"
import { cn } from "@/lib/utils"

const BUTTON_STATES = ["Default", "Hover", "Active", "Disabled"] as const

type ButtonRow = {
  label: string
  variant: "default" | "secondary" | "outline"
  hoverClassName: string
  activeClassName: string
}

/* Hover and active are painted with the tokens the Button variants use for the
   real `hover:`/`active:` states, so the row reads as a static swatch of each
   step rather than needing the reader to hover four separate controls. */
const BUTTON_ROWS: ButtonRow[] = [
  {
    label: "Primary",
    variant: "default",
    hoverClassName: "bg-primary-hover",
    activeClassName: "bg-primary-active",
  },
  {
    label: "Secondary",
    variant: "secondary",
    hoverClassName: "bg-secondary-hover",
    activeClassName: "bg-secondary-active",
  },
  {
    label: "Tertiary",
    variant: "outline",
    hoverClassName: "bg-accent text-accent-foreground",
    activeClassName: "bg-secondary-active text-secondary-foreground",
  },
]

type SampleCard = {
  icon: LucideIcon
  title: string
  body: string
  className: string
}

const SAMPLE_CARDS: SampleCard[] = [
  {
    icon: NotebookPen,
    title: "Capture a note",
    body: "Everything you write stays searchable in one place.",
    className: "bg-secondary text-secondary-foreground",
  },
  {
    icon: CalendarDays,
    title: "Plan your week",
    body: "Events, tasks and reminders share a single calendar.",
    className: "bg-secondary-hover text-secondary-foreground",
  },
  {
    icon: Sparkles,
    title: "Ask your vault",
    body: "Answers pulled straight from what you already saved.",
    className: "bg-primary text-primary-foreground",
  },
]

/* The eleven steps every other colour in the app is mapped from. Painted with
   the scale utilities rather than the computed hexes so the strip is a reading
   of the live palette; the hex is only the label. */
const SCALE_SWATCHES: Record<(typeof SCALE_STEPS)[number], string> = {
  50: "bg-scale-50",
  100: "bg-scale-100",
  200: "bg-scale-200",
  300: "bg-scale-300",
  400: "bg-scale-400",
  500: "bg-scale-500",
  600: "bg-scale-600",
  700: "bg-scale-700",
  800: "bg-scale-800",
  900: "bg-scale-900",
  950: "bg-scale-950",
}

function ScaleStrip() {
  const { theme } = useTheme()
  const hexes = scaleHexes(themeSeed(theme))

  return (
    <div>
      <div className="flex gap-1 overflow-hidden rounded-lg">
        {SCALE_STEPS.map((step) => (
          <div key={step} className="flex-1" title={SCALE_USAGE[step]}>
            <div
              className={cn(
                "h-10 rounded-md border border-border",
                SCALE_SWATCHES[step],
              )}
            />
            <p className="mt-1.5 text-center text-[0.6875rem] font-semibold text-muted-foreground">
              {step}
            </p>
            <p className="text-center font-mono text-[0.625rem] text-muted-foreground/70 uppercase">
              {hexes[step].slice(1)}
            </p>
          </div>
        ))}
      </div>
    </div>
  )
}

function ButtonMatrix() {
  return (
    <div className="overflow-x-auto">
      <div className="grid min-w-[26rem] grid-cols-4 gap-x-3 gap-y-2.5">
        {BUTTON_STATES.map((state) => (
          <p
            key={state}
            className="text-center text-xs font-semibold text-muted-foreground"
          >
            {state}
          </p>
        ))}

        {BUTTON_ROWS.map((row) =>
          BUTTON_STATES.map((state) => (
            <Button
              key={`${row.label}-${state}`}
              type="button"
              variant={row.variant}
              size="sm"
              tabIndex={-1}
              disabled={state === "Disabled"}
              className={cn(
                "pointer-events-none w-full",
                state === "Hover" && row.hoverClassName,
                state === "Active" && row.activeClassName,
              )}
            >
              {row.label}
            </Button>
          )),
        )}
      </div>
    </div>
  )
}

export function ThemePreview() {
  return (
    <div className="space-y-6 border border-border bg-background p-5">
      <div>
        <h3 className="text-sm font-semibold text-foreground">Preview</h3>
        <p className="mt-1 text-sm text-muted-foreground">
          How the selected theme renders Curio&rsquo;s controls and surfaces.
        </p>
      </div>

      <ButtonMatrix />

      <div className="grid gap-3 sm:grid-cols-3">
        {SAMPLE_CARDS.map((card) => (
          <div
            key={card.title}
            className={cn("rounded-xl p-4 shadow-sm", card.className)}
          >
            <span className="flex size-8 items-center justify-center rounded-lg bg-card/25">
              <card.icon className="size-4" />
            </span>
            <p className="mt-3 text-sm font-semibold">{card.title}</p>
            <p className="mt-1 text-xs opacity-80">{card.body}</p>
          </div>
        ))}
      </div>

      <div>
        <h4 className="text-sm font-semibold text-foreground">Scale</h4>
        <p className="mt-1 mb-3 text-sm text-muted-foreground">
          Surfaces come from the pale end, actions from the middle, type from the
          dark end. Hover a step to see what it drives.
        </p>
        <ScaleStrip />
      </div>
    </div>
  )
}
