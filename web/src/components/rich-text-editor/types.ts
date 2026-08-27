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
}

export const emptyRichTextValue = (): RichTextValue => [
  { type: "p", children: [{ text: "" }] },
]
