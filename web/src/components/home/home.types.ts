import type { LucideIcon } from "lucide-react";

export type UpcomingItem = {
  title: string;
  time: string;
  dateTime: string;
  duration: string;
  /** A semantic colour utility for the item's marker, e.g. `bg-destructive`. */
  dotClassName: string;
};

export type Cultivation = {
  title: string;
  updated: string;
  tags: readonly string[];
  icon: LucideIcon;
};
