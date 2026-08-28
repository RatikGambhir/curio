import type { ReactNode } from "react"
import type { Value } from "platejs"

/**
 * Plate's document model: an array of element nodes stored as JSON.
 *
 * This is the authoritative note format. HTML is an export-boundary format
 * only, so bodies are never round-tripped through an HTML string.
 */
export type RichTextValue = Value

/** Resolves a picked file to a URL the editor can render. */
export type ImageUploader = (file: File) => Promise<string> | string

export type RichTextEditorProps = {
  /** Controlled document. External updates never re-emit `onChange`. */
  value?: RichTextValue
  /** Initial document when the editor is left uncontrolled. */
  defaultValue?: RichTextValue
  onChange?: (value: RichTextValue) => void
  /** Fired on Cmd/Ctrl+S. */
  onSave?: (value: RichTextValue) => void
  onImageUpload?: ImageUploader
  autoFocus?: boolean
  readOnly?: boolean
  placeholder?: string
  className?: string
  /**
   * Rendered at the start of the toolbar row. It sits inside the editor's
   * context, which is what lets a page put its own title beside the controls.
   */
  header?: ReactNode
  /** Rendered at the end of the toolbar row. */
  headerTrailing?: ReactNode
  /**
   * Drop the card chrome and fill the parent instead, turning the toolbar row
   * into the surrounding page's header bar.
   */
  flush?: boolean
}

export const emptyRichTextValue = (): RichTextValue => [
  { type: "p", children: [{ text: "" }] },
]
