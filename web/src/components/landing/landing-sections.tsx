import { useState, type FormEvent } from "react";
import {
  ArrowRight,
  BookOpen,
  CalendarDays,
  ChevronDown,
  Globe,
  MessageSquare,
  NotebookPen,
} from "lucide-react";
import { Link } from "react-router-dom";

import { CurioMark, CurioWordmark } from "@/components/brand/curio-mark";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Notice } from "@/components/ui/notice";
import { Textarea } from "@/components/ui/textarea";

type LandingHeaderProps = {
  onContactClick: () => void;
  onWebClick: () => void;
  onDesktopClick: () => void;
};

export function LandingHeader({
  onContactClick,
  onWebClick,
  onDesktopClick,
}: LandingHeaderProps) {
  return (
    <header className="sticky top-0 z-30 border-b border-border/70 bg-background/90 backdrop-blur-md">
      <div className="mx-auto flex h-16 w-full max-w-[76rem] items-center justify-between gap-4 px-5 sm:px-8">
        <Link to="/" aria-label="Curio home" className="focus-ring rounded-sm">
          <CurioWordmark className="text-base text-foreground" />
        </Link>

        <nav aria-label="Primary" className="flex items-center gap-1 sm:gap-2">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm" className="hidden sm:inline-flex">
                Platforms
                <ChevronDown className="size-3.5" aria-hidden="true" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-44">
              <DropdownMenuItem onSelect={onWebClick}>Web</DropdownMenuItem>
              <DropdownMenuItem onSelect={onDesktopClick}>Desktop</DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
          <Button variant="ghost" size="sm" onClick={onContactClick}>
            Contact
          </Button>
          <Button asChild size="sm" className="ml-1">
            <Link to="/login">Sign in</Link>
          </Button>
        </nav>
      </div>
    </header>
  );
}

/* A drawing of Curio at work, built from the same tokens and type as the app
   rather than a screenshot, so it stays true to whichever theme is active. */
function ProductSketch() {
  return (
    <figure
      role="img"
      aria-label="A Curio conversation, with the answer filed into the vault"
      className="relative"
    >
      <div className="overflow-hidden rounded-xl border border-border-strong bg-sidebar shadow-xl">
        <div className="flex">
          <div
            aria-hidden="true"
            className="hidden w-14 shrink-0 flex-col items-center gap-3 py-4 text-sidebar-muted-foreground sm:flex"
          >
            <CurioMark className="size-5 text-sidebar-foreground" />
            <span className="mt-3 h-7 w-7 rounded-md bg-sidebar-accent" />
            <span className="h-7 w-7 rounded-md" />
            <span className="h-7 w-7 rounded-md" />
            <span className="h-7 w-7 rounded-md" />
          </div>
          <div className="min-w-0 flex-1 rounded-l-lg bg-card sm:my-1.5">
            <div className="flex h-11 items-center gap-3 border-b border-border px-5">
              <span className="font-display text-[1.0625rem] text-foreground">
                Why do leaves turn red?
              </span>
            </div>
            <div className="space-y-5 px-5 py-6">
              <p className="ml-auto w-fit max-w-[80%] rounded-xl rounded-br-sm bg-secondary px-3.5 py-2 text-[0.8125rem] text-foreground">
                Why do some leaves turn red instead of yellow?
              </p>
              <div className="font-serif text-[0.9375rem] leading-relaxed text-foreground">
                <p>
                  Yellow was there all summer, hidden under chlorophyll. Red is
                  new: as nights cool, some trees{" "}
                  <mark className="rounded-[2px] bg-accent-subtle px-0.5 text-inherit">
                    make anthocyanins
                  </mark>{" "}
                  to shield leaves while they reclaim nutrients.
                </p>
              </div>
              <div className="flex flex-wrap items-center gap-2 border-t border-border pt-4 text-xs text-muted-foreground">
                <BookOpen className="size-3.5 text-primary" aria-hidden="true" />
                <span>Kept in Vault</span>
                <span className="rounded-sm border border-border px-1.5 font-mono text-2xs">
                  botany
                </span>
                <span className="rounded-sm border border-border px-1.5 font-mono text-2xs">
                  autumn
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div
        aria-hidden="true"
        className="absolute -bottom-24 -left-8 hidden w-56 rotate-[-2deg] rounded-lg border border-border bg-card p-4 shadow-lg md:block"
      >
        <p className="eyebrow text-muted-foreground">Seedling thought</p>
        <p
          className="ruled-paper mt-2 pt-2 font-display text-[0.9375rem] italic leading-7 text-foreground [--rule-gap:1.75rem]"
        >
          Do evergreens do this too? Look into it.
        </p>
      </div>
    </figure>
  );
}

