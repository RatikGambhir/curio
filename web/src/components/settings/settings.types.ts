import type { LucideIcon } from "lucide-react"
import {
  Contact,
  Database,
  FolderOpen,
  KeyRound,
  Palette,
  UserRound,
  Waypoints,
} from "lucide-react"

export type SettingsTabId =
  | "account"
  | "customization"
  | "history"
  | "models"
  | "api"
  | "attachments"
  | "contact"

export type SettingsTab = {
  id: SettingsTabId
  label: string
  icon: LucideIcon
}

export const SETTINGS_TABS: SettingsTab[] = [
  { id: "account", label: "Account", icon: UserRound },
  { id: "customization", label: "Customization", icon: Palette },
  { id: "history", label: "History & Sync", icon: Database },
  { id: "models", label: "Models", icon: Waypoints },
  { id: "api", label: "API Keys", icon: KeyRound },
  { id: "attachments", label: "Attachments", icon: FolderOpen },
  { id: "contact", label: "Contact", icon: Contact },
]

export type AttachmentFilter = "all" | "images" | "documents"

export const ATTACHMENT_FILTER_OPTIONS: {
  value: AttachmentFilter
  label: string
}[] = [
  { value: "all", label: "All files" },
  { value: "images", label: "Images" },
  { value: "documents", label: "Documents" },
]

export type AttachmentRecord = {
  id: string
  name: string
  createdAt: string
  type: "image" | "document"
  size?: string
}

export type AttachmentSortDirection = "asc" | "desc"
