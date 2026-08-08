/**
 * Renders a Tauri accelerator string using the names printed on the user's own keyboard.
 *
 * A Windows user shown "⌥⇧D" has to translate it before they can press anything, and
 * there is no ⌘ key to look for at all — so the symbol set follows the platform.
 */

import { isMac } from "./platform";

const MAC_SYMBOLS: Record<string, string> = {
  commandorcontrol: "⌘",
  cmdorctrl: "⌘",
  command: "⌘",
  cmd: "⌘",
  super: "⌘",
  meta: "⌘",
  control: "⌃",
  ctrl: "⌃",
  alt: "⌥",
  option: "⌥",
  shift: "⇧",
  space: "Space",
};

/** Spelled out, and joined with "+" below, the way Windows and Linux write shortcuts. */
const PC_SYMBOLS: Record<string, string> = {
  commandorcontrol: "Ctrl",
  cmdorctrl: "Ctrl",
  command: "Win",
  cmd: "Win",
  super: "Super",
  meta: "Win",
  control: "Ctrl",
  ctrl: "Ctrl",
  alt: "Alt",
  option: "Alt",
  shift: "Shift",
  space: "Space",
};

const SYMBOLS = isMac ? MAC_SYMBOLS : PC_SYMBOLS;

/** Physical-key names round-trip through the accelerator parser but read badly. */
function prettyKey(raw: string): string {
  const named: Record<string, string> = {
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
    Enter: "↩",
    Backspace: "⌫",
    Delete: "⌦",
    Escape: "⎋",
    Tab: "⇥",
    Minus: "-",
    Equal: "=",
    Comma: ",",
    Period: ".",
    Slash: "/",
    Backslash: "\\",
    Semicolon: ";",
    Quote: "'",
    Backquote: "`",
    BracketLeft: "[",
    BracketRight: "]",
  };
  if (named[raw]) return named[raw];
  // "KeyD" -> "D", "Digit1" -> "1", "Numpad5" -> "Num5"
  const stripped = raw.replace(/^Key/, "").replace(/^Digit/, "").replace(/^Numpad/, "Num");
  return stripped.toUpperCase();
}

export function formatAccelerator(accelerator: string | null): string | null {
  if (!accelerator) return null;
  const parts = accelerator.split("+").map((p) => p.trim());
  return parts
    .map((part, index) => {
      const symbol = SYMBOLS[part.toLowerCase()];
      if (symbol) return symbol;
      // The last segment is the key; anything earlier is an unrecognised modifier.
      return index === parts.length - 1 ? prettyKey(part) : part.toUpperCase();
    })
    .join(isMac ? "" : "+");
}

