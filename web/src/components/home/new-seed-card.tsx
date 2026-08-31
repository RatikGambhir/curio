import { Plus } from "lucide-react";
import { Link } from "react-router-dom";

export function NewSeedCard() {
  return (
    <Link
      to="/notes"
      className="group flex min-h-[11.75rem] min-w-0 flex-col items-center justify-center gap-3 rounded-[0.75rem] border border-dashed border-border-strong bg-card/30 text-center text-muted-foreground outline-none transition-[border-color,background-color,color,transform] hover:-translate-y-0.5 hover:border-primary/60 hover:bg-accent/30 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-card"
    >
      <span className="flex size-10 items-center justify-center rounded-full bg-accent-subtle text-primary transition-transform group-hover:scale-105">
        <Plus className="size-5" aria-hidden="true" />
      </span>
      <span className="text-sm font-semibold">Plant new seed</span>
    </Link>
  );
}
