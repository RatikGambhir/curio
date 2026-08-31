import type { LucideIcon } from "lucide-react";

export type UpcomingItem = {
  title: string;
  time: string;
  dateTime: string;
  duration: string;
  dotClassName: string;
};

export type Cultivation = {
  title: string;
  updated: string;
  tags: readonly string[];
  icon: LucideIcon;
  iconClassName: string;
  imageSrc?: string;
};
