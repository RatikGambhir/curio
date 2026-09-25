import { useMemo, useState } from "react"
import { useSearchParams } from "react-router-dom"

import { PageHeader } from "@/components/page-header"
import { SettingsNav } from "@/components/settings/settings-nav"
import {
  SETTINGS_TABS,
  type AttachmentFilter,
  type AttachmentRecord,
  type AttachmentSortDirection,
  type SettingsTabId,
} from "@/components/settings/settings.types"
import { AccountTab } from "@/components/settings/tabs/account-tab"
import { ApiKeysTab } from "@/components/settings/tabs/api-keys-tab"
import { AttachmentsTab } from "@/components/settings/tabs/attachments-tab"
import { ContactTab } from "@/components/settings/tabs/contact-tab"
import { CustomizationTab } from "@/components/settings/tabs/customization-tab"
import { HistorySyncTab } from "@/components/settings/tabs/history-sync-tab"
import { ModelsTab } from "@/components/settings/tabs/models-tab"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"

const DEFAULT_TAB: SettingsTabId = "account"

function parseTab(value: string | null): SettingsTabId {
  return SETTINGS_TABS.find((tab) => tab.id === value)?.id ?? DEFAULT_TAB
}

function ProfileSettings() {
  const { user } = useAuthenticatedUser()
  // The open section lives in the URL, so a link can open Appearance
  // directly and the browser's back button steps between sections.
  const [searchParams, setSearchParams] = useSearchParams()
  const activeTab = parseTab(searchParams.get("tab"))

  const [selectedFilter, setSelectedFilter] = useState<AttachmentFilter>("all")
  const [sortDirection, setSortDirection] =
    useState<AttachmentSortDirection>("desc")
  const [attachments] = useState<AttachmentRecord[]>([])

  const profileSummary = useMemo(
    () => ({
      name: user?.name ?? "Curio Member",
      email: user?.email ?? "member@curio.app",
      avatarUrl: user?.avatar,
      planLabel: "base plan",
    }),
    [user],
  )

  const sortedAttachments = useMemo(() => {
    const filtered =
      selectedFilter === "all"
        ? attachments
        : attachments.filter(
            (attachment) =>
              attachment.type === (selectedFilter === "images" ? "image" : "document"),
          )
    const sortMultiplier = sortDirection === "asc" ? 1 : -1

    return [...filtered].sort(
      (left, right) =>
        (Date.parse(left.createdAt) - Date.parse(right.createdAt)) * sortMultiplier,
    )
  }, [attachments, selectedFilter, sortDirection])

  const handleTabChange = (tab: SettingsTabId) => {
    setSearchParams(tab === DEFAULT_TAB ? {} : { tab })
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
    <>
      <PageHeader title="Settings" />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto grid w-full max-w-[72rem] gap-x-14 gap-y-8 px-5 pb-16 pt-8 sm:px-8 lg:grid-cols-[15rem_minmax(0,1fr)] lg:px-12 lg:pt-12">
          <SettingsNav
            profile={profileSummary}
            activeTab={activeTab}
            onTabChange={handleTabChange}
          />
          <div key={activeTab} className="min-w-0">
            {renderTabContent()}
          </div>
        </div>
      </div>
    </>
  )
}

export default ProfileSettings
