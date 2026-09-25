/** @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it } from 'vitest';
import en from '../../locales/en.json';
import ja from '../../locales/ja.json';
import { AUTHOR, REPOSITORY_URL, type AboutApi } from '$lib/ipc/about';
import { i18n } from '$lib/stores/i18n';
import AboutWindow from './AboutWindow.svelte';

beforeEach(() => i18n.setLanguage('en'));

function api(opts: { version?: string; failVersion?: boolean } = {}): AboutApi & {
  opened: number;
  closed: number;
} {
  const a = {
    opened: 0,
    closed: 0,
    getVersion: async () => {
      if (opts.failVersion) throw new Error('no backend');
      return opts.version ?? '1.2.3';
    },
    openRepository: async () => {
      a.opened += 1;
    },
    close: async () => {
      a.closed += 1;
    },
  };
  return a;
}

describe('AboutWindow', () => {
  it('shows the running version (8.8)', async () => {
    render(AboutWindow, { api: api({ version: '0.4.2' }) });
    expect(await screen.findByTestId('version')).toHaveTextContent('0.4.2');
  });

  it('names the author, the licence and the repository (8.8, 13.1)', async () => {
    render(AboutWindow, { api: api() });
    expect(screen.getByText(AUTHOR)).toBeInTheDocument();
    expect(screen.getByText(en['about.license.value'])).toBeInTheDocument();
    expect(screen.getByText(REPOSITORY_URL)).toBeInTheDocument();
  });

  it('points at the issues rather than publishing an address to write to', async () => {
    const { container } = render(AboutWindow, { api: api() });
    expect(screen.getByText(en['about.issues'])).toBeInTheDocument();
    expect(container.textContent).not.toMatch(/@|mailto:/);
  });

  it('hands the repository URL to the browser rather than navigating itself', async () => {
    const a = api();
    render(AboutWindow, { api: a });
    await fireEvent.click(screen.getByText(en['about.repository']));
    await waitFor(() => expect(a.opened).toBe(1));
    // A real link would take the webview to GitHub and leave no way back.
    expect(document.querySelector('a')).toBeNull();
  });

  it('closes on the close button', async () => {
    const a = api();
    render(AboutWindow, { api: a });
    await fireEvent.click(screen.getByText(en['about.close']));
    await waitFor(() => expect(a.closed).toBe(1));
  });

  it('still renders when the version cannot be read', async () => {
    render(AboutWindow, { api: api({ failVersion: true }) });
    expect(await screen.findByText(en['about.license.value'])).toBeInTheDocument();
    expect(screen.getByTestId('version')).toHaveTextContent('—');
  });

  it('follows the chosen language (12.4)', async () => {
    render(AboutWindow, { api: api() });
    i18n.setLanguage('ja');
    expect(await screen.findByText(ja['about.issues'])).toBeInTheDocument();
  });
});
