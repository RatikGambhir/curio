import { ChevronDown } from "lucide-react";

import curioLogo from "@/assets/curio-logo.png";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { HyperText } from "@/components/ui/hypertext";
import {cn} from "@/lib/utils.ts";
import {useEffect, useRef} from "react";

export type BookPage = {
  left: string;
  right: string;
};

export type FeatureTile = {
  title: string;
  subtitle?: string;
  description: string;
  className: string;
};

export const bookPages: BookPage[] = [
  {
    left: "Welcome to your knowledge journey",
    right: "Every question opens a new door to understanding",
  },
  {
    left: "Curiosity is the compass that guides us",
    right: "Through the endless landscape of learning",
  },
  {
    left: "Build connections between ideas",
    right: "Watch your insights grow organically",
  },
  {
    left: "Knowledge compounds over time",
    right: "Each answer sparks new questions",
  },
  {
    left: "Your vault of wisdom awaits",
    right: "Start exploring today",
  },
];

export const featureTiles: FeatureTile[] = [
  {
    title: "Instant Search",
    description:
      "Find any answer instantly across your entire knowledge vault with powerful full-text search.",
    className: "md:col-span-1 md:row-span-1",
  },
  {
    title: "Smart Notes",
    subtitle: "Auto-organized Q&A pairs that grow smarter over time",
    description:
      "Every question you ask becomes a searchable note. Watch your knowledge compound as connections emerge naturally.",
    className: "md:col-span-2 md:row-span-2",
  },
  {
    title: "Knowledge Vault",
    subtitle: "Your personal library of discoveries",
    description:
      "Build a living collection of insights that grows with every question you explore.",
    className: "md:col-span-1 md:row-span-2",
  },
  {
    title: "Tags & Categories",
    description:
      "Organize seamlessly with intelligent categorization and custom tags.",
    className: "md:col-span-2 md:row-span-1",
  },
  {
    title: "Continuous Learning",
    description: "Build knowledge that compounds over time.",
    className: "md:col-span-1 md:row-span-1",
  },
  {
    title: "Cross-Platform",
    description: "Learn anywhere, anytime, on any device.",
    className: "md:col-span-1 md:row-span-1",
  },
  {
    title: "Privacy First",
    description: "Your data, your control, always secured.",
    className: "md:col-span-1 md:row-span-1",
  },
];

type LandingNavbarProps = {
  onContactClick: () => void;
};

export function LandingNavbar({ onContactClick }: LandingNavbarProps) {
  return (
    <nav className="animate-in fade-in-0 slide-in-from-top-2 mx-auto flex w-full max-w-7xl items-center justify-between px-6 py-6 duration-500 md:px-8">
      <div className="h-10 w-10 rounded-md object-contain md:h-12 md:w-12">

      </div>

      <div className="flex items-center gap-2 md:gap-6">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="gap-1">
              Platforms
              <ChevronDown className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-40 rounded-xl">
            <DropdownMenuItem>Desktop</DropdownMenuItem>
            <DropdownMenuItem>Web</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        <Button variant="ghost" onClick={onContactClick}>
          Contact
        </Button>
      </div>
    </nav>
  );
}

interface HeroSectionProps {
  className?: string;
}

export const HeroSection = ({ className }: HeroSectionProps) => {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (videoRef.current) {
      videoRef.current.playbackRate = 0.40; // 0.5 = half speed, 0.75 = subtle slow
    }
  }, []);
  return (
      <section
          className={cn(
              "flex min-h-screen items-center justify-between py-14",
              className,
          )}
      >
        <div className="flex flex-col gap-3 px-[10%] lg:w-[50%] lg:pr-0">
          <h1
              className="animate-in fade-in-0 slide-in-from-bottom-4 mb-2 text-5xl font-semibold tracking-tight duration-500 md:text-7xl"
              style={{ fontFamily: "var(--font-serif)" }}
          >
            <span className="flex items-center">
              <span>curio</span>
              <img
                src={curioLogo}
                alt="Curio logo"
                className="h-6/12 w-6/12 rounded-md object-contain"
              />
            </span>
          </h1>

            <h2
                className="animate-in fade-in-0 slide-in-from-bottom-4 mb-5 text-2xl  tracking-tight duration-500 md:text-2xl"
                style={{ fontFamily: "var(--font-serif)" }}
            >
              For every question, there's an{" "}
              <HyperText
                as="h2"
                startOnView={true}
                animateOnHover={false}
                duration={2000}
                className=" py-0 text-inherit font-sans"
              >
                Answer
              </HyperText>
            </h2>


          <div className="flex flex-col gap-4 font-medium md:flex-row">
            <div className="animate-in fade-in-0 slide-in-from-bottom-2 flex flex-col items-center justify-center gap-4 duration-1000 sm:flex-row">
              <Button
                  size="lg"
                  className="h-14 rounded-xl px-8 text-lg transition-transform duration-200 hover:-translate-y-0.5"
              >
                Start Exploring
              </Button>
              <Button
                  variant="outline"
                  size="lg"
                  className="h-14 rounded-xl bg-card px-8 text-lg transition-transform duration-200 hover:-translate-y-0.5"
              >
                See How It Works
              </Button>
            </div>
          </div>
        </div>
        <div className="relative hidden h-[720px] w-[45%] overflow-hidden rounded-l-full bg-black lg:block">
          <video
              autoPlay
              ref={videoRef}
              loop
              muted
              playsInline
              data-wf-ignore="true"
              data-object-fit="cover"
              className="h-full w-full rounded-tl-xl object-cover"
          >
            <source src="https://deifkwefumgah.cloudfront.net/shadcnblocks/block/video-1.mp4" type="video/mp4" />
          </video>
        </div>
      </section>
  );
};

