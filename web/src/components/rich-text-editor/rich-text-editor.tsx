import { useEffect, useRef } from "react"
import type { KeyboardEvent } from "react"
import { Plate, PlateContent, usePlateEditor } from "platejs/react"

import { cn } from "@/lib/utils"

import { FloatingToolbar } from "./floating-toolbar"
import { richTextPlugins } from "./plugins"
import { RichTextToolbar } from "./toolbar"
import { emptyRichTextValue } from "./types"
import type { RichTextEditorProps, RichTextValue } from "./types"

/**
 * Plate-backed WYSIWYG surface.
 *
 * The document is JSON (`RichTextValue`), never HTML. Callers own persistence:
 * `onChange` fires on every edit and `onSave` on Cmd/Ctrl+S, and neither one
 * performs any network work of its own.
 */
export function RichTextEditor({
  autoFocus = false,
  className,
  defaultValue,
  onChange,
  onImageUpload,
  onSave,
  placeholder = "Start writing…",
  readOnly = false,
  value,
}: RichTextEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const initialValue = useRef(value ?? defaultValue ?? emptyRichTextValue())
  const editor = usePlateEditor({
    plugins: richTextPlugins,
    value: initialValue.current,
    autoSelect: autoFocus ? "end" : false,
  })

  // Echo guard: a `value` update that came from outside must not be re-emitted
  // through `onChange`, or a caller storing that value round-trips its own
  // write back into whichever document is mounted now.
  const lastValue = useRef<RichTextValue | undefined>(value)
  const isApplyingExternalValue = useRef(false)

  useEffect(() => {
    if (value === undefined || value === lastValue.current) {
      return
    }

    lastValue.current = value
    isApplyingExternalValue.current = true
    editor.tf.setValue(value)
    isApplyingExternalValue.current = false
  }, [editor, value])

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (!(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== "s") {
      return
    }

    // Keep the browser's own save dialog from opening over the editor.
    event.preventDefault()
    onSave?.(editor.children as RichTextValue)
  }

  return (
    <Plate
      editor={editor}
      readOnly={readOnly}
      onValueChange={({ value: nextValue }) => {
        const previousValue = lastValue.current
        lastValue.current = nextValue as RichTextValue

        if (isApplyingExternalValue.current || nextValue === previousValue) {
          return
        }

        onChange?.(nextValue as RichTextValue)
      }}
    >
      <div
        ref={containerRef}
        onKeyDown={handleKeyDown}
        className={cn(
          "flex min-h-0 w-full flex-1 flex-col overflow-hidden rounded-lg border border-border bg-card shadow-xs",
          className,
        )}
      >
        <RichTextToolbar onImageUpload={onImageUpload} />
        <div className="min-h-0 flex-1 overflow-y-auto">
          <PlateContent
            readOnly={readOnly}
            placeholder={placeholder}
            className="curio-rich-text mx-auto min-h-full w-full max-w-3xl px-6 py-8 text-[15px] text-foreground outline-none"
          />
        </div>
        <FloatingToolbar containerRef={containerRef} />
      </div>
    </Plate>
  )
}
