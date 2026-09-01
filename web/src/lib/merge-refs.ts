import type { Ref } from "react"

type PossibleRef<T> = Ref<T> | ((node: T | null) => void) | null | undefined

/**
 * Combines several refs into a single callback ref so one node can be handed to
 * a forwarded ref and an internal ref at the same time.
 */
export function mergeRefs<T>(...refs: PossibleRef<T>[]) {
  return (node: T | null) => {
    for (const ref of refs) {
      if (typeof ref === "function") {
        ref(node)
      } else if (ref) {
        ;(ref as { current: T | null }).current = node
      }
    }
  }
}
