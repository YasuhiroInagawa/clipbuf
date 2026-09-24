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

  /** jsdom reports zero-sized rects, so anchors state their geometry explicitly. */
  function anchorAt(left: number, top: number): HTMLElement {
    const el = document.createElement('div');
    el.getBoundingClientRect = () =>
      ({ left, top, right: left + 300, bottom: top + 24, width: 300, height: 24 }) as DOMRect;
    document.body.appendChild(el);
    return el;
  }

  it('starts at the bottom-left corner of the hovered row, leaving it visible (4.10.1)', async () => {
    const load = vi.fn(async () => ({ text: 'text', skippedTransforms: false }));
    const { container } = render(FullTextPreview, {
      load,
      anchorEl: anchorAt(40, 120),
      wrap: true,
    });
    await waitFor(() => expect(container.querySelector('.pv')).not.toBeNull());
    const box = container.querySelector('.pv') as HTMLElement;
    expect(box.style.left).toBe('40px');
    // The anchor is 24px tall, so the popover starts just under it and never covers the row.
    expect(box.style.top).toBe('144px');
  });

  it('never reaches past the window, keeping a margin at the bottom right (4.10.2)', async () => {
    const load = vi.fn(async () => ({ text: 'x'.repeat(5000), skippedTransforms: false }));
    const { container } = render(FullTextPreview, {
      load,
      anchorEl: anchorAt(200, 300),
      wrap: false,
    });
    await waitFor(() => expect(container.querySelector('.pv')).not.toBeNull());
    const box = container.querySelector('.pv') as HTMLElement;
    const margin = 16;
    expect(parseFloat(box.style.maxWidth)).toBe(window.innerWidth - margin - 200);
    expect(parseFloat(box.style.maxHeight)).toBe(window.innerHeight - margin - 324);
    expect(parseFloat(box.style.width)).toBeLessThanOrEqual(window.innerWidth - margin - 200);
    expect(parseFloat(box.style.height)).toBeLessThanOrEqual(window.innerHeight - margin - 324);
  });

  it('shrinks to fit when the window gets smaller (4.10.2)', async () => {
    const load = vi.fn(async () => ({ text: 'x'.repeat(5000), skippedTransforms: false }));
    const { container } = render(FullTextPreview, {
      load,
      anchorEl: anchorAt(0, 0),
      wrap: false,
    });
    await waitFor(() => expect(container.querySelector('.pv')).not.toBeNull());
    const box = container.querySelector('.pv') as HTMLElement;

    const originalWidth = window.innerWidth;
    try {
      Object.defineProperty(window, 'innerWidth', { value: 320, configurable: true });
      window.dispatchEvent(new Event('resize'));
      await waitFor(() => expect(parseFloat(box.style.width)).toBeLessThanOrEqual(320 - 16));
      expect(parseFloat(box.style.maxWidth)).toBe(320 - 16);
    } finally {
      Object.defineProperty(window, 'innerWidth', { value: originalWidth, configurable: true });
    }
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
