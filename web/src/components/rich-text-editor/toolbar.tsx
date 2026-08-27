import { useRef, useState } from "react"
import type { ReactNode } from "react"
import { toggleCodeBlock } from "@platejs/code-block"
import { upsertLink } from "@platejs/link"
import { useListToolbarButton, useListToolbarButtonState } from "@platejs/list/react"
import { insertTable } from "@platejs/table"
import {
  Baseline,
  Bold,
  Code,
  Heading1,
  Heading2,
  Heading3,
  ImagePlus,
  Italic,
  Link2,
  List,
  ListOrdered,
  Minus,
  Quote,
  SquareCode,
  Strikethrough,
  Subscript,
  Superscript,
  Table as TableIcon,
  Underline,
} from "lucide-react"
import { KEYS } from "platejs"
import type { TElement } from "platejs"
import { useEditorReadOnly, useEditorRef } from "platejs/react"

import { cn } from "@/lib/utils"

import {
  useCodeBlockEntry,
  useIsBlockActive,
  useIsMarkActive,
  useMarkValue,
} from "./editor-state"
import { codeBlockLanguages } from "./plugins"
import type { ImageUploader } from "./types"

const fontFamilies = [
  { label: "Sans", value: "var(--font-sans)" },
  { label: "Serif", value: "var(--font-serif)" },
  { label: "Mono", value: "var(--font-mono)" },
]

const fontSizes = ["12px", "14px", "16px", "18px", "24px", "32px"]

const controlClassName =
  "h-8 rounded-md border border-transparent bg-transparent px-2 text-xs text-muted-foreground outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:ring-2 focus-visible:ring-ring"

export function ToolbarButton({
  active = false,
  children,
  disabled = false,
  label,
  onClick,
}: {
  active?: boolean
  children: ReactNode
  disabled?: boolean
  label: string
  onClick: () => void
}) {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      aria-pressed={active}
      disabled={disabled}
      onMouseDown={(event) => event.preventDefault()}
      onClick={onClick}
      className={cn(
        "inline-flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:pointer-events-none disabled:opacity-40",
        active && "bg-accent text-accent-foreground",
      )}
    >
      {children}
    </button>
  )
}

export function ToolbarSeparator() {
  return <span aria-hidden className="mx-1 h-5 w-px shrink-0 bg-border" />
}

export function MarkButton({
  icon,
  label,
  markKey,
}: {
  icon: ReactNode
  label: string
  markKey: string
}) {
  const editor = useEditorRef()
  const active = useIsMarkActive(markKey)

  return (
    <ToolbarButton
      active={active}
      label={label}
      onClick={() => editor.tf.toggleMark(markKey)}
    >
      {icon}
    </ToolbarButton>
  )
}

export function BlockButton({
  icon,
  label,
  type,
}: {
  icon: ReactNode
  label: string
  type: string
}) {
  const editor = useEditorRef()
  const active = useIsBlockActive(type)

  return (
    <ToolbarButton
      active={active}
      label={label}
      onClick={() => editor.tf.toggleBlock(type)}
    >
      {icon}
    </ToolbarButton>
  )
}

export function ListButton({
  icon,
  label,
  listStyleType,
}: {
  icon: ReactNode
  label: string
  listStyleType: string
}) {
  const state = useListToolbarButtonState({ nodeType: listStyleType })
  const { props } = useListToolbarButton(state)

  return (
    <ToolbarButton active={props.pressed} label={label} onClick={props.onClick}>
      {icon}
    </ToolbarButton>
  )
}

export function CodeBlockButton() {
  const editor = useEditorRef()
  const active = useIsBlockActive(KEYS.codeBlock)

  return (
    <ToolbarButton
      active={active}
      label="Code block"
      onClick={() => toggleCodeBlock(editor)}
    >
      <SquareCode className="size-4" />
    </ToolbarButton>
  )
}

export function CodeBlockLanguageSelect() {
  const editor = useEditorRef()
  const entry = useCodeBlockEntry(KEYS.codeBlock)
  const language = typeof entry?.[0].lang === "string" ? entry[0].lang : ""

  if (!entry) {
    return null
  }

  return (
    <select
      aria-label="Code block language"
      value={language}
      onChange={(event) => {
        editor.tf.setNodes({ lang: event.target.value }, { at: entry[1] })
      }}
      className={controlClassName}
    >
      <option value="">Plain text</option>
      {Object.entries(codeBlockLanguages).map(([value, label]) => (
        <option key={value} value={value}>
          {label}
        </option>
      ))}
    </select>
  )
}

export function HorizontalRuleButton() {
  const editor = useEditorRef()

  return (
    <ToolbarButton
      label="Divider"
      onClick={() => {
        editor.tf.insertNodes({
          type: KEYS.hr,
          children: [{ text: "" }],
        } as TElement)
      }}
    >
      <Minus className="size-4" />
    </ToolbarButton>
  )
}

