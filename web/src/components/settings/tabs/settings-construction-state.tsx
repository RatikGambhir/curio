import { Hammer } from "lucide-react"

import { SettingsSection } from "@/components/settings/settings-section"
import { EmptyState } from "@/components/ui/empty-state"

type SettingsConstructionStateProps = {
  title: string
}

export function SettingsConstructionState({ title }: SettingsConstructionStateProps) {
  return (
    <SettingsSection title={title}>
      <EmptyState
        icon={Hammer}
        title="Not built yet"
        headingLevel={3}
        className="rounded-lg border border-dashed border-border-strong py-16"
      >
        This part of Curio is still being made. Nothing here changes your
        account yet.
      </EmptyState>
    </SettingsSection>
  )
}
