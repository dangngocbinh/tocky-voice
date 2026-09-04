/** The labeled prompt textarea shared by "Chế độ" and "Đọc" → read modes. */

interface Props {
  label: string;
  hint: string;
  value: string;
  disabled?: boolean;
  onChange: (value: string) => void;
  rows?: number;
}

export function PromptField({ label, hint, value, disabled, onChange, rows = 13 }: Props) {
  return (
    <div className="row row--stack">
      <div className="row__label">
        {label}
        <span className="row__hint">{hint}</span>
      </div>
      <textarea rows={rows} disabled={disabled} value={value} onChange={(e) => onChange(e.target.value)} />
    </div>
  );
}
