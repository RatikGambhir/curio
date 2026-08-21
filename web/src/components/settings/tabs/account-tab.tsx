import { useState, type FormEvent } from "react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { useAuthenticatedUser } from "@/hooks/useAuthenticatedUser"
import { useSaveUser } from "@/hooks/useSaveUser"

export function AccountTab() {
  const { user } = useAuthenticatedUser()
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

  return (
    <section className="rounded-[2rem] border border-border bg-card p-8 shadow-md">
      <h2 className="text-2xl font-bold text-card-foreground">Account</h2>
      <p className="mt-2 text-sm text-muted-foreground">
        Update the profile Curio stores for your account.
      </p>

      <form className="mt-6 flex max-w-md flex-col gap-4" onSubmit={handleSubmit}>
        <div className="space-y-2">
          <Label htmlFor="account-name">Name</Label>
          <Input
            id="account-name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Your name"
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
            required
          />
        </div>

        <div className="flex items-center gap-3">
          <Button type="submit" disabled={!user || saveUser.isPending}>
            {saveUser.isPending ? "Saving..." : "Save changes"}
          </Button>
          {saveUser.isSuccess && (
            <p className="text-sm text-muted-foreground">Profile saved.</p>
          )}
          {saveUser.isError && (
            <p className="text-sm text-destructive">
              {saveUser.error instanceof Error
                ? saveUser.error.message
                : "Something went wrong while saving your profile."}
            </p>
          )}
        </div>
      </form>
    </section>
  )
}
