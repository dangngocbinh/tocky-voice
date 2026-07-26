/**
 * Language codes offered in the UI.
 *
 * Typing a BCP-47 tag from memory is a bad ask — nobody recalls whether Chinese is
 * `zh`, `cmn` or `zh-CN`. The list is deliberately short: the languages the supported
 * speech providers actually handle well, with an escape hatch for anything else.
 */

export interface Language {
  code: string;
  label: string;
}

export const LANGUAGES: Language[] = [
  { code: "vi", label: "Tiếng Việt" },
  { code: "en", label: "English" },
  { code: "ja", label: "日本語 — Japanese" },
  { code: "ko", label: "한국어 — Korean" },
  { code: "zh", label: "中文 — Chinese" },
  { code: "th", label: "ไทย — Thai" },
  { code: "id", label: "Bahasa Indonesia" },
  { code: "fr", label: "Français" },
  { code: "de", label: "Deutsch" },
  { code: "es", label: "Español" },
  { code: "pt", label: "Português" },
  { code: "it", label: "Italiano" },
  { code: "ru", label: "Русский" },
  { code: "hi", label: "हिन्दी — Hindi" },
  { code: "ar", label: "العربية — Arabic" },
  { code: "nl", label: "Nederlands" },
];

export function languageLabel(code: string): string {
  return LANGUAGES.find((l) => l.code === code)?.label ?? code;
}
