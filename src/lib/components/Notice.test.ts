/** @vitest-environment jsdom */
import { fireEvent, render, screen } from '@testing-library/svelte';
import { beforeAll, describe, expect, it } from 'vitest';
import en from '../../locales/en.json';
import { i18n } from '$lib/stores/i18n';
import { createNoticeStore } from '$lib/stores/notice';
import Notice from './Notice.svelte';

beforeAll(() => i18n.setLanguage('en'));

describe('Notice', () => {
  it('renders nothing without a notice', () => {
    const store = createNoticeStore();
    const { container } = render(Notice, { store });
    expect(container.querySelector('.notice')).toBeNull();
  });

  it('renders the translated message with a status role for success and alert for errors', async () => {
    const store = createNoticeStore();
    render(Notice, { store });
    store.show('success', 'notice.transferred', 60_000);
    expect(await screen.findByRole('status')).toHaveTextContent(en['notice.transferred']);
    store.show('error', 'notice.writeFailed', 60_000);
    expect(await screen.findByRole('alert')).toHaveTextContent(en['notice.writeFailed']);
  });

  it('is dismissed by clicking it', async () => {
    const store = createNoticeStore();
    render(Notice, { store });
    store.show('info', 'notice.transformsSkipped', 60_000);
    const el = await screen.findByRole('status');
    await fireEvent.click(el);
    expect(screen.queryByRole('status')).toBeNull();
  });
});
