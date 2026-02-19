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
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="ml-[var(--sidebar-width)] w-[calc(100vw-var(--sidebar-width))]">
        <SidebarTrigger />
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Home;
