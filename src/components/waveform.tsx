/** Shared level meter — used at two sizes by the overlay and the Dictate view. */

interface Props {
  levels: number[];
  active: boolean;
  size?: "sm" | "md";
}

/** Floor keeps a visible sliver so silence reads as "armed", not "broken". */
const IDLE_HEIGHT = 8;

export function Waveform({ levels, active, size = "md" }: Props) {
  return (
    <div
      className={`wave ${size === "sm" ? "wave--sm" : ""} ${active ? "wave--live" : ""}`}
      aria-hidden="true"
    >
      {levels.map((level, index) => (
        <span
          key={index}
          className="wave__bar"
          style={{
            // Square root, not linear: speech peaks sit low in the 0..1 range and a
            // linear meter reads as a flat line for a normally-spoken sentence.
            height: `${
              active ? Math.max(IDLE_HEIGHT, Math.min(100, Math.sqrt(level) * 115)) : IDLE_HEIGHT
            }%`,
          }}
        />
      ))}
    </div>
  );
}
