import { Hammer } from "lucide-react"

type SettingsConstructionStateProps = {
  title: string
}

export function SettingsConstructionState({ title }: SettingsConstructionStateProps) {
  return (
    <section className="flex min-h-[26rem] items-center justify-center rounded-[2rem] border border-border bg-card p-8 shadow-md">
      <div className="flex max-w-sm flex-col items-center text-center">
        <div className="flex size-14 items-center justify-center rounded-2xl bg-accent text-accent-foreground">
          <Hammer className="size-6" />
        </div>
        <h2 className="mt-4 text-2xl font-bold text-card-foreground">{title}</h2>
        <p className="mt-2 text-sm text-muted-foreground">We're working on this section.</p>
      </div>
    </section>
  )
}
