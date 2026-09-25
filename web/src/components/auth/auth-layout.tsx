import type { ReactNode } from "react";

import { CurioWordmark } from "@/components/brand/curio-mark";

/* The frame for sign-in and onboarding: an ink page on the left carrying the
   brand and one line of voice, paper on the right for the task. Below `lg` the
   ink page folds away and only the wordmark stays. */
export function AuthLayout({
  statement,
  aside,
  children,
}: {
  /** A single serif line for the ink page. */
  statement: ReactNode;
  /** Optional content under the statement, e.g. onboarding progress. */
  aside?: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="grid min-h-svh bg-card lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
      <aside className="relative hidden flex-col justify-between overflow-hidden bg-sidebar px-12 py-10 text-sidebar-foreground lg:flex xl:px-16">
        {/* The notebook's margin rule, in the theme's ink. */}
        <span
          aria-hidden="true"
          className="pointer-events-none absolute inset-y-0 left-7 w-px bg-sidebar-primary/45 xl:left-9"
        />
        <CurioWordmark className="relative text-lg" />
        <div className="relative max-w-md">
          <p className="font-display text-display-lg text-sidebar-foreground">
            {statement}
          </p>
          {aside ? <div className="mt-10">{aside}</div> : null}
        </div>
        <p className="eyebrow relative text-sidebar-muted-foreground">
          A notebook for everything you&rsquo;re curious about
        </p>
      </aside>

      <main className="flex min-h-svh flex-col px-5 py-8 sm:px-10">
        <CurioWordmark className="text-lg text-foreground lg:hidden" />
        <div className="flex flex-1 items-center justify-center py-10">
          <div className="rise-in w-full max-w-md">{children}</div>
        </div>
      </main>
    </div>
  );
}
