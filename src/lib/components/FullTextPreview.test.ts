/** @vitest-environment jsdom */
import { fireEvent, render, waitFor } from '@testing-library/svelte';
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
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(lines(container)).toHaveLength(3));
    expect(lines(container)[0]).toBe('first↵');
    expect(lines(container)[1]).toBe('second↵');
    expect(lines(container)[2]).toBe('third');
  });

  it('keeps whitespace visualisation and renders a tab as one symbol (4.6)', async () => {
    const load = vi.fn(async () => ({ text: 'a\tb c　d', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(lines(container)).toHaveLength(1));
    expect(lines(container)[0]).toBe('a→b·c□d');
    expect(container.querySelectorAll('.tok[data-kind="tab"]')).toHaveLength(1);
  });

  it('asks the backend for the transferred text (4.7)', async () => {
    const load = vi.fn(async () => ({ text: 'transformed', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(lines(container)[0]).toBe('transformed'));
    expect(load).toHaveBeenCalledTimes(1);
  });

  it('notes when formatting was kept so transforms were skipped', async () => {
    const load = vi.fn(async () => ({ text: 'raw\ttext', skippedTransforms: true }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(container.querySelector('.pv-note')).not.toBeNull());
  });

  it('shows a legend for the newline kinds the text contains (3.9)', async () => {
    const load = vi.fn(async () => ({ text: 'a\r\nb\nc', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(container.querySelector('.pv-legend')).not.toBeNull());
    const entries = [...container.querySelectorAll('.pv-legend [data-newline]')].map((el) =>
      el.getAttribute('data-newline'),
    );
    expect(entries).toEqual(['crlf', 'lf']);
    expect(container.querySelector('.pv-legend')).toHaveTextContent('CRLF');
  });

  it('omits the legend when the text has no newlines', async () => {
    const load = vi.fn(async () => ({ text: 'one line', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(container.querySelector('.pv-body')).not.toBeNull());
    expect(container.querySelector('.pv-legend')).toBeNull();
  });

  it('wraps long lines when asked, so nothing scrolls sideways (4.11)', async () => {
    const load = vi.fn(async () => ({ text: 'x'.repeat(400), skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(container.querySelector('.pv-body')).not.toBeNull());
    expect(getComputedStyle(container.querySelector('.pv-body') as HTMLElement).whiteSpace).toBe(
      'pre-wrap',
    );
  });

  it('keeps one line per line when wrapping is off (4.12)', async () => {
    const load = vi.fn(async () => ({ text: 'x'.repeat(400), skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: false });
    await waitFor(() => expect(container.querySelector('.pv-body')).not.toBeNull());
    expect(getComputedStyle(container.querySelector('.pv-body') as HTMLElement).whiteSpace).toBe(
      'pre',
    );
  });

  it('can be resized by the user (4.10)', async () => {
    const load = vi.fn(async () => ({ text: 'text', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(container.querySelector('.pv')).not.toBeNull());
    expect(getComputedStyle(container.querySelector('.pv') as HTMLElement).resize).toBe('both');
  });

  it('can be enlarged up to the size of the screen (4.10)', async () => {
    const load = vi.fn(async () => ({ text: 'text', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(container.querySelector('.pv')).not.toBeNull());
    const style = getComputedStyle(container.querySelector('.pv') as HTMLElement);
    expect(style.maxWidth).toContain('100vw');
    expect(style.maxHeight).toContain('100vh');
  });

  it('does not close while a resize drag is in progress (4.10)', async () => {
    const load = vi.fn(async () => ({ text: 'text', skippedTransforms: false }));
    const leaves: number[] = [];
    const { container } = render(FullTextPreview, {
      load,
      anchorEl: null,
      wrap: true,
      onPointerLeave: () => leaves.push(1),
    });
    await waitFor(() => expect(container.querySelector('.pv')).not.toBeNull());
    const box = container.querySelector('.pv') as HTMLElement;

    // Dragging the resize corner takes the pointer outside the box; that must not dismiss it.
    await fireEvent.mouseDown(box);
    await fireEvent.mouseLeave(box);
    expect(leaves).toHaveLength(0);

    // Releasing outside the box ends the drag and then the usual leave applies.
    await fireEvent.mouseUp(window);
    expect(leaves).toHaveLength(1);
  });

  it('keeps the preview open when the drag ends inside it', async () => {
    const load = vi.fn(async () => ({ text: 'text', skippedTransforms: false }));
    const leaves: number[] = [];
    const { container } = render(FullTextPreview, {
      load,
      anchorEl: null,
      wrap: true,
      onPointerLeave: () => leaves.push(1),
    });
    await waitFor(() => expect(container.querySelector('.pv')).not.toBeNull());
    const box = container.querySelector('.pv') as HTMLElement;
    await fireEvent.mouseEnter(box);
    await fireEvent.mouseDown(box);
    await fireEvent.mouseUp(window);
    expect(leaves).toHaveLength(0);
  });

  it('shows an empty-text marker rather than an empty box', async () => {
    const load = vi.fn(async () => ({ text: '', skippedTransforms: false }));
    const { container } = render(FullTextPreview, { load, anchorEl: null, wrap: true });
    await waitFor(() => expect(container.querySelector('.pv-empty')).not.toBeNull());
  });
});
