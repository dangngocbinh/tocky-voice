/**
 * The sidebar list shared by "Chế độ" (dictation modes) and "Đọc" → read modes — same
 * shape (name + one-line status), just a different meta string per caller.
 */

interface Item {
  id: string;
  name: string;
  meta: string;
}

interface Props {
  items: Item[];
  selectedId: string;
  onSelect: (id: string) => void;
  onAdd?: () => void;
  addLabel?: string;
}

export function ModeList({ items, selectedId, onSelect, onAdd, addLabel }: Props) {
  return (
    <aside className="modes__list">
      {items.map((item) => (
        <button
          key={item.id}
          className={`modes__item ${item.id === selectedId ? "modes__item--on" : ""}`}
          onClick={() => onSelect(item.id)}
        >
          <span className="modes__name">{item.name}</span>
          <span className="modes__meta">{item.meta}</span>
        </button>
      ))}
      {onAdd && (
        <button className="btn-quiet modes__add" onClick={onAdd}>
          {addLabel}
        </button>
      )}
    </aside>
  );
}
