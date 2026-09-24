/** @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import en from '../../locales/en.json';
import type { UpdateApi, UpdateHandle } from '$lib/ipc/update';
import { i18n } from '$lib/stores/i18n';
import UpdatePrompt from './UpdatePrompt.svelte';

beforeEach(() => i18n.setLanguage('en'));

function handle(version = '9.9.9'): UpdateHandle & { installs: number } {
  const h = {
    version,
    installs: 0,
    downloadAndInstall: async () => {
      h.installs += 1;
    },
  };
  return h;
}

function api(
  update: UpdateHandle | null,
  opts: { fail?: boolean } = {},
): UpdateApi & {
  checks: number;
  relaunches: number;
} {
  const a = {
    checks: 0,
    relaunches: 0,
    check: async () => {
      a.checks += 1;
      if (opts.fail) throw new Error('offline');
      return update;
    },
    relaunch: async () => {
      a.relaunches += 1;
    },
  };
  return a;
}

describe('UpdatePrompt', () => {
  it('says nothing when the app is already up to date (13.6)', async () => {
    const a = api(null);
    render(UpdatePrompt, { api: a });
    await waitFor(() => expect(a.checks).toBe(1));
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('announces an available version on startup (13.6)', async () => {
    render(UpdatePrompt, { api: api(handle('1.2.3')) });
    const banner = await screen.findByRole('status');
    expect(banner).toHaveTextContent(en['update.available'].replace('{version}', '1.2.3'));
  });

  it('installs and offers a restart when the user accepts (13.7)', async () => {
    const h = handle();
    const a = api(h);
    render(UpdatePrompt, { api: a });
    await screen.findByRole('status');
    await fireEvent.click(screen.getByText(en['update.install']));
    await waitFor(() => expect(h.installs).toBe(1));
    const restart = await screen.findByText(en['update.restart']);
    await fireEvent.click(restart);
    await waitFor(() => expect(a.relaunches).toBe(1));
  });

  it('keeps running the current version when the user declines (13.8)', async () => {
    const h = handle();
    render(UpdatePrompt, { api: api(h) });
    await screen.findByRole('status');
    await fireEvent.click(screen.getByText(en['update.dismiss']));
    await waitFor(() => expect(screen.queryByRole('status')).toBeNull());
    expect(h.installs).toBe(0);
  });

  it('stays quiet when the check itself fails, so being offline is not an error', async () => {
    const a = api(null, { fail: true });
    render(UpdatePrompt, { api: a });
    await waitFor(() => expect(a.checks).toBe(1));
    expect(screen.queryByRole('status')).toBeNull();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('reports a failed download without dismissing the banner', async () => {
    const h = {
      version: '1.0.1',
      downloadAndInstall: vi.fn(async () => {
        throw new Error('network');
      }),
    };
    render(UpdatePrompt, { api: api(h) });
    await screen.findByRole('status');
    await fireEvent.click(screen.getByText(en['update.install']));
    expect(await screen.findByText(en['update.failed'])).toBeInTheDocument();
    expect(screen.getByText(en['update.install'])).toBeInTheDocument();
  });
});
