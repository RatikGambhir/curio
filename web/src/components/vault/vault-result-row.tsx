import { HighlightMatch } from "@/components/vault/highlight-match";
import type { QAPair } from "@/components/vault/vault.types";
import { Badge } from "@/components/ui/badge";

const ENTRY_DATE = new Intl.DateTimeFormat(undefined, {
  day: "numeric",
  month: "short",
  year: "numeric",
});

interface VaultResultRowProps {
  item: QAPair;
  query: string;
}

/* A question and its kept answer, read left to right like a two-column index
   entry on wide screens and stacked on narrow ones. */
export function VaultResultRow({ item, query }: VaultResultRowProps) {
  return (
    <article className="grid gap-x-10 gap-y-3 border-b border-border py-6 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
      <header className="min-w-0">
        <h2 className="font-display text-[1.3125rem] leading-snug text-foreground">
          <HighlightMatch text={item.question} query={query} />
        </h2>
        <p className="mt-2.5 flex items-center gap-2 text-xs text-muted-foreground">
          <Badge variant="tag">{item.category.toLowerCase()}</Badge>
          <time dateTime={item.date} className="font-mono tabular-nums">
            {ENTRY_DATE.format(new Date(`${item.date}T00:00:00`))}
          </time>
        </p>
      </header>
      <p className="min-w-0 text-[0.9375rem] leading-relaxed text-secondary-foreground">
        <HighlightMatch text={item.answer} query={query} />
      </p>
    </article>
  );
}
