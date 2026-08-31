import { ThemePicker } from "@/components/settings/theme-picker"
import { ThemePreview } from "@/components/settings/theme-preview"

export function CustomizationTab() {
  return (
    <section className="p-6 sm:p-8">
      <h2 className="text-2xl font-bold text-card-foreground">Customization</h2>
      <p className="mt-2 text-sm text-muted-foreground">
        Choose how Curio looks. Changes apply straight away and are remembered on
        this device.
      </p>

      <div className="mt-8 space-y-8">
        <ThemePicker />
        <ThemePreview />
      </div>
    </section>
  )
}
