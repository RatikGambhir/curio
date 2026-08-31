import * as React from "react";
import {
  BookOpen,
  CalendarDays,
  Frame,
  Globe,
  House,
  Map,
  MessageSquare,
  NotebookPen,
  PieChart,
} from "lucide-react";
import { NavMain, PlatformHeader } from "@/components/nav-main";
import { NavProjects } from "@/components/nav-projects";
import { NavUser } from "@/components/nav-user";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar";

const data = {
  navMain: [
    {
      title: "Home",
      url: "/home",
      icon: House,
    },
    {
      title: "Vaults",
      url: "/vault",
      icon: BookOpen,
    },
    {
      title: "Chat",
      url: "/chat",
      icon: MessageSquare,
    },
    {
      title: "Calendar",
      url: "/calendar",
      icon: CalendarDays,
    },
    {
      title: "Notes",
      url: "/notes",
      icon: NotebookPen,
    },
    {
      title: "Atlas",
      url: "/atlas",
      icon: Globe,
    },
  ],
  projects: [
    {
      name: "Design Engineering",
      url: "#",
      icon: Frame,
    },
    {
      name: "Sales & Marketing",
      url: "#",
      icon: PieChart,
    },
    {
      name: "Travel",
      url: "#",
      icon: Map,
    },
  ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader className="h-(--app-header-height) shrink-0 justify-center p-0">
        <PlatformHeader />
      </SidebarHeader>
      <SidebarContent className="pt-0">
        <NavMain items={data.navMain} />
        <NavProjects projects={data.projects} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
