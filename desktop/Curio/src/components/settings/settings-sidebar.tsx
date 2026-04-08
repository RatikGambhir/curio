import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";
import { Info } from "lucide-react";

export type SettingsProfileSummary = {
  name: string;
  email: string;
  planLabel: string;
  avatarUrl?: string;
};

type SettingsSidebarProps = {
  profile: SettingsProfileSummary;
  usagePercent?: number;
  className?: string;
};

const shortcuts = [
  { action: "Search", keys: "Ctrl+K" },
  { action: "New Chat", keys: "Ctrl+N" },
  { action: "Toggle Sidebar", keys: "Ctrl+B" },
  { action: "Open Model Picker", keys: "Ctrl+M" },
  { action: "Delete Current Chat", keys: "Ctrl+Backspace" },
];

export function SettingsSidebar({
  profile,
  usagePercent = 42,
  className,
}: SettingsSidebarProps) {
  const initials = profile.name
    .split(" ")
    .filter(Boolean)
    .map((part) => part[0]?.toUpperCase())
    .slice(0, 2)
    .join("");

  return (
    <aside className={cn("flex flex-col gap-4", className)}>
      <section className="rounded-[1.75rem] border border-border bg-card p-5 shadow-sm">
        <div className="flex items-center gap-3">
          <Avatar className="size-14 border border-border">
            <AvatarImage src={profile.avatarUrl} alt={profile.name} />
            <AvatarFallback className="bg-accent text-sm font-semibold text-accent-foreground">
              {initials || "CU"}
            </AvatarFallback>
          </Avatar>
          <div className="min-w-0 space-y-1">
            <p className="truncate text-base font-bold text-card-foreground">{profile.name}</p>
            <p className="truncate text-sm text-muted-foreground">{profile.email}</p>
          </div>
        </div>
        <div className="mt-4 inline-flex rounded-full bg-accent px-3 py-1 text-xs font-semibold text-accent-foreground">
          {profile.planLabel}
        </div>
      </section>

      <section className="rounded-[1.75rem] border border-border bg-card p-5 shadow-sm">
        <div className="flex items-center justify-between gap-2">
          <h3 className="text-sm font-semibold text-card-foreground">Usage limits</h3>
          <Info className="size-4 text-muted-foreground" />
        </div>
        <p className="mt-1 text-sm text-muted-foreground">Plan: Base</p>
        <div className="mt-4 h-2 rounded-full bg-secondary">
          <div
            className="h-2 rounded-full bg-primary"
            style={{ width: `${Math.min(Math.max(usagePercent, 0), 100)}%` }}
          />
        </div>
        <p className="mt-2 text-xs text-muted-foreground">{usagePercent}% used</p>
      </section>

      <section className="rounded-[1.75rem] border border-border bg-card p-5 shadow-sm">
        <h3 className="text-sm font-semibold text-card-foreground">Keyboard shortcuts</h3>
        <ul className="mt-4 space-y-2">
          {shortcuts.map((shortcut) => (
            <li
              key={shortcut.action}
              className="flex items-center justify-between gap-3 text-sm"
            >
              <span className="text-muted-foreground">{shortcut.action}</span>
              <kbd className="rounded-md border border-border bg-secondary px-2 py-1 font-mono text-xs text-foreground">
                {shortcut.keys}
              </kbd>
            </li>
          ))}
        </ul>
      </section>
    </aside>
  );
}
