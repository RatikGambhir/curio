import { Link } from "react-router-dom";

import type { Cultivation } from "@/components/home/home.types";
import { cn } from "@/lib/utils";

type CultivationCardProps = {
  cultivation: Cultivation;
};

export function CultivationCard({ cultivation }: CultivationCardProps) {
  const Icon = cultivation.icon;

  return (
    <Link
      to="/vault"
      className="group relative flex min-h-[11.75rem] min-w-0 flex-col overflow-hidden rounded-[0.75rem] border border-border bg-card p-5 shadow-sm outline-none transition-[border-color,box-shadow,transform] hover:-translate-y-0.5 hover:border-border-strong hover:shadow-md focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-card"
    >
      {cultivation.imageSrc ? (
        <>
          <img
            src={cultivation.imageSrc}
            alt=""
            loading="lazy"
            decoding="async"
            className="absolute inset-0 size-full object-cover opacity-20 transition-opacity group-hover:opacity-30 dark:opacity-30 dark:group-hover:opacity-40"
          />
          <span className="absolute inset-0 bg-card/70" aria-hidden="true" />
        </>
      ) : null}

      <span
        className={cn(
          "relative flex size-9 items-center justify-center rounded-full",
          cultivation.iconClassName,
        )}
      >
        <Icon className="size-4" aria-hidden="true" />
      </span>

      <span className="relative mt-auto block min-w-0 pt-6">
        <span className="block text-base font-semibold leading-snug text-foreground">
          {cultivation.title}
        </span>
        <span className="mt-1 block text-xs text-muted-foreground">
          {cultivation.updated}
        </span>
        <span className="mt-3 flex flex-wrap gap-1.5">
          {cultivation.tags.map((tag) => (
            <span
              key={tag}
              className="rounded-full bg-secondary px-2 py-0.5 text-[0.6875rem] font-medium leading-4 text-secondary-foreground"
            >
              {tag}
            </span>
          ))}
        </span>
      </span>
    </Link>
  );
}
