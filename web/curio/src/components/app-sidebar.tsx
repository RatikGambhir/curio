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
import curioLogo from "../assets/curio-logo.png";
import { NavMain } from "@/components/nav-main";
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
  user: {
    name: "shadcn",
    email: "m@example.com",
    avatar: "/avatars/shadcn.jpg",
  },
  navMain: [
    {
      title: "Home",
      url: "/home",
      icon: House,
    },
    {
      title: "Vaults",
      url: "#",
      icon: BookOpen,
    },
    {
      title: "Chat",
      url: "/chat",
      icon: MessageSquare,
    },
    {
      title: "Knowledge Atals",
      url: "#",
      icon: Globe,
    },
    {
      title: "Settings",
      url: "/profile",
      icon: Settings2,
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
      <SidebarHeader className="px-2 pt-2 pb-0">
        <Link to="/home" className="flex items-center gap-2 px-2 py-1">
          <img
            src={curioLogo}
            alt="Curio Logo"
            className="h-12 w-auto object-contain"
            data-image="logo"
            style={{ mixBlendMode: "multiply" }}
          />
          <span
            className="truncate text-lg leading-none tracking-tight group-data-[collapsible=icon]:hidden"
            style={{ fontFamily: "var(--font-serif)" }}
          >
            curio
          </span>
        </Link>
      </SidebarHeader>
      <SidebarContent className="pt-0">
        <NavMain items={data.navMain} />
        <NavProjects projects={data.projects} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
