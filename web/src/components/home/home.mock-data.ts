import { BrainCircuit, DraftingCompass, PlaneTakeoff } from "lucide-react";

import cognitiveBackdrop from "@/assets/download.jpg";
import type { Cultivation, UpcomingItem } from "@/components/home/home.types";

export const UPCOMING_ITEMS = [
  {
    title: "Design Sync",
    time: "10:00 AM",
    dateTime: "10:00",
    duration: "45m",
    dotClassName: "bg-destructive",
  },
  {
    title: "Review Research",
    time: "2:30 PM",
    dateTime: "14:30",
    duration: "1h",
    dotClassName: "bg-accent-brand",
  },
] satisfies readonly UpcomingItem[];

export const RECENT_CULTIVATIONS = [
  {
    title: "Design System Audit",
    updated: "Updated 2 hours ago",
    tags: ["UI/UX", "Q2"],
    icon: DraftingCompass,
    iconClassName: "bg-accent-subtle text-primary",
  },
  {
    title: "Kyoto Itinerary",
    updated: "Updated yesterday",
    tags: ["Personal", "Travel"],
    icon: PlaneTakeoff,
    iconClassName: "bg-destructive/10 text-destructive",
  },
  {
    title: "Cognitive Load Theory",
    updated: "Updated 3 days ago",
    tags: ["Research"],
    icon: BrainCircuit,
    iconClassName: "bg-card/90 text-foreground",
    imageSrc: cognitiveBackdrop,
  },
] satisfies readonly Cultivation[];
