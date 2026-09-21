import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import type { AppError, Settings } from '$lib/ipc/types';
import { createSettingsStore, type SettingsApi } from './settings';

const defaults: Settings = {
  version: 1,
  capacity: 20,
  hotkey: 'Alt+Shift+V',
  tabWidth: 4,
  pollIntervalMs: 200,
  autostart: false,
  language: null,
  transfer: {
    keepStyle: false,
    newline: 'keep',
    trim: false,
    tabsToSpaces: false,
    fullwidthToSpace: false,
  },
};

function fakeApi(opts: { reject?: AppError } = {}) {
  let changed: ((s: Settings) => void) | null = null;
  const calls: Settings[] = [];
  const api: SettingsApi & { calls: Settings[]; fireChanged: (s: Settings) => void } = {
    calls,
    getSettings: async () => defaults,
    setSettings: async (s: Settings) => {
      calls.push(s);
      if (opts.reject) throw opts.reject;
      return s;
    },
    onSettingsChanged: async (cb) => {
      changed = cb;
      return () => {};
    },
    fireChanged: (s) => changed?.(s),
  };
  return api;
}

describe('settings store', () => {
  it('loads settings and applies a transfer toggle optimistically, persisting it', async () => {
    const api = fakeApi();
    const store = createSettingsStore(api);
    await store.start();
    expect(get(store.settings)).toEqual(defaults);

    const p = store.setTransfer({ trim: true, newline: 'space' });
    // optimistic: visible before the backend answers
    expect(get(store.settings)?.transfer.trim).toBe(true);
    await p;
    expect(api.calls).toHaveLength(1);
    expect(api.calls[0].transfer).toEqual({ ...defaults.transfer, trim: true, newline: 'space' });
    expect(api.calls[0].capacity).toBe(20);
  });

  it('reverts the toggle and reports the error when persisting fails', async () => {
    const api = fakeApi({ reject: { kind: 'settingsIo' } });
    const store = createSettingsStore(api);
    await store.start();
    const result = await store.setTransfer({ keepStyle: true });
    expect(result).toEqual({ kind: 'settingsIo' });
    expect(get(store.settings)?.transfer.keepStyle).toBe(false);
  });

  it('follows settings-changed events from the backend', async () => {
    const api = fakeApi();
    const store = createSettingsStore(api);
    await store.start();
    api.fireChanged({ ...defaults, capacity: 7 });
    expect(get(store.settings)?.capacity).toBe(7);
  });

  it('update persists arbitrary fields and returns the applied settings', async () => {
    const api = fakeApi();
    const store = createSettingsStore(api);
    await store.start();
    const applied = await store.update({ ...defaults, capacity: 3 });
    expect(applied).toEqual({ ...defaults, capacity: 3 });
    expect(get(store.settings)?.capacity).toBe(3);
  });
});
