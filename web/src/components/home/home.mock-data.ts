import { BrainCircuit, DraftingCompass, PlaneTakeoff } from "lucide-react";

import type { Cultivation, UpcomingItem } from "@/components/home/home.types";

export const UPCOMING_ITEMS = [
  {
    title: "Design Sync",
    time: "10:00",
    dateTime: "10:00",
    duration: "45m",
    dotClassName: "bg-destructive",
  },
  {
    title: "Review Research",
    time: "14:30",
    dateTime: "14:30",
    duration: "1h",
    dotClassName: "bg-accent-brand",
  },
] satisfies readonly UpcomingItem[];

export const RECENT_CULTIVATIONS = [
  {
    title: "Design System Audit",
    updated: "2 hours ago",
    tags: ["ui/ux", "q2"],
    icon: DraftingCompass,
  },
  {
    title: "Kyoto Itinerary",
    updated: "Yesterday",
    tags: ["personal", "travel"],
    icon: PlaneTakeoff,
  },
  {
    title: "Cognitive Load Theory",
    updated: "3 days ago",
    tags: ["research"],
    icon: BrainCircuit,
  },
] satisfies readonly Cultivation[];
