import type { ComponentProps } from "react"

import { cn } from "@/lib/utils"

/* The Curio mark: three arms of an isometric block meeting at one corner, drawn
   in `currentColor` so it takes the ink of whatever surface it sits on. Tops
   are left open to the background; the two visible sides are filled at
   different strengths so the form still reads as solid in one colour.
   Faces are listed back to front, so later polygons correctly overlap. */
const FACES = [
  { kind: "top", points: "4.67,2.26 12,6.5 8.33,8.61 1,4.38" },
  { kind: "front", points: "1,8.61 8.33,12.85 8.33,8.61 1,4.38" },
  { kind: "top", points: "19.33,2.26 23,4.38 15.67,8.61 12,6.5" },
  { kind: "side", points: "23,8.61 15.67,12.85 15.67,8.61 23,4.38" },
  { kind: "front", points: "12,10.73 15.67,12.85 15.67,8.61 12,6.5" },
  { kind: "top", points: "12,6.5 15.67,8.61 12,10.73 8.33,8.61" },
  { kind: "side", points: "15.67,19.62 12,21.74 12,10.73 15.67,8.61" },
  { kind: "front", points: "8.33,19.62 12,21.74 12,10.73 8.33,8.61" },
] as const

const FACE_OPACITY = { top: 0, side: 1, front: 0.55 } as const

export function CurioMark({
  className,
  title,
  ...props
}: ComponentProps<"svg"> & { title?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      stroke="currentColor"
      strokeWidth={1.1}
      strokeLinejoin="round"
      role={title ? "img" : undefined}
      aria-hidden={title ? undefined : true}
      className={cn("size-6 shrink-0", className)}
      {...props}
    >
      {title ? <title>{title}</title> : null}
      {FACES.map((face) => (
        <polygon
          key={face.points}
          points={face.points}
          fillOpacity={FACE_OPACITY[face.kind]}
        />
      ))}
    </svg>
  )
}

/* Mark plus name, set in the reading serif. Used wherever Curio introduces
   itself: the spine, the landing header, sign-in and onboarding. */
export function CurioWordmark({
  className,
  markClassName,
}: {
  className?: string
  markClassName?: string
}) {
  return (
    <span className={cn("inline-flex items-center gap-2", className)}>
      <CurioMark className={cn("size-[1.15em]", markClassName)} />
      <span className="font-display text-[1.25em] leading-none tracking-[-0.01em]">
        Curio
      </span>
    </span>
  )
}
