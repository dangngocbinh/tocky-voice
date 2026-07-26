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

export const InfoIcon = ({ className }: Props) => (
  <svg {...base} className={className} aria-hidden="true">
    <circle cx="8" cy="8" r="6.25" />
    <path d="M8 7.25v4M8 4.9h.01" />
  </svg>
);

/** Brand mark — the app icon's waveform, same forward lean, drawn as a line glyph. */
export const WaveMark = ({ className }: Props) => (
  <svg
    viewBox="0 0 22 22"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.9"
    strokeLinecap="round"
    className={className}
    aria-hidden="true"
  >
    <g transform="rotate(-9 11 11)">
      <path d="M2.6 11h0M6.2 8.2v5.6M9.8 5.1v11.8M13.4 7.4v7.2M17 9.6v2.8M20.4 11h0" />
    </g>
  </svg>
);
