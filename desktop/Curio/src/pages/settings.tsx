import { AppSidebar } from "@/components/app-sidebar";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";

function SettingsPage() {
  return (
    <SidebarProvider className="[--sidebar:var(--background)] [--sidebar-accent:var(--background)] [--sidebar-border:var(--border)]">
      <AppSidebar />
      <SidebarInset className="bg-muted/20">
        <div className="flex h-full w-full flex-col">
          <SidebarTrigger className="m-2" />
          <div className="mx-auto flex w-full max-w-3xl flex-1 items-start px-6 pb-10 pt-4">
            <div className="w-full rounded-3xl border border-border/80 bg-card p-8 shadow-sm">
              <div className="mb-8 flex items-center justify-between gap-4">
                <div>
                  <h1 className="text-3xl font-semibold tracking-tight">Profile Settings</h1>
                  <p className="mt-2 text-muted-foreground">
                    A desktop-safe placeholder shell for the current settings route.
                  </p>
                </div>
                <Button>Save Changes</Button>
              </div>

              <div className="space-y-6">
                <div className="grid gap-2">
                  <label className="text-sm font-medium">Full Name</label>
                  <Input defaultValue="John Doe" className="h-11 rounded-xl" />
                </div>
                <div className="grid gap-2">
                  <label className="text-sm font-medium">Email Address</label>
                  <Input
                    defaultValue="john.doe@example.com"
                    type="email"
                    className="h-11 rounded-xl"
                  />
                </div>
                <div className="grid gap-2">
                  <label className="text-sm font-medium">Bio</label>
                  <Textarea
                    defaultValue="Software developer passionate about creating amazing user experiences."
                    rows={5}
                    className="rounded-2xl"
                  />
                </div>
                <div className="grid gap-2">
                  <label className="text-sm font-medium">Location</label>
                  <Input defaultValue="San Francisco, CA" className="h-11 rounded-xl" />
                </div>
                <div className="grid gap-2">
                  <label className="text-sm font-medium">Website</label>
                  <Input defaultValue="https://johndoe.dev" className="h-11 rounded-xl" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

export default SettingsPage;
