import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { AppSidebar } from "@/components/app-sidebar";
import { SettingsSidebar } from "@/components/settings/settings-sidebar";
import { SettingsTabs } from "@/components/settings/settings-tabs";
import { AccountTab } from "@/components/settings/tabs/account-tab";
import { ApiKeysTab } from "@/components/settings/tabs/api-keys-tab";
import { AttachmentsTab } from "@/components/settings/tabs/attachments-tab";
import { ContactTab } from "@/components/settings/tabs/contact-tab";
import { CustomizationTab } from "@/components/settings/tabs/customization-tab";
import { HistorySyncTab } from "@/components/settings/tabs/history-sync-tab";
import { ModelsTab } from "@/components/settings/tabs/models-tab";
import type {
  AttachmentFilter,
  AttachmentRecord,
  AttachmentSortDirection,
  SettingsTabId,
} from "@/components/settings/settings.types";
import { Button } from "@/components/ui/button";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser";
import { ArrowLeft, LogOut, MessageSquareText } from "lucide-react";

function SettingsPage() {
  const navigate = useNavigate();
  const { user, logoutUser } = useAuthenticatedUser();

  const [activeTab, setActiveTab] = useState<SettingsTabId>("attachments");
  const [selectedFilter, setSelectedFilter] = useState<AttachmentFilter>("all");
  const [sortDirection, setSortDirection] =
    useState<AttachmentSortDirection>("desc");
  const [attachments] = useState<AttachmentRecord[]>([]);

  const profileSummary = useMemo(
    () => ({
      name: user?.name || "Curio Member",
      email: user?.email || "member@curio.app",
      avatarUrl: user?.avatar,
      planLabel: "Base plan",
    }),
    [user],
  );

  const filteredAttachments = useMemo(() => {
    if (selectedFilter === "all") {
      return attachments;
    }

    const fileType = selectedFilter === "images" ? "image" : "document";
    return attachments.filter((attachment) => attachment.type === fileType);
  }, [attachments, selectedFilter]);

  const sortedAttachments = useMemo(() => {
    const sortMultiplier = sortDirection === "asc" ? 1 : -1;

    return [...filteredAttachments].sort((left, right) => {
      const leftTimestamp = Date.parse(left.createdAt);
      const rightTimestamp = Date.parse(right.createdAt);
      return (leftTimestamp - rightTimestamp) * sortMultiplier;
    });
  }, [filteredAttachments, sortDirection]);

  const handleSignOut = () => {
    logoutUser();
    navigate("/login", { replace: true });
  };

  const renderTabContent = () => {
    switch (activeTab) {
      case "account":
        return <AccountTab />;
      case "customization":
        return <CustomizationTab />;
      case "history":
        return <HistorySyncTab />;
      case "models":
        return <ModelsTab />;
      case "api":
        return <ApiKeysTab />;
      case "attachments":
        return (
          <AttachmentsTab
            selectedFilter={selectedFilter}
            onFilterChange={setSelectedFilter}
            attachments={sortedAttachments}
            sortDirection={sortDirection}
            onSortDirectionChange={setSortDirection}
          />
        );
      case "contact":
        return <ContactTab />;
    }
  };

  return (
    <SidebarProvider className="[--sidebar:var(--background)] [--sidebar-accent:var(--background)] [--sidebar-border:var(--border)]">
      <AppSidebar />
      <SidebarInset className="bg-background">
        <div className="flex h-full w-full flex-col bg-background">
          <SidebarTrigger className="m-2" />

          <main className="flex-1 px-4 pb-6 sm:px-6 sm:pb-8 lg:px-8">
            <div className="mx-auto w-full max-w-[90rem]">
              <header className="mb-6 flex flex-wrap items-start justify-between gap-4 rounded-[1.75rem] border border-border bg-card p-4 shadow-sm sm:p-5">
                <div className="flex items-start gap-3">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="rounded-full"
                    onClick={() => navigate("/chat")}
                  >
                    <ArrowLeft className="size-4" />
                    Back to chat
                  </Button>
                  <div className="space-y-1">
                    <h1 className="text-2xl font-bold text-card-foreground">Settings</h1>
                    <p className="text-sm text-muted-foreground">
                      Manage your account and workspace preferences.
                    </p>
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="icon-sm"
                    className="rounded-full"
                    onClick={() => navigate("/chat")}
                    aria-label="Open chat"
                  >
                    <MessageSquareText className="size-4" />
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="rounded-full"
                    onClick={handleSignOut}
                  >
                    <LogOut className="size-4" />
                    Sign out
                  </Button>
                </div>
              </header>

              <div className="grid gap-4 lg:grid-cols-[19.5rem_minmax(0,1fr)] lg:items-start">
                <SettingsSidebar profile={profileSummary} />

                <section className="space-y-4">
                  <SettingsTabs activeTab={activeTab} onTabChange={setActiveTab} />
                  {renderTabContent()}
                </section>
              </div>
            </div>
          </main>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}

export default SettingsPage;
