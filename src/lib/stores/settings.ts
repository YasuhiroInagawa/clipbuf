/**
 * Copy of the persisted settings. Transfer toggles are saved immediately (7.3) with an
 * optimistic update that is rolled back when the backend refuses.
 */

import { writable, type Readable } from 'svelte/store';
import type { UnlistenFn } from '$lib/ipc/events';
import { isAppError, type AppError, type Settings, type TransferOptions } from '$lib/ipc/types';

export interface SettingsApi {
  getSettings(): Promise<Settings>;
  setSettings(settings: Settings): Promise<Settings>;
  onSettingsChanged(cb: (settings: Settings) => void): Promise<UnlistenFn>;
}

export interface SettingsStore {
  /** `null` until `start` has loaded the settings. */
  settings: Readable<Settings | null>;
  start(): Promise<void>;
  stop(): void;
  /** Persist a full settings object (settings window). Rejects with `AppError`. */
  update(next: Settings): Promise<Settings>;
  /** Change transfer toggles optimistically; resolves to the error when persisting failed. */
  setTransfer(patch: Partial<TransferOptions>): Promise<AppError | null>;
}

export function createSettingsStore(api: SettingsApi): SettingsStore {
  const settings = writable<Settings | null>(null);
  let current: Settings | null = null;
  settings.subscribe((s) => (current = s));
  let unlisten: UnlistenFn | null = null;

  return {
    settings: { subscribe: settings.subscribe },
    async start() {
      unlisten = await api.onSettingsChanged((s) => settings.set(s));
      settings.set(await api.getSettings());
    },
    stop() {
      unlisten?.();
      unlisten = null;
    },
    async update(next) {
      const applied = await api.setSettings(next);
      settings.set(applied);
      return applied;
    },
    async setTransfer(patch) {
      if (current === null) return { kind: 'settingsIo' };
      const before = current;
      const next: Settings = { ...before, transfer: { ...before.transfer, ...patch } };
      settings.set(next);
      try {
        settings.set(await api.setSettings(next));
        return null;
      } catch (e) {
        settings.set(before);
        return isAppError(e) ? e : { kind: 'settingsIo' };
      }
    },
  };
}
