/* Marks every case-insensitive occurrence of `query` inside `text`, so a search
   result shows why it matched. */
export function HighlightMatch({ text, query }: { text: string; query: string }) {
  const needle = query.trim();
  if (!needle) {
    return text;
  }

  const lowerText = text.toLowerCase();
  const lowerNeedle = needle.toLowerCase();
  const parts: React.ReactNode[] = [];
  let cursor = 0;
  let index = lowerText.indexOf(lowerNeedle);

  while (index !== -1) {
    if (index > cursor) {
      parts.push(text.slice(cursor, index));
    }
    parts.push(
      <mark
        key={index}
        className="rounded-[2px] bg-accent-subtle px-px text-inherit"
      >
        {text.slice(index, index + needle.length)}
      </mark>,
    );
    cursor = index + needle.length;
    index = lowerText.indexOf(lowerNeedle, cursor);
  }

  parts.push(text.slice(cursor));
  return parts;
}