export function AppPreviewSection() {
  return (
    <section className="mx-auto w-full max-w-7xl px-6 pb-20 md:px-8 ">
      <Card className="animate-in fade-in-0 slide-in-from-bottom-6 overflow-hidden rounded-[1.75rem] border-border/80 bg-card/80 p-3 shadow-xl duration-700 transition-all hover:-translate-y-0.5 hover:shadow-2xl">
        <img
          src="https://images.unsplash.com/photo-1719938073286-437141b562e9?auto=format&fit=crop&w=1600&q=80"
          alt="Curio desktop preview"
          className="h-[260px] w-full rounded-2xl object-cover md:h-[560px]"
        />
      </Card>
    </section>
  );
}

type FeatureGridSectionProps = {
  features: FeatureTile[];
};

export function FeatureGridSection({ features }: FeatureGridSectionProps) {
  return (
    <section className="mx-auto w-full max-w-7xl px-6 pb-20 md:px-8">
      <div className="grid auto-rows-[220px] grid-cols-1 gap-5 md:grid-cols-3 md:auto-rows-[220px]">
        {features.map((feature, index) => (
          <Card
            key={feature.title}
            style={{ animationDelay: `${index * 80}ms` }}
            className={`animate-in fade-in-0 zoom-in-95 rounded-3xl bg-card/90 shadow-sm duration-500 transition-all hover:-translate-y-1 hover:shadow-lg ${feature.className}`}
          >
            <CardHeader>
              <CardTitle className="text-3xl md:text-4xl">{feature.title}</CardTitle>
              {feature.subtitle ? (
                <p className="text-lg text-foreground">{feature.subtitle}</p>
              ) : null}
            </CardHeader>
            <CardContent>
              <CardDescription className="text-base md:text-3xl/none">
                {feature.description}
              </CardDescription>
            </CardContent>
          </Card>
        ))}
      </div>
    </section>
  );
}

type BookSectionProps = {
  page: BookPage;
  onNextPage: () => void;
};

export function BookSection({ page, onNextPage }: BookSectionProps) {
  return (
    <section className="mx-auto w-full max-w-7xl px-6 pb-20 md:px-8">
      <Card className="animate-in fade-in-0 slide-in-from-bottom-6 rounded-3xl border-border/70 bg-accent/30 p-0 shadow-lg duration-700">
        <div className="grid min-h-[250px] grid-cols-1 overflow-hidden rounded-3xl md:grid-cols-2">
          <div className="border-border/70 p-8 md:border-r md:p-12">
            <p className="text-4xl tracking-tight md:text-5xl/none">{page.left}</p>
          </div>
          <div className="flex flex-col justify-between p-8 md:p-12">
            <p className="text-4xl tracking-tight md:text-5xl/none">{page.right}</p>
            <div className="mt-8 flex justify-end">
              <Button variant="ghost" onClick={onNextPage}>
                Click to turn page
              </Button>
            </div>
          </div>
        </div>
      </Card>
    </section>
  );
}

type ContactSectionProps = {
  className?: string;
};

export function ContactSection({ className }: ContactSectionProps) {
  return (
    <section
      id="contact"
      className={cn("mx-auto w-full max-w-5xl scroll-mt-8 px-6 pb-20", className)}
    >
      <Card className="animate-in fade-in-0 slide-in-from-bottom-6 rounded-3xl bg-card p-4 duration-700 md:p-6">
        <CardHeader className="text-center">
          <CardTitle className="text-4xl md:text-6xl">Get in Touch</CardTitle>
          <CardDescription className="mx-auto max-w-2xl text-base md:text-3xl/none">
            Have questions about Curio? We&apos;d love to hear from you.
          </CardDescription>
        </CardHeader>

        <CardContent>
          <form className="space-y-5">
            <div className="grid grid-cols-1 gap-5 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="name">Name</Label>
                <Input id="name" placeholder="Your name" className="h-12" />
              </div>
              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <Input
                  id="email"
                  type="email"
                  placeholder="your@email.com"
                  className="h-12"
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="subject">Subject</Label>
              <Input id="subject" placeholder="What&apos;s this about?" className="h-12" />
            </div>

            <div className="space-y-2">
              <Label htmlFor="message">Message</Label>
              <Textarea
                id="message"
                placeholder="Tell us more..."
                className="min-h-48 resize-none"
              />
            </div>

            <Button
              type="submit"
              size="lg"
              className="h-12 w-full rounded-xl transition-transform duration-200 hover:-translate-y-0.5"
            >
              Send Message
            </Button>
          </form>
        </CardContent>
      </Card>
    </section>
  );
}

export function LandingFooter() {
  return (
    <footer className="pb-10 text-center text-sm text-muted-foreground md:text-base">
      <p>© 2026 Curio. All rights reserved.</p>
    </footer>
  );
}
