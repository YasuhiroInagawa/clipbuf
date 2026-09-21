/** @vitest-environment jsdom */
import { fireEvent, render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import { PREVIEW_LIMIT } from '$lib/preview/tokenize';
import PreviewLine from './PreviewLine.svelte';

function tokens(container: HTMLElement): { kind: string | null; text: string }[] {
  return [...container.querySelectorAll('.tok')].map((el) => ({
    kind: el.getAttribute('data-kind'),
    text: el.textContent ?? '',
  }));
}

describe('PreviewLine', () => {
  it('renders one line with symbols for whitespace and newlines, keeping the data intact', () => {
    const { container } = render(PreviewLine, { text: 'a\tb\r\nc　d' });
    expect(tokens(container)).toEqual([
      { kind: 'text', text: 'a' },
      { kind: 'tab', text: '→' },
      { kind: 'text', text: 'b' },
      { kind: 'newline', text: '↵' },
      { kind: 'text', text: 'c' },
      { kind: 'fullwidthSpace', text: '□' },
      { kind: 'text', text: 'd' },
    ]);
    // The rendered line never contains a real line break (3.2).
    expect(container.querySelector('.preview')?.textContent).not.toContain('\n');
  });

  it('is a read-only, non-selectable text box (4.2, 4.3)', () => {
    const { container } = render(PreviewLine, { text: 'hello' });
    const line = container.querySelector('.preview') as HTMLElement;
    expect(line).toHaveAttribute('role', 'textbox');
    expect(line).toHaveAttribute('aria-readonly', 'true');
    expect(line).toHaveAttribute('tabindex', '0');
    expect(line.getAttribute('contenteditable')).not.toBe('true');
    const copy = new Event('copy', { bubbles: true, cancelable: true });
    line.dispatchEvent(copy);
    expect(copy.defaultPrevented).toBe(true);
  });

  it('resets the horizontal scroll position when focus leaves (4.4)', async () => {
    const { container } = render(PreviewLine, { text: 'x'.repeat(500) });
    const line = container.querySelector('.preview') as HTMLElement;
    line.scrollLeft = 120;
    await fireEvent.focusOut(line);
    expect(line.scrollLeft).toBe(0);
  });

  it('marks truncated previews with an ellipsis (3.6)', () => {
    const { container } = render(PreviewLine, { text: 'a'.repeat(PREVIEW_LIMIT + 5) });
    expect(container.querySelector('.truncated')).not.toBeNull();
    const { container: short } = render(PreviewLine, { text: 'short' });
    expect(short.querySelector('.truncated')).toBeNull();
  });

  it('labels invisible characters for assistive tech', () => {
    const { container } = render(PreviewLine, { text: 'a​b' });
    const zw = container.querySelector('.tok[data-kind="zeroWidth"]');
    expect(zw).toHaveAttribute('aria-label');
    expect(zw?.textContent).toBe('∅');
  });
});
