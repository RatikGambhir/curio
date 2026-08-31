import { useState, type CSSProperties, type FormEvent } from "react"
import { Check, Monitor, Moon, Plus, Sun, X } from "lucide-react"
import type { LucideIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { normalizeHexColor } from "@/features/theme/color"
import {
  APPEARANCE_OPTIONS,
  MAX_CUSTOM_THEMES,
  THEME_CLASS,
  themeSeedStyle,
  type Appearance,
  type ResolvedAppearance,
  type ThemeDefinition,
} from "@/features/theme/theme"
import { useTheme } from "@/hooks/useTheme"
import { cn } from "@/lib/utils"

const APPEARANCE_ICONS: Record<Appearance, LucideIcon> = {
  light: Sun,
  dark: Moon,
  system: Monitor,
}

/* Deliberately none of the built-in hues, so the first thing the picker offers
   is visibly a colour of the user's own rather than a shade of a shipped theme. */
const INITIAL_DRAFT_COLOR = "#7c5cd6"

/* A miniature of the app shell — rail, header, card, buttons — so an option
   shows how its palette lands on real surfaces rather than as loose chips.
   Restating the seed on a nested `.curio-theme` element re-derives the whole
   palette for this subtree, which is what lets an unselected theme be previewed
   without duplicating any colour values in TypeScript. */
function ThemeMiniature({
  theme,
  appearance,
}: {
  theme: ThemeDefinition
  appearance: ResolvedAppearance
}) {
  return (
    <span
      className={cn(
        THEME_CLASS,
        "flex h-24 overflow-hidden rounded-md bg-sidebar",
        appearance === "dark" && "dark",
      )}
      style={themeSeedStyle(theme) as CSSProperties}
    >
      <span className="flex w-9 flex-col gap-1.5 p-2">
        <span className="h-1.5 rounded-full bg-primary" />
        <span className="h-1.5 rounded-full bg-sidebar-accent" />
        <span className="h-1.5 rounded-full bg-sidebar-accent" />
      </span>

      <span className="m-1 ml-0 flex flex-1 flex-col gap-2 rounded-md border border-border bg-card p-2.5 shadow-sm">
        <span className="h-1.5 w-1/2 rounded-full bg-foreground/60" />
        <span className="flex-1 space-y-2 rounded-md border border-border bg-card p-2 shadow-sm">
          <span className="block h-1 w-3/4 rounded-full bg-muted-foreground/40" />
          <span className="flex gap-1.5">
            <span className="h-3.5 w-9 rounded bg-primary" />
            <span className="h-3.5 w-9 rounded bg-secondary" />
          </span>
        </span>
      </span>
    </span>
  )
}

function ThemeOption({
  theme,
  isSelected,
  appearance,
  onSelect,
  onRemove,
}: {
  theme: ThemeDefinition
  isSelected: boolean
  appearance: ResolvedAppearance
  onSelect: () => void
  onRemove?: () => void
}) {
  return (
    <div className="relative">
      <label className="block cursor-pointer">
        <input
          type="radio"
          name="curio-theme"
          value={theme.id}
          checked={isSelected}
          onChange={onSelect}
          className="peer sr-only"
        />
        <span
          className={cn(
            "block rounded-md border-2 border-border bg-card p-3 transition-colors",
            "hover:border-primary/40",
            "peer-checked:border-primary",
            "peer-focus-visible:ring-[3px] peer-focus-visible:ring-ring/50",
          )}
        >
          <ThemeMiniature theme={theme} appearance={appearance} />

          <span className="mt-3 flex items-start justify-between gap-2">
            <span className="block min-w-0">
              <span className="block truncate text-sm font-semibold text-card-foreground">
                {theme.label}
              </span>
              <span className="mt-0.5 block truncate text-xs text-muted-foreground">
                {theme.description}
              </span>
            </span>

            <span
              className={cn(
                "mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full border border-border text-primary-foreground",
                isSelected && "border-primary bg-primary",
              )}
            >
              {isSelected && <Check className="size-3.5" />}
            </span>
          </span>
        </span>
      </label>

      {/* Outside the label: a button nested in one would also toggle the radio. */}
      {onRemove && (
        <button
          type="button"
          onClick={onRemove}
          aria-label={`Remove ${theme.label} theme`}
          className="absolute right-4 top-4 flex size-6 items-center justify-center rounded-full border border-border bg-card/90 text-muted-foreground shadow-xs transition-colors hover:bg-destructive hover:text-destructive-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
        >
          <X className="size-3.5" />
        </button>
      )}
    </div>
  )
}

function AddThemeForm() {
  const { addTheme, canAddTheme } = useTheme()
  const [color, setColor] = useState(INITIAL_DRAFT_COLOR)
  const [hex, setHex] = useState(INITIAL_DRAFT_COLOR)
  const [label, setLabel] = useState("")
  const [error, setError] = useState<string | null>(null)

  const handleHexChange = (value: string) => {
    setHex(value)
    setError(null)

    const normalized = normalizeHexColor(value)
    if (normalized) {
      setColor(normalized)
    }
  }

  const handleColorChange = (value: string) => {
    setColor(value)
    setHex(value)
    setError(null)
  }

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()

    if (!addTheme(hex, label)) {
      setError(
        normalizeHexColor(hex)
          ? `You can save up to ${MAX_CUSTOM_THEMES} colours.`
          : `Enter a colour as a hex value, for example ${INITIAL_DRAFT_COLOR}.`,
      )
      return
    }

    setLabel("")
  }

  return (
    <form onSubmit={handleSubmit} className="mt-4 space-y-3">
      <div className="flex flex-wrap items-end gap-3">
        <div className="space-y-1.5">
          <Label htmlFor="theme-color">Colour</Label>
          <div className="flex items-center gap-2">
            <input
              id="theme-color"
              type="color"
              value={color}
              onChange={(event) => handleColorChange(event.target.value)}
              disabled={!canAddTheme}
              className="size-9 shrink-0 cursor-pointer rounded-md border border-border bg-card p-1 disabled:cursor-not-allowed disabled:opacity-50"
            />
            <Input
              value={hex}
              onChange={(event) => handleHexChange(event.target.value)}
              disabled={!canAddTheme}
              aria-label="Hex colour value"
              spellCheck={false}
              className="w-28 font-mono"
            />
          </div>
        </div>

        <div className="w-44 space-y-1.5">
          <Label htmlFor="theme-label">Name</Label>
          <Input
            id="theme-label"
            value={label}
            onChange={(event) => setLabel(event.target.value)}
            disabled={!canAddTheme}
            placeholder="Optional"
          />
        </div>

        <Button type="submit" disabled={!canAddTheme}>
          <Plus className="size-4" />
          Add colour
        </Button>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}
      {!canAddTheme && (
        <p className="text-sm text-muted-foreground">
          You have saved the maximum of {MAX_CUSTOM_THEMES} colours. Remove one
          to add another.
        </p>
      )}
    </form>
  )
}

