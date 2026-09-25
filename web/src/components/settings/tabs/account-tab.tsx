import { useState, type FormEvent } from "react"
import { useNavigate } from "react-router-dom"
import { LogOut } from "lucide-react"

import { SettingsSection } from "@/components/settings/settings-section"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Notice } from "@/components/ui/notice"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"
import { useSaveUser } from "@/hooks/useSaveUser"

const USAGE_PERCENT = 42

export function AccountTab() {
  const navigate = useNavigate()
  const { user, logoutUser } = useAuthenticatedUser()
  const saveUser = useSaveUser()
  const [name, setName] = useState(user?.name ?? "")
  const [email, setEmail] = useState(user?.email ?? "")

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!user) {
      return
    }
    saveUser.mutate({ id: user.id, name: name.trim(), email: email.trim() })
  }

  const handleSignOut = () => {
    logoutUser()
    navigate("/login", { replace: true })
  }

  return (
    <div className="space-y-14">
      <SettingsSection
        title="Account"
        description="The profile Curio stores for you on the service."
      >
        <form className="flex max-w-md flex-col gap-5" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="account-name">Name</Label>
            <Input
              id="account-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Your name"
              autoComplete="name"
              required
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="account-email">Email</Label>
            <Input
              id="account-email"
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="you@example.com"
              autoComplete="email"
              required
            />
          </div>

          <div className="flex flex-wrap items-center gap-3 pt-1">
            <Button type="submit" disabled={!user || saveUser.isPending}>
              {saveUser.isPending ? "Saving…" : "Save changes"}
            </Button>
            {saveUser.isSuccess ? (
              <p role="status" className="text-sm text-muted-foreground">
                Profile saved.
              </p>
            ) : null}
          </div>

          {saveUser.isError ? (
            <Notice tone="error" title="Your profile wasn’t saved">
              {saveUser.error instanceof Error
                ? saveUser.error.message
                : "Something went wrong while saving your profile."}
            </Notice>
          ) : null}
        </form>
      </SettingsSection>

      <SettingsSection
        title="Plan"
        description="Usage is illustrative until billing is connected."
      >
        <div className="max-w-md">
          <div className="flex items-baseline justify-between gap-4">
            <p className="text-sm font-medium text-foreground">Base plan</p>
            <p className="font-mono text-xs tabular-nums text-muted-foreground">
              {USAGE_PERCENT}% used
            </p>
          </div>
          <div
            role="meter"
            aria-label="Plan usage"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={USAGE_PERCENT}
            className="mt-3 h-1.5 overflow-hidden rounded-full bg-secondary"
          >
            <div
              className="h-full rounded-full bg-primary"
              style={{ width: `${USAGE_PERCENT}%` }}
            />
          </div>
        </div>
      </SettingsSection>

      <SettingsSection
        title="Session"
        description="Signing out clears this device’s local session."
      >
        <Button type="button" variant="outline" onClick={handleSignOut}>
          <LogOut aria-hidden="true" />
          Sign out
        </Button>
      </SettingsSection>
    </div>
  )
}
