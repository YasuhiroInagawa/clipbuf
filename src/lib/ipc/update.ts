/**
 * Update checking (13.6–13.8). Wrapped like the other IPC so the component can be tested
 * without Tauri, and so `@tauri-apps/plugin-*` imports stay inside `lib/ipc`.
 *
 * This is the only network access the app performs (10.3, 10.4).
 */

import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';

/** The parts of a pending update the prompt needs. */
export interface UpdateHandle {
  version: string;
  downloadAndInstall(): Promise<void>;
}

export interface UpdateApi {
  /** Resolves to the pending update, or `null` when the app is current. */
  check(): Promise<UpdateHandle | null>;
  relaunch(): Promise<void>;
}

export const updateApi: UpdateApi = {
  check: async () => {
    const update = await check();
    return update
      ? { version: update.version, downloadAndInstall: () => update.downloadAndInstall() }
      : null;
  },
  relaunch: () => relaunch(),
};