export function ThemePicker() {
  const {
    themes,
    theme: activeTheme,
    appearance,
    resolvedAppearance,
    setThemeId,
    setAppearance,
    removeTheme,
  } = useTheme()

  return (
    <div className="space-y-8">
      <fieldset>
        <legend className="text-sm font-semibold text-card-foreground">
          Theme
        </legend>
        <p className="mt-1 text-sm text-muted-foreground">
          Curio builds a full palette from a single colour, so every surface,
          control and chart moves together.
        </p>

        <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          {themes.map((theme) => (
            <ThemeOption
              key={theme.id}
              theme={theme}
              isSelected={theme.id === activeTheme.id}
              appearance={resolvedAppearance}
              onSelect={() => setThemeId(theme.id)}
              onRemove={
                theme.origin === "custom"
                  ? () => removeTheme(theme.id)
                  : undefined
              }
            />
          ))}
        </div>

        <AddThemeForm />
      </fieldset>

      <fieldset>
        <legend className="text-sm font-semibold text-card-foreground">
          Appearance
        </legend>
        <p className="mt-1 text-sm text-muted-foreground">
          Choose a fixed appearance, or follow your operating system.
        </p>

        <div className="mt-4 inline-flex gap-1 rounded-full border border-border bg-muted p-1">
          {APPEARANCE_OPTIONS.map((option) => {
            const Icon = APPEARANCE_ICONS[option.value]
            const isSelected = option.value === appearance

            return (
              <label key={option.value} className="cursor-pointer">
                <input
                  type="radio"
                  name="curio-appearance"
                  value={option.value}
                  checked={isSelected}
                  onChange={() => setAppearance(option.value)}
                  className="peer sr-only"
                />
                <span
                  className={cn(
                    "flex items-center gap-2 rounded-full px-4 py-1.5 text-sm font-semibold text-muted-foreground transition-colors",
                    "hover:text-foreground",
                    "peer-checked:bg-card peer-checked:text-foreground peer-checked:shadow-sm",
                    "peer-focus-visible:ring-[3px] peer-focus-visible:ring-ring/50",
                  )}
                >
                  <Icon className="size-4" />
                  {option.label}
                </span>
              </label>
            )
          })}
        </div>

        {appearance === "system" && (
          <p className="mt-3 text-xs text-muted-foreground">
            Currently following your system, which is set to{" "}
            {resolvedAppearance}.
          </p>
        )}
      </fieldset>
    </div>
  )
}
