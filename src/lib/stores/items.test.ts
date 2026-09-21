import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import type { ItemDto } from '$lib/ipc/types';
import { createItemsStore, type ItemsApi } from './items';

const dto = (id: number, text = `item ${id}`): ItemDto => ({
  id,
  capturedAtMs: 1000 + id,
  text,
  hasStyle: false,
  warnings: [],
});

/** Fake IPC: records listeners so tests can fire events. */
function fakeApi(initial: ItemDto[]): ItemsApi & {
  fireAdded: (i: ItemDto) => void;
  fireChanged: (l: ItemDto[]) => void;
  unlistened: number;
} {
  let added: ((i: ItemDto) => void) | null = null;
  let changed: ((l: ItemDto[]) => void) | null = null;
  const api = {
    unlistened: 0,
    listItems: async () => initial,
    onItemAdded: async (cb: (i: ItemDto) => void) => {
      added = cb;
      return () => {
        added = null;
        api.unlistened += 1;
      };
    },
    onItemsChanged: async (cb: (l: ItemDto[]) => void) => {
      changed = cb;
      return () => {
        changed = null;
        api.unlistened += 1;
      };
    },
    fireAdded: (i: ItemDto) => added?.(i),
    fireChanged: (l: ItemDto[]) => changed?.(l),
  };
  return api;
}

describe('items store', () => {
  it('starts empty, then loads the current list from the backend', async () => {
    const api = fakeApi([dto(2), dto(1)]);
    const store = createItemsStore(api);
    expect(get(store.items)).toEqual([]);
    await store.start();
    expect(get(store.items).map((i) => i.id)).toEqual([2, 1]);
  });

  it('inserts item-added at the front and replaces on items-changed', async () => {
    const api = fakeApi([dto(1)]);
    const store = createItemsStore(api);
    await store.start();
    api.fireAdded(dto(2));
    expect(get(store.items).map((i) => i.id)).toEqual([2, 1]);
    api.fireChanged([dto(5)]);
    expect(get(store.items).map((i) => i.id)).toEqual([5]);
  });

  it('stop detaches both listeners', async () => {
    const api = fakeApi([]);
    const store = createItemsStore(api);
    await store.start();
    store.stop();
    expect(api.unlistened).toBe(2);
    api.fireAdded(dto(9));
    expect(get(store.items)).toEqual([]);
  });
});
