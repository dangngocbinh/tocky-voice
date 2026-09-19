/**
 * Release notes, rendered.
 *
 * The text arrives as the Markdown a maintainer wrote in `CHANGELOG.md` — headings,
 * bullets, bold, inline code — and used to be dropped into a `<p>` as-is, so the panel
 * showed people literal `### Sửa lỗi` and `- **Linux: …**`. That is the one screen where
 * someone is deciding whether to install something, so it has to read like prose.
 *
 * This renders the small subset the changelog actually uses rather than pulling in a
 * Markdown library: the app ships with React and the Tauri bindings and nothing else,
 * and a parser for six constructs is smaller than the dependency would be.
 *
 * Everything becomes React elements — never `dangerouslySetInnerHTML`. The notes are
 * signed, but "signed" only means they came from the release pipeline, not that they
 * are safe to hand to the DOM as markup.
 */

import type { ReactNode } from "react";

/** Bold, inline code, italics — matched in one pass so the first opener wins. */
const INLINE = /(\*\*[^*]+\*\*|`[^`]+`|_[^_]+_)/g;

function renderInline(text: string, keyPrefix: string): ReactNode[] {
  return text.split(INLINE).map((part, i) => {
    const key = `${keyPrefix}-${i}`;
    if (part.startsWith("**") && part.endsWith("**")) {
      return <strong key={key}>{part.slice(2, -2)}</strong>;
    }
    if (part.startsWith("`") && part.endsWith("`")) {
      return <code key={key}>{part.slice(1, -1)}</code>;
    }
    if (part.startsWith("_") && part.endsWith("_")) {
      return <em key={key}>{part.slice(1, -1)}</em>;
    }
    return part;
  });
}

type Block =
  | { kind: "heading"; text: string }
  | { kind: "paragraph"; text: string }
  | { kind: "list"; items: string[] }
  | { kind: "rule" };

/**
 * Line-by-line, because the wrapping is what carries the meaning here: a changelog
 * bullet is hard-wrapped across several lines, and each continuation line belongs to
 * the bullet above it rather than starting a paragraph of its own.
 */
export function parseNotes(body: string): Block[] {
  const blocks: Block[] = [];
  let paragraph: string[] = [];
  let list: string[] | null = null;

  const flushParagraph = () => {
    if (paragraph.length > 0) {
      blocks.push({ kind: "paragraph", text: paragraph.join(" ") });
      paragraph = [];
    }
  };
  const flushList = () => {
    if (list) {
      blocks.push({ kind: "list", items: list });
      list = null;
    }
  };
  const flush = () => {
    flushParagraph();
    flushList();
  };

  for (const raw of body.replace(/\r\n/g, "\n").split("\n")) {
    const line = raw.trimEnd();

    if (line.trim() === "") {
      flush();
      continue;
    }
    if (/^#{1,6}\s+/.test(line)) {
      flush();
      blocks.push({ kind: "heading", text: line.replace(/^#{1,6}\s+/, "") });
      continue;
    }
    if (/^([-*_])\1{2,}$/.test(line.trim())) {
      flush();
      blocks.push({ kind: "rule" });
      continue;
    }
    if (/^\s*[-*]\s+/.test(line)) {
      flushParagraph();
      list = list ?? [];
      list.push(line.replace(/^\s*[-*]\s+/, ""));
      continue;
    }
    // A continuation of whatever is open: the rest of a wrapped bullet, or the rest
    // of a paragraph.
    if (list) list[list.length - 1] += ` ${line.trim()}`;
    else paragraph.push(line.trim());
  }

  flush();
  return blocks;
}

export function ReleaseNotes({ body }: { body: string }) {
  return (
    <div className="notes">
      {parseNotes(body).map((block, i) => {
        switch (block.kind) {
          case "heading":
            return (
              <h3 key={i} className="notes__heading">
                {renderInline(block.text, `h${i}`)}
              </h3>
            );
          case "rule":
            return <hr key={i} className="notes__rule" />;
          case "list":
            return (
              <ul key={i} className="notes__list">
                {block.items.map((item, j) => (
                  <li key={j}>{renderInline(item, `l${i}-${j}`)}</li>
                ))}
              </ul>
            );
          case "paragraph":
            return (
              <p key={i} className="notes__paragraph">
                {renderInline(block.text, `p${i}`)}
              </p>
            );
        }
      })}
    </div>
  );
}
