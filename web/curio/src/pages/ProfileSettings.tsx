import { useMemo, useRef, useState } from "react"
import { useNavigate } from "react-router-dom"
import { ApiKeysTab } from "@/components/settings/tabs/api-keys-tab"
import { AttachmentsTab } from "@/components/settings/tabs/attachments-tab"
import { ContactTab } from "@/components/settings/tabs/contact-tab"
import { CustomizationTab } from "@/components/settings/tabs/customization-tab"
import { HistorySyncTab } from "@/components/settings/tabs/history-sync-tab"
import { ModelsTab } from "@/components/settings/tabs/models-tab"
import { AccountTab } from "@/components/settings/tabs/account-tab"
import { SettingsSidebar } from "@/components/settings/settings-sidebar"
import { SettingsTabs } from "@/components/settings/settings-tabs"
import {
  type AttachmentFilter,
  type AttachmentRecord,
  type AttachmentSortDirection,
  type SettingsTabId,
} from "@/components/settings/settings.types"
import { Button } from "@/components/ui/button"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"
import { supabase } from "@/supabase"
import { ArrowLeft, LogOut, MessageSquareText } from "lucide-react"

function ProfileSettings() {
  const navigate = useNavigate()
  const { user, logoutUser } = useAuthenticatedUser()
  const hasTriggeredSignOutRef = useRef(false)

  const [activeTab, setActiveTab] = useState<SettingsTabId>("attachments")
  const [selectedFilter, setSelectedFilter] = useState<AttachmentFilter>("all")
  const [sortDirection, setSortDirection] =
    useState<AttachmentSortDirection>("desc")
  const [attachments] = useState<AttachmentRecord[]>([])
  const [isSigningOut, setIsSigningOut] = useState(false)

  const profileSummary = useMemo(
    () => ({
      name: "Curio Member",
      email: user ? `${user.slice(0, 8)}@curio.app` : "member@curio.app",
      planLabel: "Base plan",
    }),
    [user],
  )

  const filteredAttachments = useMemo(() => {
    if (selectedFilter === "all") {
      return attachments
    }

    const fileType = selectedFilter === "images" ? "image" : "document"
    return attachments.filter((attachment) => attachment.type === fileType)
  }, [attachments, selectedFilter])

  const sortedAttachments = useMemo(() => {
    const sortMultiplier = sortDirection === "asc" ? 1 : -1

    return [...filteredAttachments].sort((left, right) => {
      const leftTimestamp = Date.parse(left.createdAt)
      const rightTimestamp = Date.parse(right.createdAt)
      return (leftTimestamp - rightTimestamp) * sortMultiplier
    })
  }, [filteredAttachments, sortDirection])

  const handleSignOut = async () => {
    if (isSigningOut || hasTriggeredSignOutRef.current) {
      return
    }

    hasTriggeredSignOutRef.current = true
    setIsSigningOut(true)

    try {
      const { error } = await supabase.auth.signOut({ scope: "global" })
      if (error) {
        console.error("Failed to sign out from Supabase:", error.message)
      }

      logoutUser()
      navigate("/login", { replace: true })
    } finally {
      setIsSigningOut(false)
      hasTriggeredSignOutRef.current = false
    }
  }

  const renderTabContent = () => {
    switch (activeTab) {
      case "account":
        return <AccountTab />
      case "customization":
        return <CustomizationTab />
      case "history":
        return <HistorySyncTab />
      case "models":
        return <ModelsTab />
      case "api":
        return <ApiKeysTab />
      case "attachments":
        return (
          <AttachmentsTab
            selectedFilter={selectedFilter}
            onFilterChange={setSelectedFilter}
            attachments={sortedAttachments}
            sortDirection={sortDirection}
            onSortDirectionChange={setSortDirection}
          />
        )
      case "contact":
        return <ContactTab />
    }
  }

  return (
    <main className="min-h-screen bg-background px-4 py-6 sm:px-6 lg:px-8">
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
              disabled={isSigningOut}
            >
              <LogOut className="size-4" />
              {isSigningOut ? "Signing out..." : "Sign out"}
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
  )
}

export default ProfileSettings
