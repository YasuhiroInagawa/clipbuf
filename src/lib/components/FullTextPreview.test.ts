/** @vitest-environment jsdom */
import { render, waitFor } from '@testing-library/svelte';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { i18n } from '$lib/stores/i18n';
import FullTextPreview from './FullTextPreview.svelte';

beforeAll(() => i18n.setLanguage('en'));

/** Visible lines, each as the concatenated token text. */
function lines(container: HTMLElement): string[] {
  return [...container.querySelectorAll('.pv-line')].map((el) => el.textContent ?? '');
}

describe('FullTextPreview', () => {
  it('shows the transferred text with newlines actually breaking lines (4.6)', async () => {
    const load = vi.fn(async () => ({ text: 'first\r\nsecond\nthird', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null });
    await waitFor(() => expect(lines(container)).toHaveLength(3));
    expect(lines(container)[0]).toBe('first↵');
    expect(lines(container)[1]).toBe('second↵');
    expect(lines(container)[2]).toBe('third');
  });

  it('keeps whitespace visualisation and renders a tab as one symbol (4.6)', async () => {
    const load = vi.fn(async () => ({ text: 'a\tb c　d', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null });
    await waitFor(() => expect(lines(container)).toHaveLength(1));
    expect(lines(container)[0]).toBe('a→b·c□d');
    expect(container.querySelectorAll('.tok[data-kind="tab"]')).toHaveLength(1);
  });

  it('asks the backend for the transferred text (4.7)', async () => {
    const load = vi.fn(async () => ({ text: 'transformed', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null });
    await waitFor(() => expect(lines(container)[0]).toBe('transformed'));
    expect(load).toHaveBeenCalledTimes(1);
  });

  it('notes when formatting was kept so transforms were skipped', async () => {
    const load = vi.fn(async () => ({ text: 'raw\ttext', skippedTransforms: true }));
    const { container } = render(FullTextPreview, { load, anchorEl: null });
    await waitFor(() => expect(container.querySelector('.pv-note')).not.toBeNull());
  });

  it('shows a legend for the newline kinds the text contains (3.9)', async () => {
    const load = vi.fn(async () => ({ text: 'a\r\nb\nc', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null });
    await waitFor(() => expect(container.querySelector('.pv-legend')).not.toBeNull());
    const entries = [...container.querySelectorAll('.pv-legend [data-newline]')].map((el) =>
      el.getAttribute('data-newline'),
    );
    expect(entries).toEqual(['crlf', 'lf']);
    expect(container.querySelector('.pv-legend')).toHaveTextContent('CRLF');
  });

  it('omits the legend when the text has no newlines', async () => {
    const load = vi.fn(async () => ({ text: 'one line', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null });
    await waitFor(() => expect(container.querySelector('.pv-body')).not.toBeNull());
    expect(container.querySelector('.pv-legend')).toBeNull();
  });

  it('shows an empty-text marker rather than an empty box', async () => {
    const load = vi.fn(async () => ({ text: '', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null });
    await waitFor(() => expect(container.querySelector('.pv-empty')).not.toBeNull());
  });
});
