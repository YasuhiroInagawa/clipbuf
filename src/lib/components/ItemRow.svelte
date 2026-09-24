<script lang="ts">
  import type { ItemDto, TransferMode, TransferPreview } from '$lib/ipc/types';
  import { newlineKindsIn } from '$lib/preview/charset';
  import { t } from '$lib/stores/i18n';
  import FullTextPreview from './FullTextPreview.svelte';
  import PreviewLine from './PreviewLine.svelte';
  import WarningIcons from './WarningIcons.svelte';

  interface Props {
    item: ItemDto;
    selected: boolean;
    highlight: boolean;
    onTransfer: (mode: TransferMode) => void;
    onSelect: () => void;
    /** Text this item would put on the clipboard with the current options (4.7). */
    loadPreview: () => Promise<TransferPreview>;
    /** Wrap long lines in the preview (4.11, 4.12). */
    previewWrap: boolean;
  }

  let { item, selected, highlight, onTransfer, onSelect, loadPreview, previewWrap }: Props =
    $props();
  let element: HTMLLIElement | undefined = $state();
  let previewOpen = $state(false);
  /** The pointer travels through a gap between the row and the popover; close only if it has
   *  not arrived in the popover shortly after leaving the row (4.8). */
  let closeTimer: ReturnType<typeof setTimeout> | null = null;

  const CLOSE_GRACE_MS = 120;

  function openPreview(): void {
    if (closeTimer) {
      clearTimeout(closeTimer);
      closeTimer = null;
    }
    previewOpen = true;
  }

  function requestClosePreview(): void {
    if (closeTimer) clearTimeout(closeTimer);
    closeTimer = setTimeout(() => {
      previewOpen = false;
      closeTimer = null;
    }, CLOSE_GRACE_MS);
  }

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
  onmouseleave={requestClosePreview}
  onkeydown={onKeydown}
  onclick={() => {
    onSelect();
    onTransfer('options');
  }}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="text" onmouseover={openPreview} onfocusin={openPreview}>
    <PreviewLine text={item.text} />
  </div>
  <WarningIcons warnings={item.warnings} newlineKinds={newlineKindsIn(item.text)} />
  <span class="actions">
    <button type="button" data-mode="plain" title={$t('list.transferPlain')} onclick={alt('plain')}>
      {$t('list.plainShort')}
    </button>
    {#if item.hasStyle}
      <!-- Only meaningful for items that carry formatting: without it the result would be
           the same as a row click (6.4.1). -->
      <button type="button" data-mode="raw" title={$t('list.transferRaw')} onclick={alt('raw')}>
        {$t('list.rawShort')}
      </button>
    {/if}
  </span>
</li>

{#if previewOpen}
  <FullTextPreview
    load={loadPreview}
    anchorEl={element ?? null}
    onPointerEnter={openPreview}
    onPointerLeave={requestClosePreview}
    wrap={previewWrap}
  />
{/if}

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
  .text {
    display: flex;
    flex: 1 1 auto;
    min-width: 0;
  }
  /* Always visible so the two alternatives are discoverable, and clearly buttons —
     the warning icons to their left are status, not controls (6.4). */
  .actions {
    display: inline-flex;
    flex: none;
    gap: 0.25em;
  }
  .actions button {
    font: inherit;
    font-size: 0.75em;
    line-height: 1;
    padding: 0.35em 0.6em;
    border: 1px solid color-mix(in srgb, currentColor 35%, transparent);
    border-radius: 4px;
    background: color-mix(in srgb, currentColor 8%, transparent);
    color: inherit;
    cursor: pointer;
    white-space: nowrap;
  }
  .actions button:hover {
    background: color-mix(in srgb, currentColor 20%, transparent);
  }
  .actions button:active {
    background: color-mix(in srgb, currentColor 30%, transparent);
  }
</style>
