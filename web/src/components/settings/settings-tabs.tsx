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
    <div className={cn("border-b border-border px-2 py-2", className)}>
      <div className="overflow-x-auto">
        <div className="flex min-w-max flex-wrap gap-1">
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
                  "rounded-md px-4 text-sm font-semibold text-muted-foreground hover:bg-accent/70 hover:text-foreground",
                  isActive && "bg-secondary text-foreground",
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
