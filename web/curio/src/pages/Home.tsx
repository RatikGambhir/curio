import { AppSidebar } from "@/components/app-sidebar";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";

const Home = () => {
  const user = useAuthenticatedUser();
  console.log("HOME WITH USER");
  console.log(user?.user);
  return (
    <SidebarProvider className="[--sidebar:var(--background)] [--sidebar-accent:var(--background)] [--sidebar-border:var(--border)]">
      <AppSidebar />
      <SidebarInset className="bg-background">
        <SidebarTrigger className="m-2" />
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Home;
