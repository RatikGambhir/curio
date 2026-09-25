import { SettingsSection } from "@/components/settings/settings-section"
import { ThemePicker } from "@/components/settings/theme-picker"
import { ThemePreview } from "@/components/settings/theme-preview"

export function CustomizationTab() {
  return (
    <SettingsSection
      title="Appearance"
      description="Choose the ink Curio writes with and whether it works in daylight or at night. Changes apply straight away and are remembered on this device."
    >
      <div className="space-y-12">
        <ThemePicker />
        <ThemePreview />
      </div>
    </SettingsSection>
  )
}
