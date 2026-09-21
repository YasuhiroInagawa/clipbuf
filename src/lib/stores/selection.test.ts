import { get, writable } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import type { ItemDto } from '$lib/ipc/types';
import { createSelectionStore, moveSelection, reconcileSelection } from './selection';

const dto = (id: number): ItemDto => ({
  id,
  capturedAtMs: id,
  text: `t${id}`,
  hasStyle: false,
  warnings: [],
});
const list = (...ids: number[]) => ids.map(dto);

describe('moveSelection', () => {
  it('moves down and up through the list and stops at the ends', () => {
    const items = list(3, 2, 1);
    expect(moveSelection(items, 3, 1)).toBe(2);
    expect(moveSelection(items, 2, 1)).toBe(1);
    expect(moveSelection(items, 1, 1)).toBe(1);
    expect(moveSelection(items, 1, -1)).toBe(2);
    expect(moveSelection(items, 3, -1)).toBe(3);
  });

  it('selects the first item when nothing is selected, and null on an empty list', () => {
    expect(moveSelection(list(3, 2), null, 1)).toBe(3);
    expect(moveSelection(list(3, 2), null, -1)).toBe(3);
    expect(moveSelection([], null, 1)).toBeNull();
    expect(moveSelection(list(3, 2), 99, 1)).toBe(3);
  });
});

describe('reconcileSelection', () => {
  it('keeps the selection when the item still exists', () => {
    expect(reconcileSelection(list(3, 2, 1), 2, list(3, 2, 1))).toBe(2);
  });

  it('moves to the neighbour that took the removed item place, else the last item', () => {
    // 2 removed from [3,2,1] → the item now at that index (1) is selected
    expect(reconcileSelection(list(3, 1), 2, list(3, 2, 1))).toBe(1);
    // 1 (last) removed from [3,2,1] → previous neighbour 2
    expect(reconcileSelection(list(3, 2), 1, list(3, 2, 1))).toBe(2);
    // only item removed → nothing
    expect(reconcileSelection([], 1, list(1))).toBeNull();
  });

  it('selects nothing when the list is empty and the first item when selection is unknown', () => {
    expect(reconcileSelection([], null, [])).toBeNull();
    expect(reconcileSelection(list(4), 99, [])).toBe(4);
  });
});

describe('selection store', () => {
  it('follows the items store and reacts to window-shown by selecting the newest', () => {
    const items = writable<ItemDto[]>(list(3, 2, 1));
    const sel = createSelectionStore(items);
    expect(get(sel.selectedId)).toBe(3);
    sel.moveDown();
    sel.moveDown();
    expect(get(sel.selectedId)).toBe(1);
    sel.selectNewest();
    expect(get(sel.selectedId)).toBe(3);
    sel.moveDown();
    items.set(list(3, 1)); // 2 removed by the backend
    expect(get(sel.selectedId)).toBe(1);
    items.set(list(9, 3, 1)); // newer item arrived: selection stays on 1
    expect(get(sel.selectedId)).toBe(1);
    sel.select(9);
    expect(get(sel.selectedId)).toBe(9);
  });
});
