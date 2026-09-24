/** @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import en from '../../locales/en.json';
import type { AppError, PlatformInfo, Settings } from '$lib/ipc/types';
import { i18n } from '$lib/stores/i18n';
import { createSettingsStore, type SettingsApi } from '$lib/stores/settings';
import SettingsWindow from './SettingsWindow.svelte';

// Saving a language change switches the UI immediately, so reset before each test.
beforeEach(() => i18n.setLanguage('en'));

const defaults: Settings = {
  version: 1,
  capacity: 20,
  hotkey: 'Alt+Shift+KeyV',
  tabWidth: 4,
  pollIntervalMs: 200,
  autostart: false,
  language: null,
  previewWrap: true,
  transfer: {
    keepStyle: false,
    newline: 'keep',
    trim: false,
    tabsToSpaces: false,
    fullwidthToSpace: false,
  },
};

const MAC: PlatformInfo = { os: 'macos', displayServer: 'quartz', capture: 'full' };
const WINDOWS: PlatformInfo = { os: 'windows', displayServer: 'win32', capture: 'full' };
const WAYLAND: PlatformInfo = { os: 'linux', displayServer: 'wayland', capture: 'full' };

async function setup(
  opts: { platform?: PlatformInfo; reject?: AppError; initial?: Settings } = {},
) {
  const calls: Settings[] = [];
  const closed: number[] = [];
  const hotkeyState: string[] = [];
  const api: SettingsApi = {
    getSettings: async () => opts.initial ?? defaults,
    setSettings: async (s) => {
      calls.push(s);
      if (opts.reject) throw opts.reject;
      return s;
    },
    onSettingsChanged: async () => () => {},
  };
  const store = createSettingsStore(api);
  await store.start();
  const utils = render(SettingsWindow, {
    store,
    getPlatformInfo: async () => opts.platform ?? MAC,
    closeWindow: async () => {
      closed.push(1);
    },
    suspendHotkey: async () => {
      hotkeyState.push('suspend');
    },
    resumeHotkey: async () => {
      hotkeyState.push('resume');
    },
  });
  await screen.findByLabelText(en['settings.capacity']);
  return { ...utils, store, calls, closed, hotkeyState };
}

const save = () => fireEvent.click(screen.getByText(en['settings.save']));

describe('SettingsWindow — fields', () => {
  it('shows the persisted settings and no transfer options (9.1, 9.6)', async () => {
    await setup();
    expect(screen.getByLabelText(en['settings.capacity'])).toHaveValue(20);
    expect(screen.getByLabelText(en['settings.tabWidth'])).toHaveValue(4);
    expect(screen.getByLabelText(en['settings.autostart'])).not.toBeChecked();
    expect(screen.getByLabelText(en['settings.language'])).toHaveValue('system');
    expect(screen.getByLabelText(en['settings.hotkey'])).toHaveTextContent('Alt+Shift+KeyV');
    for (const key of ['transfer.trim', 'transfer.tabsToSpaces', 'transfer.keepStyle'] as const) {
      expect(screen.queryByLabelText(en[key])).toBeNull();
    }
  });

  it('offers the preview wrap toggle (9.1)', async () => {
    const { calls } = await setup();
    const toggle = screen.getByLabelText(en['settings.previewWrap']);
    expect(toggle).toBeChecked();
    await fireEvent.click(toggle);
    await save();
    await waitFor(() => expect(calls[0].previewWrap).toBe(false));
  });

  it('offers the poll interval on macOS, where the clipboard is polled', async () => {
    await setup({ platform: MAC });
    expect(await screen.findByLabelText(en['settings.pollIntervalMs'])).toBeInTheDocument();
  });

  it('offers the poll interval on Wayland, where the clipboard is polled', async () => {
    await setup({ platform: WAYLAND });
    expect(await screen.findByLabelText(en['settings.pollIntervalMs'])).toBeInTheDocument();
  });

  it('hides the poll interval on Windows and X11', async () => {
    await setup({ platform: WINDOWS });
    expect(screen.queryByLabelText(en['settings.pollIntervalMs'])).toBeNull();
  });
});

describe('SettingsWindow — saving', () => {
  it('persists the edited values (9.1)', async () => {
    const { calls } = await setup();
    await fireEvent.input(screen.getByLabelText(en['settings.capacity']), {
      target: { value: '50' },
    });
    await fireEvent.click(screen.getByLabelText(en['settings.autostart']));
    await fireEvent.change(screen.getByLabelText(en['settings.language']), {
      target: { value: 'ja' },
    });
    await save();
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toMatchObject({ capacity: 50, autostart: true, language: 'ja' });
    expect(calls[0].transfer).toEqual(defaults.transfer);
  });

  it('closes the window once the settings are stored (9.7)', async () => {
    const { closed } = await setup();
    await save();
    await waitFor(() => expect(closed).toHaveLength(1));
  });

  it('stays open when saving fails, so the input can be corrected', async () => {
    const { closed } = await setup({ reject: { kind: 'hotkeyUnavailable' } });
    await save();
    await screen.findByTestId('error-hotkey');
    expect(closed).toHaveLength(0);
  });

  it('rejects out-of-range values per field without calling the backend', async () => {
    const { calls } = await setup();
    await fireEvent.input(screen.getByLabelText(en['settings.capacity']), {
      target: { value: '999' },
    });
    await save();
    const error = await screen.findByTestId('error-capacity');
    expect(error).toHaveTextContent(en['settings.invalid']);
    expect(calls).toHaveLength(0);
    // The typed value stays so it can be corrected.
    expect(screen.getByLabelText(en['settings.capacity'])).toHaveValue(999);
  });

  it('shows a hotkey conflict on the hotkey field and keeps the other edits (9.4)', async () => {
    const { calls } = await setup({ reject: { kind: 'hotkeyUnavailable' } });
    await fireEvent.input(screen.getByLabelText(en['settings.tabWidth']), {
      target: { value: '8' },
    });
    await save();
    const error = await screen.findByTestId('error-hotkey');
    expect(error).toHaveTextContent(en['settings.hotkey.unavailable']);
    expect(calls).toHaveLength(1);
    expect(screen.getByLabelText(en['settings.tabWidth'])).toHaveValue(8);
  });

  it('reports other failures without blaming a field', async () => {
    await setup({ reject: { kind: 'settingsIo' } });
    await save();
    expect(await screen.findByRole('alert')).toHaveTextContent(en['settings.saveFailed']);
    expect(screen.queryByTestId('error-capacity')).toBeNull();
  });
});

describe('SettingsWindow — hotkey recorder', () => {
  it('records a pressed combination (9.1)', async () => {
    const { calls } = await setup();
    const recorder = screen.getByLabelText(en['settings.hotkey']);
    await fireEvent.click(recorder);
    expect(recorder).toHaveTextContent(en['settings.hotkey.record']);
    await fireEvent.keyDown(window, { code: 'KeyK', key: 'k', ctrlKey: true, shiftKey: true });
    expect(recorder).toHaveTextContent('Control+Shift+KeyK');
    await save();
    await waitFor(() => expect(calls[0].hotkey).toBe('Control+Shift+KeyK'));
  });

  it('records keys that reach the window, since clicking a button does not focus it on macOS', async () => {
    const { calls } = await setup();
    await fireEvent.click(screen.getByLabelText(en['settings.hotkey']));
    await fireEvent.keyDown(window, { code: 'KeyJ', key: 'j', metaKey: true, altKey: true });
    expect(screen.getByLabelText(en['settings.hotkey'])).toHaveTextContent('Command+Alt+KeyJ');
    await save();
    await waitFor(() => expect(calls[0].hotkey).toBe('Command+Alt+KeyJ'));
  });

  it('silences the active hotkey while recording and restores it afterwards (9.5.1)', async () => {
    const { hotkeyState } = await setup();
    await fireEvent.click(screen.getByLabelText(en['settings.hotkey']));
    expect(hotkeyState).toEqual(['suspend']);
    await fireEvent.keyDown(window, { code: 'KeyJ', key: 'j', metaKey: true });
    expect(hotkeyState).toEqual(['suspend', 'resume']);
  });

  it('restores the hotkey when recording is abandoned with Escape', async () => {
    const { hotkeyState } = await setup();
    await fireEvent.click(screen.getByLabelText(en['settings.hotkey']));
    await fireEvent.keyDown(window, { code: 'Escape', key: 'Escape' });
    expect(hotkeyState).toEqual(['suspend', 'resume']);
  });

  it('ignores keys when not recording', async () => {
    await setup();
    await fireEvent.keyDown(window, { code: 'KeyJ', key: 'j', metaKey: true });
    expect(screen.getByLabelText(en['settings.hotkey'])).toHaveTextContent('Alt+Shift+KeyV');
  });

  it('ignores a combination without modifiers, which could not be global', async () => {
    await setup();
    const recorder = screen.getByLabelText(en['settings.hotkey']);
    await fireEvent.click(recorder);
    await fireEvent.keyDown(window, { code: 'KeyK', key: 'k' });
    expect(recorder).toHaveTextContent(en['settings.hotkey.record']);
    expect(await screen.findByTestId('error-hotkey')).toHaveTextContent(
      en['settings.hotkey.needsModifier'],
    );
  });

  it('abandons recording on Escape and keeps the previous hotkey', async () => {
    await setup();
    const recorder = screen.getByLabelText(en['settings.hotkey']);
    await fireEvent.click(recorder);
    await fireEvent.keyDown(window, { code: 'Escape', key: 'Escape' });
    expect(recorder).toHaveTextContent('Alt+Shift+KeyV');
  });
});

describe('SettingsWindow — language', () => {
  it('applies the chosen language to the UI right away (12.4)', async () => {
    await setup();
    await fireEvent.change(screen.getByLabelText(en['settings.language']), {
      target: { value: 'ja' },
    });
    await save();
    await waitFor(() => expect(get(i18n.language)).toBe('ja'));
  });
});
