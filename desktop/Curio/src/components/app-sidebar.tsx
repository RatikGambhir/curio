import * as React from "react";
import {
  BookOpen,
  Frame,
  Globe,
  House,
  Map,
  MessageSquare,
  PieChart,
  Settings2,
} from "lucide-react";
import { Link } from "react-router-dom";

import curioLogo from "@/assets/curio-logo.png";
import { NavMain } from "@/components/nav-main";
import { NavProjects } from "@/components/nav-projects";
import { NavUser } from "@/components/nav-user";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
  SidebarTrigger,
} from "@/components/ui/sidebar";

const data = {
  navMain: [
    { title: "Home", url: "/home", icon: House },
    { title: "Vaults", url: "/vault", icon: BookOpen },
    { title: "Chat", url: "/chat", icon: MessageSquare },
    { title: "Knowledge Atals", url: "#", icon: Globe },
    { title: "Settings", url: "/profile", icon: Settings2 },
  ],
  projects: [
    { name: "Design Engineering", url: "#", icon: Frame },
    { name: "Sales & Marketing", url: "#", icon: PieChart },
    { name: "Travel", url: "#", icon: Map },
  ],
};

export function AppSidebar(props: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader className="h-16 shrink-0 justify-center border-b border-sidebar-border p-2">
        <div className="grid h-full grid-cols-[1fr_auto] items-center gap-2 group-data-[collapsible=icon]:grid-cols-1 group-data-[collapsible=icon]:place-items-center">
          <Link
            to="/home"
            className="flex min-w-0 items-center gap-2 px-2 group-data-[collapsible=icon]:hidden"
          >
            <img
              src={curioLogo}
              alt="Curio Logo"
              className="h-10 w-auto object-contain"
              data-image="logo"
              style={{ mixBlendMode: "multiply" }}
            />
            <span
              className="truncate text-lg leading-none tracking-tight"
              style={{ fontFamily: "var(--font-serif)" }}
            >
              curio
            </span>
          </Link>
          <SidebarTrigger className="rounded-full text-muted-foreground hover:bg-white/45 hover:text-foreground" />
        </div>
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
