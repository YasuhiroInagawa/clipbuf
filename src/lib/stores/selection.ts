/**
 * Keyboard selection over the item list (6.3, 8.3).
 */

import { derived, get, writable, type Readable } from 'svelte/store';
import type { ItemDto, ItemId } from '$lib/ipc/types';

/** Move `delta` rows (+1 down / −1 up), clamped to the list. Nothing selected → first item. */
export function moveSelection(
  items: ItemDto[],
  current: ItemId | null,
  delta: number,
): ItemId | null {
  if (items.length === 0) return null;
  const index = current === null ? -1 : items.findIndex((i) => i.id === current);
  if (index < 0) return items[0].id;
  const next = Math.min(items.length - 1, Math.max(0, index + delta));
  return items[next].id;
}

/**
 * After the list changed, keep the selection if the item survived; otherwise select the item
 * that now occupies the removed item's row, or the last item when it was at the end.
 */
export function reconcileSelection(
  items: ItemDto[],
  current: ItemId | null,
  previous: ItemDto[],
): ItemId | null {
  if (items.length === 0) return null;
  if (current !== null && items.some((i) => i.id === current)) return current;
  const oldIndex = current === null ? -1 : previous.findIndex((i) => i.id === current);
  if (oldIndex < 0) return items[0].id;
  return items[Math.min(oldIndex, items.length - 1)].id;
}

export interface SelectionStore {
  selectedId: Readable<ItemId | null>;
  selectedItem: Readable<ItemDto | null>;
  select(id: ItemId | null): void;
  moveUp(): void;
  moveDown(): void;
  /** Called when the window is shown (8.3). */
  selectNewest(): void;
}

export function createSelectionStore(items: Readable<ItemDto[]>): SelectionStore {
  const selectedId = writable<ItemId | null>(null);
  let previous: ItemDto[] = [];
  let currentItems: ItemDto[] = [];

  items.subscribe((list) => {
    selectedId.update((sel) => reconcileSelection(list, sel, previous));
    previous = list;
    currentItems = list;
  });

  const move = (delta: number) =>
    selectedId.update((sel) => moveSelection(currentItems, sel, delta));

  return {
    selectedId: { subscribe: selectedId.subscribe },
    selectedItem: derived([items, selectedId], ([list, sel]) =>
      sel === null ? null : (list.find((i) => i.id === sel) ?? null),
    ),
    select: (id) => selectedId.set(id),
    moveUp: () => move(-1),
    moveDown: () => move(1),
    selectNewest: () => selectedId.set(get(items)[0]?.id ?? null),
  };
}
