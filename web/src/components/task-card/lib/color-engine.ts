import type { TaskItem } from "../types";
import { clamp01, parseIso } from "./time";

/** Returns elapsed in [0, 1] or null when the engine is inactive for this item. */
export function computeElapsed(item: TaskItem, now: Date): number | null {
  const start = parseIso(item.startAt ?? item.setAt);
  if (!start) return null;

  if (item.expireAt) {
    const end = parseIso(item.expireAt);
    if (!end) return null;
    if (end.getTime() <= start.getTime()) return null;
    return clamp01((now.getTime() - start.getTime()) / (end.getTime() - start.getTime()));
  }

  if (item.duration && item.duration > 0) {
    return clamp01((now.getTime() - start.getTime()) / item.duration);
  }

  return null;
}

/**
 * Border color resolution.
 * Returns null when the renderer should fall back to the semantic-token
 * className (border-border) — i.e., no inline style.borderColor.
 */
export function resolveBorderColor(
  item: TaskItem,
  elapsed: number | null,
  ramp: (t: number) => string,
): string | null {
  if (item.borderColor != null && item.borderColor.length > 0) {
    return item.borderColor;
  }
  if (elapsed == null) return null;
  return ramp(elapsed);
}
