/**
 * Copy of the item list held by the Rust core (design: "データは Rust が正").
 * Initialised with `list_items`, then kept in sync by `item-added` (insert at front) and
 * `items-changed` (replace).
 */

import { writable, type Readable } from 'svelte/store';
import type { ItemDto } from '$lib/ipc/types';
import type { UnlistenFn } from '$lib/ipc/events';

export interface ItemsApi {
  listItems(): Promise<ItemDto[]>;
  onItemAdded(cb: (item: ItemDto) => void): Promise<UnlistenFn>;
  onItemsChanged(cb: (items: ItemDto[]) => void): Promise<UnlistenFn>;
}

export interface ItemsStore {
  items: Readable<ItemDto[]>;
  /** Subscribe to backend events and load the current list. */
  start(): Promise<void>;
  stop(): void;
}

export function createItemsStore(api: ItemsApi): ItemsStore {
  const items = writable<ItemDto[]>([]);
  let unlisten: UnlistenFn[] = [];

  return {
    items: { subscribe: items.subscribe },
    async start() {
      unlisten = await Promise.all([
        api.onItemAdded((item) => items.update((list) => [item, ...list])),
        api.onItemsChanged((list) => items.set(list)),
      ]);
      items.set(await api.listItems());
    },
    stop() {
      for (const u of unlisten) u();
      unlisten = [];
    },
  };
}
