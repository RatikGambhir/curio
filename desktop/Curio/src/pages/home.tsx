import { AppSidebar } from "@/components/app-sidebar";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";

function HomePage() {
  return (
    <SidebarProvider className="[--sidebar:var(--background)] [--sidebar-accent:var(--background)] [--sidebar-border:var(--border)]">
      <AppSidebar />
      <SidebarInset className="bg-background">
        <div className="flex h-full w-full flex-col bg-background">
          <SidebarTrigger className="m-2" />
          <div className="flex flex-1 items-center justify-center px-8 pb-10">
            <div className="max-w-2xl space-y-4 text-center">
              <p
                className="text-sm uppercase tracking-[0.35em] text-muted-foreground"
                style={{ fontFamily: "var(--font-serif)" }}
              >
                Curio Workspace
              </p>
              <h1 className="text-4xl font-semibold tracking-tight text-foreground">
                Your desktop shell now matches the web product direction.
              </h1>
              <p className="text-lg leading-8 text-muted-foreground">
                Home intentionally stays minimal in the current web build, so the desktop
                version keeps the same airy shell and lets the sidebar carry the product
                chrome.
              </p>
            </div>
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

export default HomePage;
