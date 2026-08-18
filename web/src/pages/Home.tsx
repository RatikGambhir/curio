import { AppSidebar } from "@/components/app-sidebar";
import { PageHeader } from "@/components/page-header";
import {
  SidebarInset,
  SidebarProvider,
} from "@/components/ui/sidebar";

const Home = () => {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="bg-background">
        <div className="flex h-full w-full flex-col bg-background">
          <PageHeader />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Home;
