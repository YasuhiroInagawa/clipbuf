/**
 * Minimal i18n (12.1–12.4): flat JSON resources per language, a `t` store that re-renders
 * on language change. Two languages and a few dozen strings do not warrant a library.
 */

import { derived, writable, type Readable } from 'svelte/store';
import type { Language } from '$lib/ipc/types';
import en from '../../locales/en.json';
import ja from '../../locales/ja.json';

export type MessageKey = keyof typeof en;

const resources: Record<Language, Record<string, string>> = { en, ja };

/** Saved setting wins; otherwise Japanese OS → ja, anything else → en (12.2, 12.3). */
export function resolveInitialLanguage(
  setting: Language | null,
  navigatorLanguage: string,
): Language {
  if (setting) return setting;
  return navigatorLanguage.toLowerCase().startsWith('ja') ? 'ja' : 'en';
}

export type Translate = (key: MessageKey | string) => string;

export interface I18n {
  language: Readable<Language>;
  t: Readable<Translate>;
  setLanguage(language: Language): void;
}

export function createI18n(initial: Language): I18n {
  const language = writable<Language>(initial);
  const t = derived(language, (lang): Translate => {
    const table = resources[lang];
    return (key) => table[key] ?? resources.en[key] ?? key;
  });
  return {
    language: { subscribe: language.subscribe },
    t,
    setLanguage: (l) => language.set(l),
  };
}

/** App-wide instance. The settings store switches it when the saved language changes. */
export const i18n: I18n = createI18n(
  resolveInitialLanguage(null, typeof navigator === 'undefined' ? '' : navigator.language),
);
/** Convenience: `$t('key')` in components. */
export const t = i18n.t;