export function HeroSection({ onHowItWorks }: { onHowItWorks: () => void }) {
  return (
    <section className="mx-auto grid w-full max-w-[76rem] items-center gap-x-16 gap-y-16 px-5 pb-24 pt-14 sm:px-8 md:pt-20 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.05fr)] lg:pb-40 lg:pt-24">
      <div className="max-w-xl">
        <p className="eyebrow rise-in text-muted-foreground">
          A knowledge notebook
        </p>
        <h1 className="rise-in mt-5 font-display text-display-xl text-foreground [--rise-index:1]">
          For every question, an answer worth&nbsp;keeping.
        </h1>
        <p className="rise-in mt-6 max-w-lg text-lg leading-relaxed text-secondary-foreground [--rise-index:2]">
          Curio answers what you ask, then files it with your notes, plans and
          saved answers, so what you learn today is still there next month.
        </p>
        <div className="rise-in mt-9 flex flex-col gap-3 sm:flex-row [--rise-index:3]">
          <Button asChild size="lg">
            <Link to="/login">
              Start exploring
              <ArrowRight aria-hidden="true" />
            </Link>
          </Button>
          <Button type="button" variant="outline" size="lg" onClick={onHowItWorks}>
            See how it works
          </Button>
        </div>
      </div>
      <div className="rise-in [--rise-index:2]">
        <ProductSketch />
      </div>
    </section>
  );
}

const STEPS = [
  {
    title: "Ask anything",
    description:
      "Get a clear answer, then keep the ones worth keeping as part of a history you can search.",
  },
  {
    title: "Find it again",
    description:
      "Search every note, topic and answer in one place and go straight back to what mattered.",
  },
  {
    title: "Connect ideas",
    description:
      "Link notes and answers over time so what you learn gains structure you can reuse.",
  },
  {
    title: "Keep learning",
    description:
      "Turn everyday questions into lasting understanding, in a vault that compounds as you use it.",
  },
];

