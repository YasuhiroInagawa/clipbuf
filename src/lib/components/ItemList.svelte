<script lang="ts">
  import type { ItemDto, ItemId, TransferMode } from '$lib/ipc/types';
  import { t } from '$lib/stores/i18n';
  import type { SelectionStore } from '$lib/stores/selection';
  import type { Readable } from 'svelte/store';
  import ItemRow from './ItemRow.svelte';

  interface Props {
    items: Readable<ItemDto[]>;
    selection: SelectionStore;
    highlightId: ItemId | null;
    onTransfer: (id: ItemId, mode: TransferMode) => void;
  }

  let { items, selection, highlightId, onTransfer }: Props = $props();
  const selectedId = $derived(selection.selectedId);
</script>

{#if $items.length === 0}
  <p class="empty">{$t('list.empty')}</p>
{:else}
  <ul class="list" role="listbox" aria-label={$t('app.name')}>
    {#each $items as item (item.id)}
      <ItemRow
        {item}
        selected={item.id === $selectedId}
        highlight={item.id === highlightId}
        onSelect={() => selection.select(item.id)}
        onTransfer={(mode) => onTransfer(item.id, mode)}
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
