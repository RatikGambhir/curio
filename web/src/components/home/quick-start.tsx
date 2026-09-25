import { ArrowRight, CalendarPlus, MessageSquarePlus, NotebookPen } from "lucide-react";
import { Link } from "react-router-dom";

const ACTIONS = [
  {
    to: "/chat",
    icon: MessageSquarePlus,
    label: "Ask Curio",
    hint: "Start a conversation",
  },
  {
    to: "/notes",
    icon: NotebookPen,
    label: "Write a note",
    hint: "Open your notebook",
  },
  {
    to: "/calendar",
    icon: CalendarPlus,
    label: "Plan something",
    hint: "Add to your calendar",
  },
] as const;

/* The three things people come to Curio to do, one step away. */
export function QuickStart() {
  return (
    <nav aria-labelledby="quick-start-title" className="min-w-0">
      <h2 id="quick-start-title" className="eyebrow text-muted-foreground">
        Start something
      </h2>
      <ul className="mt-3 border-t border-border">
        {ACTIONS.map(({ to, icon: Icon, label, hint }) => (
          <li key={to} className="border-b border-border">
            <Link
              to={to}
              className="focus-ring group flex items-center gap-3 rounded-sm py-3"
            >
              <span className="flex size-8 shrink-0 items-center justify-center rounded-md bg-secondary text-foreground transition-colors duration-150 group-hover:bg-primary group-hover:text-primary-foreground">
                <Icon className="size-4" aria-hidden="true" />
              </span>
              <span className="min-w-0 flex-1">
                <span className="block text-sm font-medium text-foreground">
                  {label}
                </span>
                <span className="block text-xs text-muted-foreground">{hint}</span>
              </span>
              <ArrowRight
                className="size-4 text-muted-foreground opacity-0 transition-[opacity,transform] duration-150 group-hover:translate-x-0.5 group-hover:opacity-100 group-focus-visible:opacity-100"
                aria-hidden="true"
              />
            </Link>
          </li>
        ))}
      </ul>
    </nav>
  );
}
