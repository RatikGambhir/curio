import type { NodeEntry, TElement } from "platejs"
import { useEditorRef, useEditorSelection, useEditorVersion } from "platejs/react"

/**
 * Subscribes a toolbar control to both selection and content changes so its
 * active state stays in sync with the caret.
 */
function useEditorTick() {
  useEditorSelection()
  useEditorVersion()
}

export function useActiveMarks(): Record<string, unknown> {
  const editor = useEditorRef()
  useEditorTick()

  return (editor.api.marks() ?? {}) as Record<string, unknown>
}

export function useIsMarkActive(markKey: string): boolean {
  return Boolean(useActiveMarks()[markKey])
}

export function useMarkValue(markKey: string): string | undefined {
  const value = useActiveMarks()[markKey]
  return typeof value === "string" ? value : undefined
}

export function useIsBlockActive(type: string): boolean {
  const editor = useEditorRef()
  useEditorTick()

  if (!editor.selection) {
    return false
  }

  return editor.api.some({
    match: (node) => (node as TElement).type === type,
  })
}

/** The code block entry the caret sits in, if any. */
export function useCodeBlockEntry(type: string): NodeEntry<TElement> | undefined {
  const editor = useEditorRef()
  useEditorTick()

  if (!editor.selection) {
    return undefined
  }

  return editor.api.node<TElement>({
    match: (node) => (node as TElement).type === type,
  })
}
