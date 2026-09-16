/**
 * Hand-drawn on a 16px grid rather than pulled from an icon package — six glyphs do
 * not justify a dependency, and drawing them keeps the stroke weight matched to the
 * type rather than fighting it.
 */

type Props = { className?: string };

const base = {
  viewBox: "0 0 16 16",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.4,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
};

export const MicIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <rect x="5.75" y="1.75" width="4.5" height="8" rx="2.25" />
    <path d="M3.25 7.25v.5a4.75 4.75 0 0 0 9.5 0v-.5M8 12.5v1.75" />
  </svg>
);

export const ModesIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <path d="M2.25 4.25h11.5M2.25 8h7.5M2.25 11.75h4.5" />
    <circle cx="12.25" cy="11.75" r="1.6" />
  </svg>
);

export const KeyIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <rect x="1.75" y="3.25" width="12.5" height="9.5" rx="1.8" />
    <path d="M4.5 6.25h.01M7 6.25h.01M9.5 6.25h.01M11.5 6.25h.01M5 9.5h6" />
  </svg>
);

export const PlugIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <path d="M6 1.75v3.5M10 1.75v3.5" />
    <path d="M3.75 5.25h8.5v2.5a4.25 4.25 0 0 1-8.5 0v-2.5ZM8 12v2.25" />
  </svg>
);

export const LogIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <circle cx="8" cy="8" r="6.25" />
    <path d="M8 4.5V8l2.5 1.5" />
  </svg>
);

export const SpeakerIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <path d="M2.25 6.25h2.5L8.5 3v10L4.75 9.75h-2.5v-3.5Z" strokeLinejoin="round" />
    <path d="M11 5.5a4 4 0 0 1 0 5M13 3.75a7 7 0 0 1 0 8.5" />
  </svg>
);

export const InfoIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <circle cx="8" cy="8" r="6.25" />
    <path d="M8 7.25v4M8 4.9h.01" />
  </svg>
);

/**
 * Brand mark — the app icon at sidebar size, drawn as a line glyph.
 *
 * Redrawn rather than scaled: at 22px the icon's six bars a side close into a block and
 * its cradle thins to nothing, so the waveform is cut to two bars a side and each part
 * carries its own stroke weight. The capsule is a stroke too, not a filled shape, so the
 * whole mark stays one `currentColor` line drawing like the rest of the icon set.
 */
export const CopyIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <rect x="5.75" y="5.75" width="8.5" height="8.5" rx="2" />
    <path d="M10.25 3.25a1.5 1.5 0 0 0-1.5-1.5h-5a1.5 1.5 0 0 0-1.5 1.5v5a1.5 1.5 0 0 0 1.5 1.5" />
  </svg>
);

export const CheckIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <path d="M3 8.5 6.5 12 13 4.5" />
  </svg>
);

/**
 * The welcome screen's picker glyphs.
 *
 * The flags are drawn here like every other icon rather than written as emoji: a flag
 * emoji renders as two boxed letters ("VN") on Windows, which is exactly the platform
 * where someone is most likely to be picking a language they can actually read.
 *
 * They carry real national colours instead of `currentColor` — a flag in the interface
 * grey would not be a flag — and keep the 10:7 proportion every other flag has.
 */
const flagBase = { viewBox: "0 0 20 14", width: 20, height: 14 };

export const VietnamFlag = ({ className }: Props) => (
  <svg {...flagBase} className={className} aria-hidden="true">
    <rect width="20" height="14" rx="2" fill="#DA251D" />
    <path
      d="M10.00 2.60 L11.03 5.58 L14.18 5.64 L11.66 7.54 L12.59 10.56 L10.00 8.75 L7.41 10.56 L8.34 7.54 L5.82 5.64 L8.97 5.58 Z"
      fill="#FFFF00"
    />
  </svg>
);

export const UnionFlag = ({ className }: Props) => (
  <svg {...flagBase} className={className} aria-hidden="true">
    {/* Clipped to the rounded rectangle, so the diagonals stop at the flag's edge
        instead of running across the button. */}
    <defs>
      <clipPath id="union-flag-clip">
        <rect width="20" height="14" rx="2" />
      </clipPath>
    </defs>
    <g clipPath="url(#union-flag-clip)">
      <rect width="20" height="14" fill="#012169" />
      <path d="M0 0 20 14 M20 0 0 14" stroke="#FFFFFF" strokeWidth="3" />
      <path d="M0 0 20 14 M20 0 0 14" stroke="#C8102E" strokeWidth="1.3" />
      <path d="M10 0v14M0 7h20" stroke="#FFFFFF" strokeWidth="4.4" />
      <path d="M10 0v14M0 7h20" stroke="#C8102E" strokeWidth="2.4" />
    </g>
  </svg>
);

export const SunIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <circle cx="8" cy="8" r="3.1" />
    <path d="M8 1.5v1.4M8 13.1v1.4M1.5 8h1.4M13.1 8h1.4M3.4 3.4l1 1M11.6 11.6l1 1M12.6 3.4l-1 1M4.4 11.6l-1 1" />
  </svg>
);

export const MoonIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    {/* One arc, not a circle with a bite taken out: a crescent drawn as a stroke keeps
        the same weight as every other glyph here. */}
    <path d="M13 9.6A5.9 5.9 0 0 1 6.4 3a5.9 5.9 0 1 0 6.6 6.6Z" />
  </svg>
);

/** "Whatever the computer is set to" — used for both follow-system choices. */
export const MonitorIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <rect x="1.75" y="2.75" width="12.5" height="8.5" rx="1.5" />
    <path d="M6 14h4M8 11.25V14" />
  </svg>
);

export const WaveMark = ({ className }: Props) => (
  <svg
    viewBox="0 0 22 22"
    fill="none"
    stroke="currentColor"
    strokeLinecap="round"
    className={className}
    aria-hidden="true"
  >
    <g strokeWidth="1.5">
      <path d="M2.2 9.4v3.2" />
      <path d="M5.4 7.2v7.6" />
      <path d="M16.6 7.2v7.6" />
      <path d="M19.8 9.4v3.2" />
    </g>
    {/* Capsule head: a bar thick enough to read as a body rather than another wave. */}
    <path d="M11 4.5v5.4" strokeWidth="3.4" />
    <path d="M8 10.3a3 3 0 0 0 6 0" strokeWidth="1.2" />
    <path d="M11 13.3v2.9" strokeWidth="1.2" />
    <path d="M8.8 17.2h4.4" strokeWidth="1.3" />
  </svg>
);
