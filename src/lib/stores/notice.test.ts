import { get } from 'svelte/store';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createNoticeStore } from './notice';

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('notice store', () => {
  it('shows a notice and clears it after its lifetime', () => {
    const store = createNoticeStore();
    expect(get(store.notice)).toBeNull();
    store.show('success', 'notice.transferred', 1000);
    expect(get(store.notice)).toMatchObject({ kind: 'success', message: 'notice.transferred' });
    vi.advanceTimersByTime(999);
    expect(get(store.notice)).not.toBeNull();
    vi.advanceTimersByTime(1);
    expect(get(store.notice)).toBeNull();
  });

  it('a newer notice replaces the current one and restarts the timer', () => {
    const store = createNoticeStore();
    store.show('info', 'first', 1000);
    vi.advanceTimersByTime(800);
    store.show('error', 'second', 1000);
    vi.advanceTimersByTime(800);
    expect(get(store.notice)).toMatchObject({ kind: 'error', message: 'second' });
    vi.advanceTimersByTime(200);
    expect(get(store.notice)).toBeNull();
  });

  it('dismiss clears immediately and cancels the timer', () => {
    const store = createNoticeStore();
    store.show('info', 'x', 1000);
    store.dismiss();
    expect(get(store.notice)).toBeNull();
    store.show('info', 'y', 5000);
    vi.advanceTimersByTime(1000);
    expect(get(store.notice)).toMatchObject({ message: 'y' }); // old timer did not fire
  });

  it('errors stay longer than successes by default', () => {
    const store = createNoticeStore();
    store.show('success', 'ok');
    vi.advanceTimersByTime(2500);
    expect(get(store.notice)).toBeNull();
    store.show('error', 'bad');
    vi.advanceTimersByTime(2500);
    expect(get(store.notice)).not.toBeNull();
    vi.advanceTimersByTime(3500);
    expect(get(store.notice)).toBeNull();
  });
});
