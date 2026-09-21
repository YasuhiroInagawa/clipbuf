/** @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { beforeAll, describe, expect, it } from 'vitest';
import en from '../../locales/en.json';
import type {
  AppError,
  CaptureStatus,
  ItemDto,
  PlatformInfo,
  Settings,
  TransferMode,
  TransferOutcome,
} from '$lib/ipc/types';
import { createMainContext, type MainApi, type MainContext } from '$lib/stores/context';
import { i18n } from '$lib/stores/i18n';
import MainWindow from './MainWindow.svelte';

beforeAll(() => i18n.setLanguage('en'));

const settings: Settings = {
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

const dto = (id: number, text: string, warnings: ItemDto['warnings'] = []): ItemDto => ({
  id,
  capturedAtMs: id,
  text,
  hasStyle: warnings.includes('hasStyle'),
  warnings,
});

interface Fake {
  ctx: MainContext;
  transfers: [number, TransferMode][];
  removed: number[];
  hidden: number;
  cleared: number;
  fireCaptureStatus: (s: CaptureStatus) => void;
  fireWindowShown: () => void;
}

function fake(
  opts: {
    items?: ItemDto[];
    outcome?: TransferOutcome;
    reject?: AppError;
    platform?: PlatformInfo;
  } = {},
): Fake {
  const items = opts.items ?? [dto(3, 'third\tx', ['hasTab']), dto(2, 'second'), dto(1, 'first')];
  const transfers: [number, TransferMode][] = [];
  const removed: number[] = [];
  let captureCb: ((s: CaptureStatus) => void) | null = null;
  let shownCb: (() => void) | null = null;
  const state = { hidden: 0, cleared: 0 };
  const api: MainApi = {
    listItems: async () => items,
    onItemAdded: async () => () => {},
    onItemsChanged: async () => () => {},
    getSettings: async () => settings,
    setSettings: async (s) => s,
    onSettingsChanged: async () => () => {},
    transferItem: async (id, mode) => {
      transfers.push([id, mode]);
      if (opts.reject) throw opts.reject;
      return opts.outcome ?? { skippedTransforms: false };
    },
    removeItem: async (id) => {
      removed.push(id);
    },
    clearItems: async () => {
      state.cleared += 1;
    },
    hideWindow: async () => {
      state.hidden += 1;
    },
    getPlatformInfo: async () =>
      opts.platform ?? { os: 'macos', displayServer: 'quartz', capture: 'full' },
    onCaptureStatus: async (cb) => {
      captureCb = cb;
      return () => {};
    },
    onWindowShown: async (cb) => {
      shownCb = cb;
      return () => {};
    },
  };
  const ctx = createMainContext(api);
  return {
    ctx,
    transfers,
    removed,
    get hidden() {
      return state.hidden;
    },
    get cleared() {
      return state.cleared;
    },
    fireCaptureStatus: (s) => captureCb?.(s),
    fireWindowShown: () => shownCb?.(),
  };
}

async function mount(f: Fake) {
  const utils = render(MainWindow, { ctx: f.ctx });
  await waitFor(() => expect(utils.container.querySelectorAll('.item')).toHaveLength(3));
  return utils;
}

describe('MainWindow — list', () => {
  it('renders one row per item, newest first, with preview and warning icons (3.1)', async () => {
    const f = fake();
    const { container } = await mount(f);
    const rows = [...container.querySelectorAll('.item')];
    expect(rows[0].querySelector('.preview')).toHaveTextContent('third→x');
    expect(rows[0].querySelector('[data-warning="hasTab"]')).not.toBeNull();
    expect(rows[2].querySelector('.preview')).toHaveTextContent('first');
    expect(rows[0]).toHaveClass('selected');
  });

  it('shows the empty message when there are no items', async () => {
    const f = fake({ items: [] });
    render(MainWindow, { ctx: f.ctx });
    expect(await screen.findByText(en['list.empty'])).toBeInTheDocument();
  });
});

describe('MainWindow — transfer', () => {
  it('clicking a row transfers with options and highlights it briefly (6.1, 6.7)', async () => {
    const f = fake();
    const { container } = await mount(f);
    const row = container.querySelectorAll('.item')[1];
    await fireEvent.click(row.querySelector('.preview')!);
    await waitFor(() => expect(f.transfers).toEqual([[2, 'options']]));
    await waitFor(() => expect(row).toHaveClass('highlight'));
    // list order and selection are unchanged (6.8)
    const rows = [...container.querySelectorAll('.item .preview')].map((el) => el.textContent);
    expect(rows[0]).toContain('third');
    expect(get(f.ctx.selection.selectedId)).toBe(2);
  });

  it('hover buttons transfer plain / raw without triggering the row click (6.4, 6.5, 6.6)', async () => {
    const f = fake();
    const { container } = await mount(f);
    const row = container.querySelectorAll('.item')[0];
    await fireEvent.click(row.querySelector('button[data-mode="plain"]')!);
    await fireEvent.click(row.querySelector('button[data-mode="raw"]')!);
    await waitFor(() =>
      expect(f.transfers).toEqual([
        [3, 'plain'],
        [3, 'raw'],
      ]),
    );
  });

  it('reports skipped transforms as an info notice (7.5)', async () => {
    const f = fake({ outcome: { skippedTransforms: true } });
    const { container } = await mount(f);
    await fireEvent.click(container.querySelector('.item .preview')!);
    expect(await screen.findByRole('status')).toHaveTextContent(en['notice.transformsSkipped']);
  });

  it('reports a write failure as an error notice (6.10)', async () => {
    const f = fake({ reject: { kind: 'writeFailed' } });
    const { container } = await mount(f);
    await fireEvent.click(container.querySelector('.item .preview')!);
    expect(await screen.findByRole('alert')).toHaveTextContent(en['notice.writeFailed']);
  });
});

describe('MainWindow — keyboard', () => {
  it('moves the selection with arrows and transfers with Enter / Shift+Enter (6.2, 6.3)', async () => {
    const f = fake();
    await mount(f);
    await fireEvent.keyDown(window, { key: 'ArrowDown' });
    expect(get(f.ctx.selection.selectedId)).toBe(2);
    await fireEvent.keyDown(window, { key: 'ArrowDown' });
    await fireEvent.keyDown(window, { key: 'ArrowUp' });
    expect(get(f.ctx.selection.selectedId)).toBe(2);
    await fireEvent.keyDown(window, { key: 'Enter' });
    await fireEvent.keyDown(window, { key: 'Enter', shiftKey: true });
    await waitFor(() =>
      expect(f.transfers).toEqual([
        [2, 'options'],
        [2, 'plain'],
      ]),
    );
  });

  it('Delete removes the selected item, Escape hides the window', async () => {
    const f = fake();
    await mount(f);
    await fireEvent.keyDown(window, { key: 'Delete' });
    await waitFor(() => expect(f.removed).toEqual([3]));
    await fireEvent.keyDown(window, { key: 'Escape' });
    await waitFor(() => expect(f.hidden).toBe(1));
  });

  it('ignores Enter while a form control has focus', async () => {
    const f = fake();
    await mount(f);
    const checkbox = screen.getByLabelText(en['transfer.trim']);
    checkbox.focus();
    await fireEvent.keyDown(checkbox, { key: 'Enter' });
    await new Promise((r) => setTimeout(r, 10));
    expect(f.transfers).toEqual([]);
  });
});

describe('MainWindow — capture status and window-shown', () => {
  it('shows a persistent banner when capture is denied or limited (11.5, 11.6)', async () => {
    const f = fake({ platform: { os: 'macos', displayServer: 'quartz', capture: 'denied' } });
    await mount(f);
    expect(await screen.findByRole('note')).toHaveTextContent(en['capture.denied']);
    f.fireCaptureStatus('limitedXWayland');
    expect(await screen.findByRole('note')).toHaveTextContent(en['capture.limitedXWayland']);
    f.fireCaptureStatus('full');
    await waitFor(() => expect(screen.queryByRole('note')).toBeNull());
  });

  it('reports read failures as an error notice', async () => {
    const f = fake();
    await mount(f);
    f.fireCaptureStatus('readFailed');
    expect(await screen.findByRole('alert')).toHaveTextContent(en['notice.readFailed']);
  });

  it('selects the newest item when the window is shown (8.3)', async () => {
    const f = fake();
    await mount(f);
    f.ctx.selection.select(1);
    f.fireWindowShown();
    expect(get(f.ctx.selection.selectedId)).toBe(3);
  });

  it('clear-all asks the backend to clear', async () => {
    const f = fake();
    await mount(f);
    await fireEvent.click(screen.getByText(en['list.clear']));
    await waitFor(() => expect(f.cleared).toBe(1));
  });
});
