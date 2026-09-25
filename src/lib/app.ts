/** Application wiring: real IPC into the stores. Components never import this; App does. */
import { commands } from '$lib/ipc/commands';
import { events } from '$lib/ipc/events';
import { createMainContext, type MainContext } from '$lib/stores/context';
import { i18n, resolveInitialLanguage } from '$lib/stores/i18n';
import { notices } from '$lib/stores/notice';
import { createSettingsStore, type SettingsStore } from '$lib/stores/settings';

export const appName = 'clipbuf';

export function createAppContext(): MainContext {
  return createMainContext({ ...commands, ...events }, notices);
}

/** Settings window: only the settings store and the platform query are needed. */
export function createSettingsContext(): {
  store: SettingsStore;
  getPlatformInfo: typeof commands.getPlatformInfo;
  closeWindow: typeof commands.closeSettings;
  suspendHotkey: typeof commands.suspendHotkey;
  resumeHotkey: typeof commands.resumeHotkey;
} {
  return {
    store: createSettingsStore({ ...commands, ...events }),
    getPlatformInfo: commands.getPlatformInfo,
    closeWindow: commands.closeSettings,
    suspendHotkey: commands.suspendHotkey,
    resumeHotkey: commands.resumeHotkey,
  };
}

/**
 * A settings store for a window that has no settings UI of its own (the about window): it
 * exists only so that window follows the saved language like the others (12.4).
 */
export function createLanguageOnlyStore(): SettingsStore {
  return createSettingsStore({ ...commands, ...events });
}

/**
 * Keep the UI language in step with the saved setting (12.4). Both windows call this, so a
 * change saved in the settings window reaches the main window through `settings-changed`.
 */
export function syncLanguage(store: SettingsStore): () => void {
  return store.settings.subscribe((settings) => {
    if (settings) i18n.setLanguage(resolveInitialLanguage(settings.language, navigator.language));
  });
}
