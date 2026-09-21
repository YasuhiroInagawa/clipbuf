/**
 * Transient notices (transfer done, transforms skipped, failures). One at a time; a newer
 * notice replaces the current one. Messages are i18n keys, resolved by the component.
 */

import { writable, type Readable } from 'svelte/store';

export type NoticeKind = 'success' | 'info' | 'error';

export interface Notice {
  id: number;
  kind: NoticeKind;
  /** i18n key */
  message: string;
}

export interface NoticeStore {
  notice: Readable<Notice | null>;
  show(kind: NoticeKind, message: string, lifetimeMs?: number): void;
  dismiss(): void;
}

const DEFAULT_LIFETIME_MS: Record<NoticeKind, number> = {
  success: 2000,
  info: 3500,
  error: 6000,
};

export function createNoticeStore(): NoticeStore {
  const notice = writable<Notice | null>(null);
  let timer: ReturnType<typeof setTimeout> | null = null;
  let nextId = 1;

  const clearTimer = () => {
    if (timer !== null) clearTimeout(timer);
    timer = null;
  };

  return {
    notice: { subscribe: notice.subscribe },
    show(kind, message, lifetimeMs = DEFAULT_LIFETIME_MS[kind]) {
      clearTimer();
      const id = nextId++;
      notice.set({ id, kind, message });
      timer = setTimeout(() => {
        notice.update((n) => (n?.id === id ? null : n));
        timer = null;
      }, lifetimeMs);
    },
    dismiss() {
      clearTimer();
      notice.set(null);
    },
  };
}

/** App-wide instance. */
export const notices: NoticeStore = createNoticeStore();
