import { useCallback, useState } from "react"

import { useMediaQuery } from "@/hooks/useMediaQuery"

/* Wide enough for the spine, a 17rem pane and a readable content column. */
const INLINE_QUERY = "(min-width: 64rem)"

export type ContextPaneState = {
  /** True when the pane sits beside the content rather than over it. */
  isInline: boolean
  open: boolean
  setOpen: (open: boolean) => void
  toggle: () => void
  /** Closes the pane only when it is covering the content. */
  dismissOverlay: () => void
}

/* Owns the pane's two lives: inline and open by default on wide screens, a
   dismissible sheet that starts closed on narrow ones. The two are tracked
   separately so hiding the inline pane does not also decide what a phone sees. */
export function useContextPane(): ContextPaneState {
  const isInline = useMediaQuery(INLINE_QUERY)
  const [inlineOpen, setInlineOpen] = useState(true)
  const [overlayOpen, setOverlayOpen] = useState(false)
  const open = isInline ? inlineOpen : overlayOpen
  const setOpen = isInline ? setInlineOpen : setOverlayOpen

  const toggle = useCallback(() => setOpen(!open), [open, setOpen])
  const dismissOverlay = useCallback(() => {
    if (!isInline) {
      setOverlayOpen(false)
    }
  }, [isInline])

  return { isInline, open, setOpen, toggle, dismissOverlay }
}
