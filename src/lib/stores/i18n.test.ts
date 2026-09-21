import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import en from '../../locales/en.json';
import ja from '../../locales/ja.json';
import { createI18n, resolveInitialLanguage } from './i18n';

describe('resolveInitialLanguage', () => {
  it('prefers the saved setting', () => {
    expect(resolveInitialLanguage('en', 'ja-JP')).toBe('en');
    expect(resolveInitialLanguage('ja', 'en-US')).toBe('ja');
  });

  it('follows the OS language when there is no setting: Japanese → ja, anything else → en', () => {
    expect(resolveInitialLanguage(null, 'ja')).toBe('ja');
    expect(resolveInitialLanguage(null, 'ja-JP')).toBe('ja');
    expect(resolveInitialLanguage(null, 'en-US')).toBe('en');
    expect(resolveInitialLanguage(null, 'de')).toBe('en');
    expect(resolveInitialLanguage(null, '')).toBe('en');
  });
});

describe('locales', () => {
  it('ja and en define exactly the same keys', () => {
    expect(Object.keys(ja).sort()).toEqual(Object.keys(en).sort());
    expect(Object.keys(ja).length).toBeGreaterThan(10);
  });

  it('cover every warning kind', () => {
    for (const w of [
      'hasStyle',
      'edgeWhitespace',
      'hasTab',
      'platformDependent',
      'controlOrBinary',
      'mixedNewlines',
      'encodingNotice',
    ]) {
      expect(en).toHaveProperty(`warning.${w}`);
      expect(en).toHaveProperty(`warning.${w}.hint`);
    }
  });
});

describe('createI18n', () => {
  it('translates in the current language and switches at runtime', () => {
    const i18n = createI18n('ja');
    expect(get(i18n.t)('app.name')).toBe('clipbuf');
    expect(get(i18n.t)('warning.hasTab')).toBe(ja['warning.hasTab']);
    i18n.setLanguage('en');
    expect(get(i18n.language)).toBe('en');
    expect(get(i18n.t)('warning.hasTab')).toBe(en['warning.hasTab']);
  });

  it('falls back to the key for unknown ids', () => {
    const i18n = createI18n('en');
    expect(get(i18n.t)('no.such.key')).toBe('no.such.key');
  });
});