export function HowItWorksSection() {
  return (
    <section
      id="how-it-works"
      aria-labelledby="how-it-works-title"
      className="scroll-mt-20 border-t border-border bg-card"
    >
      <div className="mx-auto w-full max-w-[76rem] px-5 py-20 sm:px-8 lg:py-28">
        <div className="max-w-3xl">
          <p className="eyebrow text-muted-foreground">How it works</p>
          <h2
            id="how-it-works-title"
            className="mt-4 font-display text-display-lg text-foreground"
          >
            Questions in. Understanding out.
          </h2>
        </div>
        <ol className="mt-14 grid gap-x-8 gap-y-12 sm:grid-cols-2 lg:grid-cols-4">
          {STEPS.map((step, index) => (
            <li key={step.title} className="border-t border-foreground/80 pt-5">
              <span className="font-mono text-xs tabular-nums text-primary">
                {String(index + 1).padStart(2, "0")}
              </span>
              <h3 className="mt-5 font-display text-display-sm text-foreground">
                {step.title}
              </h3>
              <p className="mt-3 text-[0.9375rem] leading-relaxed text-muted-foreground">
                {step.description}
              </p>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}

const SURFACES = [
  {
    icon: MessageSquare,
    name: "Chat",
    description: "Ask, follow up, and watch the answer stream in as it's written.",
  },
  {
    icon: NotebookPen,
    name: "Notes",
    description: "A rich-text notebook with folders, headings, code and tables.",
  },
  {
    icon: CalendarDays,
    name: "Calendar",
    description: "Month, week, day and agenda views, with tasks as a list or a board.",
  },
  {
    icon: BookOpen,
    name: "Vault",
    description: "Every answer you kept, searchable by phrase and category.",
  },
  {
    icon: Globe,
    name: "Atlas",
    description: "A map of how your ideas connect, drawn as you link them.",
  },
];

export function SurfacesSection() {
  return (
    <section aria-labelledby="surfaces-title" className="border-t border-border">
      <div className="mx-auto grid w-full max-w-[76rem] gap-x-16 gap-y-10 px-5 py-20 sm:px-8 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:py-28">
        <div className="lg:sticky lg:top-28 lg:self-start">
          <p className="eyebrow text-muted-foreground">Inside Curio</p>
          <h2
            id="surfaces-title"
            className="mt-4 font-display text-display-lg text-foreground"
          >
            One notebook, five ways in.
          </h2>
          <p className="mt-5 max-w-md text-lg leading-relaxed text-secondary-foreground">
            The same workspace on the web and on your desktop, with a single
            place for what you ask, write and plan.
          </p>
        </div>
        <ul className="border-t border-border">
          {SURFACES.map(({ icon: Icon, name, description }) => (
            <li
              key={name}
              className="grid grid-cols-[2.5rem_minmax(0,1fr)] items-start gap-4 border-b border-border py-6"
            >
              <span className="flex size-10 items-center justify-center rounded-md bg-secondary text-foreground">
                <Icon className="size-[1.125rem]" aria-hidden="true" />
              </span>
              <span>
                <span className="block font-display text-display-sm text-foreground">
                  {name}
                </span>
                <span className="mt-1 block text-[0.9375rem] leading-relaxed text-muted-foreground">
                  {description}
                </span>
              </span>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}

export function ContactSection() {
  const [submitted, setSubmitted] = useState(false);

  // No contact endpoint exists yet; say so instead of pretending to send.
  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setSubmitted(true);
  };

  return (
    <section
      id="contact"
      aria-labelledby="contact-title"
      className="scroll-mt-20 border-t border-border bg-card"
    >
      <div className="mx-auto grid w-full max-w-[76rem] gap-x-16 gap-y-10 px-5 py-20 sm:px-8 lg:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] lg:py-28">
        <div>
          <p className="eyebrow text-muted-foreground">Contact</p>
          <h2
            id="contact-title"
            className="mt-4 font-display text-display-lg text-foreground"
          >
            Get in touch.
          </h2>
          <p className="mt-5 max-w-md text-lg leading-relaxed text-secondary-foreground">
            Questions about Curio, ideas for it, or something that broke. We
            read everything.
          </p>
        </div>

        <form className="space-y-5" onSubmit={handleSubmit}>
          <div className="grid gap-5 sm:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="contact-name">Name</Label>
              <Input id="contact-name" autoComplete="name" placeholder="Your name" className="h-11" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="contact-email">Email</Label>
              <Input
                id="contact-email"
                type="email"
                autoComplete="email"
                placeholder="you@example.com"
                className="h-11"
              />
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="contact-subject">Subject</Label>
            <Input id="contact-subject" placeholder="What’s this about?" className="h-11" />
          </div>
          <div className="space-y-2">
            <Label htmlFor="contact-message">Message</Label>
            <Textarea
              id="contact-message"
              placeholder="Tell us more…"
              className="min-h-40 resize-y"
            />
          </div>
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center">
            <Button type="submit" size="lg">
              Send message
            </Button>
          </div>
          {submitted ? (
            <Notice title="This form isn’t connected yet">
              Nothing was sent. Messages will reach us once the contact service
              is live.
            </Notice>
          ) : null}
        </form>
      </div>
    </section>
  );
}

export function LandingFooter() {
  return (
    <footer className="border-t border-border">
      <div className="mx-auto flex w-full max-w-[76rem] flex-col items-start justify-between gap-4 px-5 py-10 text-sm text-muted-foreground sm:flex-row sm:items-center sm:px-8">
        <CurioWordmark className="text-base text-foreground" />
        <p>© 2026 Curio. All rights reserved.</p>
      </div>
    </footer>
  );
}
