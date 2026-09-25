import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar"
import { Badge } from "@/components/ui/badge"
import {
  SETTINGS_TABS,
  type SettingsTabId,
} from "@/components/settings/settings.types"
import { initialsFor } from "@/lib/initials"
import { cn } from "@/lib/utils"

type SettingsProfileSummary = {
  name: string
  email: string
  planLabel: string
  avatarUrl?: string
}

const MOD_KEY =
  typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.userAgent)
    ? "⌘"
    : "Ctrl"

/* Only shortcuts that exist in the app today. */
const SHORTCUTS = [
  { action: "Show or hide navigation", keys: [MOD_KEY, "B"] },
  { action: "Send a message", keys: ["Enter"] },
  { action: "New line in a message", keys: ["Shift", "Enter"] },
  { action: "Save the open note", keys: [MOD_KEY, "S"] },
]

export function SettingsNav({
  profile,
  activeTab,
  onTabChange,
}: {
  profile: SettingsProfileSummary
  activeTab: SettingsTabId
  onTabChange: (tab: SettingsTabId) => void
}) {
  return (
    <div className="min-w-0 lg:sticky lg:top-0">
      <div className="flex items-center gap-3">
        <Avatar className="size-11 rounded-md">
          <AvatarImage src={profile.avatarUrl} alt="" />
          <AvatarFallback className="rounded-md bg-accent-subtle font-mono text-xs font-medium text-foreground">
            {initialsFor(profile.name)}
          </AvatarFallback>
        </Avatar>
        <div className="min-w-0">
          <p className="truncate text-sm font-medium text-foreground">{profile.name}</p>
          <p className="truncate text-xs text-muted-foreground">{profile.email}</p>
        </div>
      </div>
      <Badge variant="tag" className="mt-3">
        {profile.planLabel}
      </Badge>

      <nav aria-label="Settings sections" className="mt-6">
        <ul className="no-scrollbar -mx-1 flex gap-1 overflow-x-auto px-1 pb-1 lg:mx-0 lg:flex-col lg:gap-0.5 lg:overflow-visible lg:px-0">
          {SETTINGS_TABS.map((tab) => {
            const isActive = tab.id === activeTab

            return (
              <li key={tab.id} className="shrink-0">
                <button
                  type="button"
                  onClick={() => onTabChange(tab.id)}
                  aria-current={isActive ? "page" : undefined}
                  className={cn(
                    "focus-ring relative flex h-9 w-full items-center gap-2.5 rounded-md px-3 text-sm transition-colors duration-150",
                    "text-muted-foreground hover:bg-accent hover:text-foreground",
                    "before:absolute before:inset-y-2 before:left-0 before:hidden before:w-0.5 before:rounded-full before:bg-primary lg:before:block before:opacity-0",
                    isActive &&
                      "bg-accent text-foreground font-medium lg:bg-accent-subtle/70 before:opacity-100",
                  )}
                >
                  <tab.icon className="size-4 shrink-0" aria-hidden="true" />
                  {tab.label}
                </button>
              </li>
            )
          })}
        </ul>
      </nav>

      <section aria-labelledby="settings-shortcuts" className="mt-8 hidden lg:block">
        <h2 id="settings-shortcuts" className="eyebrow text-muted-foreground">
          Keyboard
        </h2>
        <dl className="mt-3 space-y-2.5">
          {SHORTCUTS.map((shortcut) => (
            <div key={shortcut.action} className="flex items-center justify-between gap-3">
              <dt className="text-[0.8125rem] text-muted-foreground">{shortcut.action}</dt>
              <dd className="flex shrink-0 gap-1">
                {shortcut.keys.map((key) => (
                  <kbd
                    key={key}
                    className="inline-flex h-5 min-w-5 items-center justify-center rounded-sm border border-border bg-secondary px-1 font-mono text-2xs text-muted-foreground"
                  >
                    {key}
                  </kbd>
                ))}
              </dd>
            </div>
          ))}
        </dl>
      </section>
    </div>
  )
}
