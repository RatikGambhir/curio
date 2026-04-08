import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import { SETTINGS_TABS, type SettingsTabId } from "@/components/settings/settings.types"

type SettingsTabsProps = {
  activeTab: SettingsTabId
  onTabChange: (tab: SettingsTabId) => void
  className?: string
}

export function SettingsTabs({
  activeTab,
  onTabChange,
  className,
}: SettingsTabsProps) {
  return (
    <div
      className={cn(
        "rounded-2xl border border-border bg-card p-2 shadow-sm",
        className,
      )}
    >
      <div className="overflow-x-auto">
        <div className="flex min-w-max flex-wrap gap-2">
          {SETTINGS_TABS.map((tab) => {
            const isActive = tab.id === activeTab

            return (
              <Button
                key={tab.id}
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => onTabChange(tab.id)}
                className={cn(
                  "rounded-full px-4 text-sm font-semibold text-muted-foreground hover:bg-accent/70 hover:text-foreground",
                  isActive && "bg-secondary text-foreground shadow-xs",
                )}
              >
                <tab.icon className="size-4" />
                {tab.label}
              </Button>
            )
          })}
        </div>
      </div>
    </div>
  )
}
