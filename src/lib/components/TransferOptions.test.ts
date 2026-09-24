/** @vitest-environment jsdom */
import { fireEvent, render, screen } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { beforeAll, describe, expect, it } from 'vitest';
import en from '../../locales/en.json';
import type { AppError, Settings } from '$lib/ipc/types';
import { i18n } from '$lib/stores/i18n';
import { createNoticeStore } from '$lib/stores/notice';
import { createSettingsStore, type SettingsApi } from '$lib/stores/settings';
import TransferOptions from './TransferOptions.svelte';

beforeAll(() => i18n.setLanguage('en'));

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

async function setup(opts: { initial?: Settings; reject?: AppError } = {}) {
  const calls: Settings[] = [];
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
  const notices = createNoticeStore();
  const utils = render(TransferOptions, { store, notices });
  return { ...utils, store, notices, calls };
}

describe('TransferOptions', () => {
  it('renders the options from the store, with formatting as a two-way radio', async () => {
    await setup();
    expect(screen.getByLabelText(en['transfer.style.keep'])).not.toBeChecked();
    expect(screen.getByLabelText(en['transfer.style.strip'])).toBeChecked();
    expect(screen.getByLabelText(en['transfer.trim'])).not.toBeChecked();
    expect(screen.getByLabelText(en['transfer.tabsToSpaces'])).not.toBeChecked();
    expect(screen.getByLabelText(en['transfer.fullwidthToSpace'])).not.toBeChecked();
    expect(screen.getByLabelText(en['transfer.newline.keep'])).toBeChecked();
    expect(screen.getByLabelText(en['transfer.newline.remove'])).not.toBeChecked();
    expect(screen.getByLabelText(en['transfer.newline.space'])).not.toBeChecked();
  });

  it('explains every option on hover (7.1.1)', async () => {
    const { container } = await setup();
    expect(container.querySelector('.radios')).toHaveAttribute('title', en['transfer.style.hint']);
    expect(screen.getByLabelText(en['transfer.trim']).closest('label')).toHaveAttribute(
      'title',
      en['transfer.trim.hint'],
    );
    expect(screen.getByLabelText(en['transfer.tabsToSpaces']).closest('label')).toHaveAttribute(
      'title',
      en['transfer.tabsToSpaces.hint'],
    );
    expect(screen.getByLabelText(en['transfer.fullwidthToSpace']).closest('label')).toHaveAttribute(
      'title',
      en['transfer.fullwidthToSpace.hint'],
    );
  });

  it('separates the option groups visually (7.1)', async () => {
    const { container } = await setup();
    expect(container.querySelectorAll('.sep').length).toBeGreaterThanOrEqual(4);
  });

  it('persists the formatting choice through the radio', async () => {
    const { calls } = await setup();
    await fireEvent.click(screen.getByLabelText(en['transfer.style.keep']));
    await Promise.resolve();
    expect(calls[0].transfer.keepStyle).toBe(true);
  });

  it('persists a checkbox change immediately through the store (7.3)', async () => {
    const { calls, store } = await setup();
    await fireEvent.click(screen.getByLabelText(en['transfer.trim']));
    await Promise.resolve();
    expect(calls).toHaveLength(1);
    expect(calls[0].transfer.trim).toBe(true);
    expect(get(store.settings)?.transfer.trim).toBe(true);
  });

  it('persists the newline mode as a radio group', async () => {
    const { calls } = await setup();
    await fireEvent.click(screen.getByLabelText(en['transfer.newline.space']));
    await Promise.resolve();
    expect(calls[0].transfer.newline).toBe('space');
    expect(screen.getByLabelText(en['transfer.newline.space'])).toBeChecked();
  });

  it('shows the keep-style hint on the other toggles without disabling them', async () => {
    const { container } = await setup({
      initial: { ...defaults, transfer: { ...defaults.transfer, keepStyle: true } },
    });
    expect(container.querySelector('.hint')).toHaveTextContent(en['transfer.keepStyle.hint']);
    expect(screen.getByLabelText(en['transfer.style.keep'])).toBeChecked();
    const trim = screen.getByLabelText(en['transfer.trim']);
    expect(trim).not.toBeDisabled();
    expect(trim.closest('label')).toHaveAttribute('title', en['transfer.keepStyle.hint']);
  });

  it('hides the hint when keep-style is off', async () => {
    const { container } = await setup();
    expect(container.querySelector('.hint')).toBeNull();
  });

  it('reverts the toggle and shows an error notice when saving fails', async () => {
    const { notices, store } = await setup({ reject: { kind: 'settingsIo' } });
    await fireEvent.click(screen.getByLabelText(en['transfer.fullwidthToSpace']));
    await Promise.resolve();
    await Promise.resolve();
    expect(get(store.settings)?.transfer.fullwidthToSpace).toBe(false);
    expect(get(notices.notice)).toMatchObject({ kind: 'error', message: 'settings.saveFailed' });
  });
});
