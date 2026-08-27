import { CaptionTextarea } from "@platejs/caption/react"
import type { TImageElement, TLinkElement } from "platejs"
import {
  PlateElement,
  PlateLeaf,
  useEditorRef,
  useReadOnly,
  useSelected,
} from "platejs/react"
import type { PlateElementProps, PlateLeafProps } from "platejs/react"

import { cn } from "@/lib/utils"

/** Percentage widths offered by the image size control. */
const imageWidths = [40, 70, 100] as const

const paragraphClassName = "mb-3 leading-7 text-foreground"

export function ParagraphElement(props: PlateElementProps) {
  // An indented list row renders its <ul>/<ol> below this node, and <p> may not
  // legally contain one, so list rows render as a block <div> instead.
  if (props.element.listStyleType) {
    return <PlateElement {...props} className={paragraphClassName} />
  }

  return <PlateElement {...props} as="p" className={paragraphClassName} />
}

export function H1Element(props: PlateElementProps) {
  return (
    <PlateElement
      {...props}
      as="h1"
      className="mt-8 mb-4 scroll-m-20 text-3xl font-semibold tracking-tight text-foreground first:mt-0"
    />
  )
}

export function H2Element(props: PlateElementProps) {
  return (
    <PlateElement
      {...props}
      as="h2"
      className="mt-7 mb-3 scroll-m-20 text-2xl font-semibold tracking-tight text-foreground first:mt-0"
    />
  )
}

export function H3Element(props: PlateElementProps) {
  return (
    <PlateElement
      {...props}
      as="h3"
      className="mt-6 mb-2 scroll-m-20 text-xl font-semibold tracking-tight text-foreground first:mt-0"
    />
  )
}

export function BlockquoteElement(props: PlateElementProps) {
  return (
    <PlateElement
      {...props}
      as="blockquote"
      className="my-4 border-l-2 border-primary/60 pl-6 text-muted-foreground italic"
    />
  )
}

export function HrElement(props: PlateElementProps) {
  const selected = useSelected()

  return (
    <PlateElement {...props}>
      <div className="cursor-pointer py-6" contentEditable={false}>
        <hr
          className={cn(
            "h-0.5 rounded-full border-none bg-border bg-clip-content",
            selected && "ring-2 ring-ring ring-offset-2 ring-offset-background",
          )}
        />
      </div>
      {props.children}
    </PlateElement>
  )
}

export function CodeBlockElement(props: PlateElementProps) {
  return (
    <PlateElement {...props} className="my-4">
      <div className="overflow-hidden rounded-md border border-border bg-muted">
        <pre className="overflow-x-auto px-4 py-3 font-mono text-[13px] leading-relaxed [tab-size:2]">
          <code>{props.children}</code>
        </pre>
      </div>
    </PlateElement>
  )
}

export function CodeLineElement(props: PlateElementProps) {
  return <PlateElement {...props} />
}

export function CodeSyntaxLeaf(props: PlateLeafProps) {
  const tokenClassName =
    typeof props.leaf.className === "string" ? props.leaf.className : undefined

  return <PlateLeaf {...props} className={tokenClassName} />
}

export function CodeLeaf(props: PlateLeafProps) {
  return (
    <PlateLeaf
      {...props}
      as="code"
      className="rounded-sm bg-muted px-[0.3em] py-[0.15em] font-mono text-[0.9em] text-foreground"
    />
  )
}

export function LinkElement(props: PlateElementProps<TLinkElement>) {
  return (
    <PlateElement
      {...props}
      as="a"
      className="font-medium text-primary underline decoration-primary/40 underline-offset-4 hover:decoration-primary"
      attributes={{
        ...props.attributes,
        href: props.element.url,
        rel: "noreferrer",
        target: "_blank",
      }}
    />
  )
}

export function ImageElement(props: PlateElementProps<TImageElement>) {
  const editor = useEditorRef()
  const selected = useSelected()
  const readOnly = useReadOnly()
  const width = typeof props.element.width === "number" ? props.element.width : 100

  const setWidth = (nextWidth: number) => {
    editor.tf.setNodes({ width: nextWidth }, { at: props.path })
  }

  return (
    <PlateElement {...props} className="my-5">
      <figure className="group relative flex flex-col items-center gap-2" contentEditable={false}>
        <img
          src={props.element.url}
          alt=""
          style={{ width: `${width}%` }}
          className={cn(
            "block max-w-full rounded-md object-cover",
            selected && "ring-2 ring-ring ring-offset-2 ring-offset-background",
          )}
        />
        {!readOnly && selected ? (
          <div className="absolute top-2 right-2 flex items-center gap-1 rounded-md border border-border bg-popover/95 p-1 shadow-sm">
            {imageWidths.map((imageWidth) => (
              <button
                key={imageWidth}
                type="button"
                aria-label={`Set image width to ${imageWidth}%`}
                onClick={() => setWidth(imageWidth)}
                className={cn(
                  "rounded-sm px-2 py-1 text-[11px] text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                  width === imageWidth && "bg-accent text-accent-foreground",
                )}
              >
                {imageWidth}%
              </button>
            ))}
          </div>
        ) : null}
        <CaptionTextarea
          readOnly={readOnly}
          placeholder="Write a caption…"
          className="w-full resize-none border-none bg-transparent text-center text-xs text-muted-foreground outline-none"
        />
      </figure>
      {props.children}
    </PlateElement>
  )
}

export function TableElement(props: PlateElementProps) {
  return (
    <PlateElement {...props} className="my-5 overflow-x-auto">
      <table className="w-full table-fixed border-collapse text-sm">
        <tbody className="min-w-full">{props.children}</tbody>
      </table>
    </PlateElement>
  )
}

export function TableRowElement(props: PlateElementProps) {
  return <PlateElement {...props} as="tr" className="border-b border-border" />
}

export function TableCellElement(props: PlateElementProps) {
  return (
    <PlateElement
      {...props}
      as="td"
      className="border border-border px-3 py-2 align-top"
    />
  )
}

export function TableCellHeaderElement(props: PlateElementProps) {
  return (
    <PlateElement
      {...props}
      as="th"
      className="border border-border bg-muted px-3 py-2 text-left font-semibold"
    />
  )
}
