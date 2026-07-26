/** Turns a backend failure into the sentence for the user's language. */

import type { ErrorPayload } from "./types";

type Dictionary = { errors: Record<string, string> };

export function formatError(
  error: ErrorPayload | null,
  t: Dictionary,
): string | null {
  if (!error) return null;

  // An unknown kind means the backend is newer than this UI. Showing the raw detail
  // beats showing nothing, and beats showing a blank where a reason should be.
  const sentence = t.errors[error.kind] ?? error.kind;
  return error.detail ? `${sentence} (${error.detail})` : sentence;
}
