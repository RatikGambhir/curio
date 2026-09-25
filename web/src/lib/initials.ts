/* Two-letter monogram for avatars. Shared so the spine, settings and profile
   setup agree on how a name is abbreviated. */
export function initialsFor(name: string | undefined, fallback = "CU"): string {
  const initials = (name ?? "")
    .split(/\s+/)
    .filter(Boolean)
    .map((part) => part.charAt(0))
    .join("")
    .slice(0, 2)
    .toUpperCase()

  return initials || fallback
}
