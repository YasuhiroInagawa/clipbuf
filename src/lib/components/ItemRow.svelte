<script lang="ts">
  import type { ItemDto, TransferMode } from '$lib/ipc/types';
  import { t } from '$lib/stores/i18n';
  import PreviewLine from './PreviewLine.svelte';
  import WarningIcons from './WarningIcons.svelte';

  interface Props {
    item: ItemDto;
    selected: boolean;
    highlight: boolean;
    onTransfer: (mode: TransferMode) => void;
    onSelect: () => void;
  }

  let { item, selected, highlight, onTransfer, onSelect }: Props = $props();
  let element: HTMLLIElement | undefined = $state();

  // Keep the keyboard selection visible.
  $effect(() => {
    if (selected && typeof element?.scrollIntoView === 'function') {
      element.scrollIntoView({ block: 'nearest' });
    }
  });

  /** Row-level keyboard activation (focus is programmatic, tabindex -1). The window-level
   *  handler in MainWindow covers the common case; stopPropagation avoids a double transfer. */
  function onKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      e.stopPropagation();
      onSelect();
      onTransfer(e.shiftKey ? 'plain' : 'options');
    }
  }

  function alt(mode: TransferMode) {
    return (e: MouseEvent) => {
      e.stopPropagation();
      onSelect();
      onTransfer(mode);
    };
  }
</script>

<li
  bind:this={element}
  role="option"
  tabindex="-1"
  class="item"
  class:selected
  class:highlight
  data-id={item.id}
  aria-selected={selected}
  onmouseenter={onSelect}
  onkeydown={onKeydown}
  onclick={() => {
    onSelect();
    onTransfer('options');
  }}
>
  <PreviewLine text={item.text} />
  <WarningIcons warnings={item.warnings} />
  <span class="actions">
    <button type="button" data-mode="plain" title={$t('list.transferPlain')} onclick={alt('plain')}>
      T
    </button>
    <button type="button" data-mode="raw" title={$t('list.transferRaw')} onclick={alt('raw')}>
      =
    </button>
  </span>
</li>

<style>
  .item {
    display: flex;
    align-items: center;
    gap: 0.5em;
    padding: 0.2em 0.5em;
    border-radius: 4px;
    cursor: pointer;
    list-style: none;
    transition: background-color 120ms;
  }
  .item.selected {
    background: color-mix(in srgb, #5b6ee1 18%, transparent);
  }
  .item.highlight {
    background: color-mix(in srgb, #15803d 35%, transparent);
  }
  .actions {
    display: none;
    flex: none;
    gap: 0.25em;
  }
  .item:hover .actions,
  .item:focus-within .actions {
    display: inline-flex;
  }
  .actions button {
    font: inherit;
    font-family: ui-monospace, monospace;
    font-size: 0.8em;
    line-height: 1;
    padding: 0.2em 0.45em;
    border: 1px solid color-mix(in srgb, currentColor 30%, transparent);
    border-radius: 3px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  .actions button:hover {
    background: color-mix(in srgb, currentColor 12%, transparent);
  }
</style>
