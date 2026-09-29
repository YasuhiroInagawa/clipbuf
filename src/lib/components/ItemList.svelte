<script lang="ts">
  import type { ItemDto, ItemId, TransferMode, TransferPreview } from '$lib/ipc/types';
  import { t } from '$lib/stores/i18n';
  import type { SelectionStore } from '$lib/stores/selection';
  import type { Readable } from 'svelte/store';
  import ItemRow from './ItemRow.svelte';

  interface Props {
    items: Readable<ItemDto[]>;
    selection: SelectionStore;
    highlightId: ItemId | null;
    onTransfer: (id: ItemId, mode: TransferMode) => void;
    loadPreview: (id: ItemId) => Promise<TransferPreview>;
    previewWrap: boolean;
    /** Incremented by MainWindow on every arrow-key move (4.5.2, 4.5.3). */
    keyboardMoves: number;
  }

  let {
    items,
    selection,
    highlightId,
    onTransfer,
    loadPreview,
    previewWrap,
    keyboardMoves,
  }: Props = $props();
  const selectedId = $derived(selection.selectedId);

  /** Reopening after a keyboard move uses the same delay as resting the pointer on a row. */
  const KEYBOARD_PREVIEW_DELAY_MS = 500;

  /** True while the pointer is anywhere over the list, the popover included: the pointer then
   *  owns the preview, and a keyboard-driven one would fight it (4.5.3). */
  let pointerInside = $state(false);
  /** The row whose preview the keyboard opened, if any. */
  let keyboardPreviewFor: ItemId | null = $state(null);
  let restTimer: ReturnType<typeof setTimeout> | null = null;

  // Every arrow-key move cancels the pending open and, once the keys stop, shows the row the
  // selection landed on. Reading `keyboardMoves` is what subscribes this to each move.
  $effect(() => {
    const moves = keyboardMoves;
    keyboardPreviewFor = null;
    if (restTimer) clearTimeout(restTimer);
    // Nothing has moved yet on the first run; opening here would show a preview nobody asked for.
    if (moves === 0 || pointerInside) return;
    const id = $selectedId;
    restTimer = setTimeout(() => {
      keyboardPreviewFor = id;
      restTimer = null;
    }, KEYBOARD_PREVIEW_DELAY_MS);
    return () => {
      if (restTimer) clearTimeout(restTimer);
      restTimer = null;
    };
  });

  function onPointerEnter(): void {
    pointerInside = true;
    // The pointer takes over: drop the keyboard's claim so it cannot reopen behind the hover.
    keyboardPreviewFor = null;
    if (restTimer) clearTimeout(restTimer);
    restTimer = null;
  }
</script>

{#if $items.length === 0}
  <p class="empty">{$t('list.empty')}</p>
{:else}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <ul
    class="list"
    role="listbox"
    aria-label={$t('app.name')}
    onmouseenter={onPointerEnter}
    onmouseleave={() => (pointerInside = false)}
  >
    {#each $items as item (item.id)}
      <ItemRow
        {item}
        selected={item.id === $selectedId}
        highlight={item.id === highlightId}
        onSelect={() => selection.select(item.id)}
        onTransfer={(mode) => onTransfer(item.id, mode)}
        loadPreview={() => loadPreview(item.id)}
        {previewWrap}
        closeToken={keyboardMoves}
        keyboardPreview={keyboardPreviewFor === item.id}
      />
    {/each}
  </ul>
{/if}

<style>
  .list {
    margin: 0;
    padding: 0.25em;
    overflow-y: auto;
    flex: 1 1 auto;
    min-height: 0;
  }
  .empty {
    margin: auto;
    padding: 1em;
    text-align: center;
    opacity: 0.6;
    font-size: 0.9em;
  }
</style>
