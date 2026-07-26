/**
 * Past dictations. Both the raw transcript and the polished text are kept, because
 * when the AI pass goes wrong the raw version is usually the one worth recovering.
 */

import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import * as api from "../lib/api";
import { useT } from "../lib/i18n";
import type { HistoryEntry } from "../lib/types";

/** Roughly the clamp height in `.log__text--clamped`; longer entries get a "read more". */
const CLAMP_CHARS = 320;

export function HistoryList() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [reading, setReading] = useState<HistoryEntry | null>(null);
  const t = useT();

  const reload = useCallback(() => {
    api.getHistory().then(setEntries).catch(() => setEntries([]));
  }, []);

  useEffect(() => {
    reload();
    const off = listen(api.EVENTS.historyChanged, reload);
    return () => {
      off.then((fn) => fn()).catch(() => undefined);
    };
  }, [reload]);

  const copy = async (id: string, text: string) => {
    await api.copyText(text);
    setCopiedId(id);
    window.setTimeout(() => setCopiedId(null), 1400);
  };

  return (
    <>
      <h1 className="view__title">{t.history.title}</h1>
      <p className="view__lede">
{t.history.lede}
      </p>

      {entries.length === 0 ? (
        <div className="empty">
          {t.history.empty}
          <br />
          {t.history.emptyHint}
        </div>
      ) : (
        <>
          <section className="section">
            <h2 className="section__title">
              {entries.length} {entries.length === 1 ? t.history.entry : t.history.entries}
            </h2>
          </section>

          <div className="log">
            {entries.map((entry) => (
              <article key={entry.id} className="log__item">
                <header className="log__head">
                  <span>{new Date(entry.created_at).toLocaleString()}</span>
                  <span className="chip">{entry.mode_name}</span>
                  <span>{entry.duration_secs.toFixed(1)}s</span>
                  <span>{entry.stt_provider}</span>
                  {entry.audio_path && <span className="chip">audio</span>}
                </header>

                <p
                  className={`log__text ${
                    entry.final_text.length > CLAMP_CHARS ? "log__text--clamped" : ""
                  }`}
                >
                  {entry.final_text}
                </p>

                {entry.final_text.length > CLAMP_CHARS && (
                  <button
                    className="btn-quiet"
                    style={{ alignSelf: "flex-start" }}
                    onClick={() => setReading(entry)}
                  >
                    {t.history.readFull} ({entry.final_text.length.toLocaleString()} {t.history.chars})
                  </button>
                )}

                {entry.raw_text !== entry.final_text && (
                  <details className="log__raw">
                    <summary>{t.history.rawSummary}</summary>
                    <p>{entry.raw_text}</p>
                    <button
                      className="btn-quiet"
                      onClick={() => copy(`${entry.id}-raw`, entry.raw_text)}
                    >
                      {copiedId === `${entry.id}-raw` ? t.common.copied : t.history.copyRaw}
                    </button>
                  </details>
                )}

                <div className="log__tools">
                  <button className="btn-quiet" onClick={() => copy(entry.id, entry.final_text)}>
                    {copiedId === entry.id ? t.common.copied : t.common.copy}
                  </button>
                  <button
                    className="btn-quiet btn-danger"
                    onClick={() => api.deleteHistoryEntry(entry.id).then(reload)}
                  >
                    {t.common.delete}
                  </button>
                </div>
              </article>
            ))}
          </div>

          <div className="row row--tight" style={{ marginTop: 18 }}>
            <span className="row__hint">
              {t.history.clearHint}
            </span>
            <div className="row__control">
              <button
                className="btn-quiet btn-danger"
                onClick={() => {
                  if (window.confirm(t.history.confirmClear)) {
                    api.clearHistory().then(reload);
                  }
                }}
              >
                {t.history.clearAll}
              </button>
            </div>
          </div>
        </>
      )}

      {reading && (
        <ReaderSheet
          entry={reading}
          onClose={() => setReading(null)}
          onCopy={(text) => copy(reading.id, text)}
          copied={copiedId === reading.id}
        />
      )}
    </>
  );
}

/** Full-text reader. Long dictations are unreadable inside a clamped list row. */
function ReaderSheet({
  entry,
  onClose,
  onCopy,
  copied,
}: {
  entry: HistoryEntry;
  onClose: () => void;
  onCopy: (text: string) => void;
  copied: boolean;
}) {
  const [showRaw, setShowRaw] = useState(false);
  const t = useT();
  const text = showRaw ? entry.raw_text : entry.final_text;

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    // Click-outside-to-close, with the panel stopping propagation.
    <div className="sheet" onClick={onClose}>
      <div className="sheet__panel" onClick={(e) => e.stopPropagation()}>
        <header className="sheet__head">
          <span>{new Date(entry.created_at).toLocaleString()}</span>
          <span className="chip">{entry.mode_name}</span>
          <span>{entry.duration_secs.toFixed(1)}s</span>
          <span>{text.length.toLocaleString()} {t.history.chars}</span>
        </header>

        <div className="sheet__body">{text}</div>

        <footer className="sheet__foot">
          {entry.raw_text !== entry.final_text && (
            <button className="btn-quiet" onClick={() => setShowRaw((v) => !v)}>
              {showRaw ? t.history.showPolished : t.history.showRaw}
            </button>
          )}
          <button onClick={() => onCopy(text)}>{copied ? t.common.copied : t.common.copy}</button>
          <button className="btn-primary" onClick={onClose}>
            {t.common.close}
          </button>
        </footer>
      </div>
    </div>
  );
}
