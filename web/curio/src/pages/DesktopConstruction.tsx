import { Hammer } from "lucide-react";
import { Link } from "react-router-dom";

function DesktopConstruction() {
  return (
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden bg-background px-6 py-12">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top,_hsl(var(--primary)/0.14),transparent_52%),radial-gradient(circle_at_bottom_right,_hsl(var(--accent-foreground)/0.09),transparent_40%)]" />

      <section className="relative z-10 w-full max-w-2xl rounded-3xl border border-border/70 bg-card/85 p-8 text-center shadow-2xl backdrop-blur-sm sm:p-12">
        <div className="mx-auto mb-5 flex h-14 w-14 items-center justify-center rounded-full border border-border bg-background">
          <Hammer className="h-6 w-6 text-primary" />
        </div>
        <p className="mb-2 text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
          Curio Desktop
        </p>
        <h1 className="mb-4 text-4xl font-semibold tracking-tight">
          We&apos;re working on it
        </h1>
        <p className="mx-auto max-w-xl text-sm text-muted-foreground sm:text-base">
          The desktop experience is under active construction. The web platform
          is live today and ready to use.
        </p>
        <div className="mt-8 flex justify-center">
          <Link
            to="/"
            className="inline-flex h-11 items-center justify-center rounded-xl border border-border px-6 text-sm font-medium transition-transform hover:-translate-y-0.5"
          >
            Back to landing
          </Link>
        </div>
      </section>
    </div>
  );
}

export default DesktopConstruction;
