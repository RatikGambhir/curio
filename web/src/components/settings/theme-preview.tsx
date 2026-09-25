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

/* The eleven steps of ink the seed generates. Painted with the scale utilities
   rather than the computed hexes so the strip is a reading of the live palette;
   the hex is only the label. */
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
      <div className="flex gap-1 overflow-x-auto pb-1">
        {SCALE_STEPS.map((step) => (
          <div key={step} className="min-w-10 flex-1" title={SCALE_USAGE[step]}>
            <div
              className={cn(
                "h-10 rounded-sm shadow-[inset_0_0_0_1px_oklch(0_0_0/0.06)]",
                SCALE_SWATCHES[step],
              )}
            />
            <p className="mt-1.5 text-center font-mono text-2xs text-foreground">
              {step}
            </p>
            <p className="text-center font-mono text-[0.625rem] uppercase text-muted-foreground">
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
    <section aria-labelledby="theme-preview-title" className="space-y-8">
      <div>
        <h3 id="theme-preview-title" className="eyebrow text-muted-foreground">
          Preview
        </h3>
        <p className="mt-2 text-sm text-muted-foreground">
          How the selected ink renders Curio&rsquo;s controls in every state.
        </p>
        <div className="mt-4 rounded-lg border border-border bg-background p-5">
          <ButtonMatrix />
        </div>
      </div>

      <div>
        <h3 className="eyebrow text-muted-foreground">Ink scale</h3>
        <p className="mt-2 mb-4 text-sm text-muted-foreground">
          Actions come from the middle of the scale, highlights from the pale
          end. Hover a step to see what it drives.
        </p>
        <ScaleStrip />
      </div>
    </section>
  )
}