export function LinkButton() {
  const editor = useEditorRef()
  const [isOpen, setIsOpen] = useState(false)
  const [url, setUrl] = useState("")
  const active = useIsBlockActive(KEYS.link)

  const submit = () => {
    const trimmed = url.trim()
    setIsOpen(false)
    setUrl("")

    if (trimmed) {
      upsertLink(editor, { url: trimmed })
    }
  }

  return (
    <div className="relative">
      <ToolbarButton
        active={active || isOpen}
        label="Link"
        onClick={() => setIsOpen((open) => !open)}
      >
        <Link2 className="size-4" />
      </ToolbarButton>
      {isOpen ? (
        <div className="absolute top-9 left-0 z-30 flex items-center gap-1 rounded-md border border-border bg-popover p-1 shadow-md">
          <input
            autoFocus
            type="url"
            value={url}
            placeholder="https://example.com"
            onChange={(event) => setUrl(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault()
                submit()
              }
              if (event.key === "Escape") {
                setIsOpen(false)
              }
            }}
            className="h-7 w-56 rounded-sm bg-transparent px-2 text-xs text-popover-foreground outline-none"
          />
          <button
            type="button"
            onMouseDown={(event) => event.preventDefault()}
            onClick={submit}
            className="h-7 rounded-sm bg-primary px-2 text-xs text-primary-foreground"
          >
            Apply
          </button>
        </div>
      ) : null}
    </div>
  )
}

export function ImageButton({ onImageUpload }: { onImageUpload?: ImageUploader }) {
  const editor = useEditorRef()
  const inputRef = useRef<HTMLInputElement>(null)

  const insertImage = async (file: File) => {
    const url = onImageUpload
      ? await onImageUpload(file)
      : URL.createObjectURL(file)

    editor.tf.insertNodes({
      type: KEYS.img,
      url,
      width: 100,
      children: [{ text: "" }],
    } as TElement)
  }

  return (
    <>
      <ToolbarButton label="Image" onClick={() => inputRef.current?.click()}>
        <ImagePlus className="size-4" />
      </ToolbarButton>
      <input
        ref={inputRef}
        type="file"
        accept="image/*"
        className="hidden"
        onChange={(event) => {
          const file = event.target.files?.[0]
          event.target.value = ""

          if (file) {
            void insertImage(file)
          }
        }}
      />
    </>
  )
}

export function TableButton() {
  const editor = useEditorRef()

  return (
    <ToolbarButton
      label="Table"
      onClick={() => insertTable(editor, { colCount: 3, rowCount: 3 })}
    >
      <TableIcon className="size-4" />
    </ToolbarButton>
  )
}

export function FontFamilySelect() {
  const editor = useEditorRef()
  const value = useMarkValue(KEYS.fontFamily) ?? ""

  return (
    <select
      aria-label="Font family"
      value={value}
      onChange={(event) => {
        const next = event.target.value
        if (next) {
          editor.tf.addMark(KEYS.fontFamily, next)
        } else {
          editor.tf.removeMark(KEYS.fontFamily)
        }
      }}
      className={controlClassName}
    >
      <option value="">Font</option>
      {fontFamilies.map((font) => (
        <option key={font.value} value={font.value}>
          {font.label}
        </option>
      ))}
    </select>
  )
}

export function FontSizeSelect() {
  const editor = useEditorRef()
  const value = useMarkValue(KEYS.fontSize) ?? ""

  return (
    <select
      aria-label="Font size"
      value={value}
      onChange={(event) => {
        const next = event.target.value
        if (next) {
          editor.tf.addMark(KEYS.fontSize, next)
        } else {
          editor.tf.removeMark(KEYS.fontSize)
        }
      }}
      className={controlClassName}
    >
      <option value="">Size</option>
      {fontSizes.map((size) => (
        <option key={size} value={size}>
          {size.replace("px", "")}
        </option>
      ))}
    </select>
  )
}

export function FontColorInput() {
  const editor = useEditorRef()
  const value = useMarkValue(KEYS.color)

  return (
    <label
      title="Text color"
      className="inline-flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
    >
      {/* With no colour mark the swatch inherits the toolbar's foreground, so
          it stays visible in both themes. */}
      <Baseline className="size-4" style={value ? { color: value } : undefined} />
      <input
        type="color"
        aria-label="Text color"
        value={value ?? "#000000"}
        onChange={(event) => editor.tf.addMark(KEYS.color, event.target.value)}
        className="sr-only"
      />
    </label>
  )
}

export function RichTextToolbar({ onImageUpload }: { onImageUpload?: ImageUploader }) {
  const readOnly = useEditorReadOnly()

  if (readOnly) {
    return null
  }

  return (
    <div className="flex flex-wrap items-center gap-0.5 border-b border-border bg-card/60 px-2 py-1.5">
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
      <MarkButton
        icon={<Subscript className="size-4" />}
        label="Subscript"
        markKey={KEYS.sub}
      />
      <MarkButton
        icon={<Superscript className="size-4" />}
        label="Superscript"
        markKey={KEYS.sup}
      />

      <ToolbarSeparator />

      <BlockButton icon={<Heading1 className="size-4" />} label="Heading 1" type={KEYS.h1} />
      <BlockButton icon={<Heading2 className="size-4" />} label="Heading 2" type={KEYS.h2} />
      <BlockButton icon={<Heading3 className="size-4" />} label="Heading 3" type={KEYS.h3} />
      <BlockButton
        icon={<Quote className="size-4" />}
        label="Blockquote"
        type={KEYS.blockquote}
      />
      <CodeBlockButton />
      <HorizontalRuleButton />

      <ToolbarSeparator />

      <ListButton
        icon={<List className="size-4" />}
        label="Bulleted list"
        listStyleType={KEYS.ul}
      />
      <ListButton
        icon={<ListOrdered className="size-4" />}
        label="Numbered list"
        listStyleType={KEYS.ol}
      />

      <ToolbarSeparator />

      <LinkButton />
      <ImageButton onImageUpload={onImageUpload} />
      <TableButton />

      <ToolbarSeparator />

      <FontFamilySelect />
      <FontSizeSelect />
      <FontColorInput />
      <CodeBlockLanguageSelect />
    </div>
  )
}
