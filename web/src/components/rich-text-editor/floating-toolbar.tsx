import { useEffect, useState } from "react"
import type { RefObject } from "react"
import { autoUpdate, flip, offset, shift, useFloating } from "@floating-ui/react"
import { Bold, Code, Italic, Strikethrough, Underline } from "lucide-react"
import { KEYS } from "platejs"
import {
  useEditorReadOnly,
  useEditorRef,
  useEditorSelection,
  useSelectionVersion,
} from "platejs/react"

import { LinkButton, MarkButton, ToolbarSeparator } from "./toolbar"

/**
 * Selection-anchored formatting bar. It follows the DOM selection rectangle
 * through a floating-ui virtual reference rather than any editor element, so
 * it tracks the text itself.
 */
export function FloatingToolbar({
  containerRef,
}: {
  containerRef: RefObject<HTMLDivElement | null>
}) {
  const editor = useEditorRef()
  const readOnly = useEditorReadOnly()
  const selectionVersion = useSelectionVersion()
  const [hasFocus, setHasFocus] = useState(false)

  // Re-render whenever the caret moves. The selection object itself is not a
  // usable effect dependency because Plate reuses its identity.
  useEditorSelection()

  // Slate's own `useFocused` stays false for this tree, so focus is tracked
  // from the DOM. Watching the whole container — not just the editable — keeps
  // the bar up while one of its own controls has focus.
  useEffect(() => {
    const container = containerRef.current
    if (!container) {
      return
    }

    const handleFocusIn = (event: FocusEvent) => {
      setHasFocus(container.contains(event.target as Node | null))
    }
    const handleFocusOut = (event: FocusEvent) => {
      setHasFocus(container.contains(event.relatedTarget as Node | null))
    }

    setHasFocus(container.contains(document.activeElement))
    document.addEventListener("focusin", handleFocusIn)
    document.addEventListener("focusout", handleFocusOut)

    return () => {
      document.removeEventListener("focusin", handleFocusIn)
      document.removeEventListener("focusout", handleFocusOut)
    }
  }, [containerRef])

  const isVisible =
    !readOnly && hasFocus && Boolean(editor.selection) && editor.api.isExpanded()

  const { floatingStyles, refs } = useFloating({
    placement: "top",
    strategy: "fixed",
    middleware: [offset(8), flip({ padding: 8 }), shift({ padding: 8 })],
    whileElementsMounted: autoUpdate,
  })

  useEffect(() => {
    if (!isVisible) {
      return
    }

    const domSelection = window.getSelection()
    if (!domSelection || domSelection.rangeCount === 0) {
      return
    }

    const range = domSelection.getRangeAt(0)
    refs.setPositionReference({
      getBoundingClientRect: () => range.getBoundingClientRect(),
    })
  }, [isVisible, refs, selectionVersion])

  if (!isVisible) {
    return null
  }

  return (
    <div
      ref={refs.setFloating}
      style={floatingStyles}
      className="z-50 flex items-center gap-0.5 rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md"
    >
      <MarkButton icon={<Bold className="size-4" />} label="Bold" markKey={KEYS.bold} />
      <MarkButton icon={<Italic className="size-4" />} label="Italic" markKey={KEYS.italic} />
      <MarkButton
        icon={<Underline className="size-4" />}
        label="Underline"
        markKey={KEYS.underline}
      />
      <MarkButton
        icon={<Strikethrough className="size-4" />}
        label="Strikethrough"
        markKey={KEYS.strikethrough}
      />
      <MarkButton icon={<Code className="size-4" />} label="Inline code" markKey={KEYS.code} />
      <ToolbarSeparator />
      <LinkButton />
    </div>
  )
}
