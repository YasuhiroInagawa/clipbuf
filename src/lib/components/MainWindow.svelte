<script lang="ts">
  import { onMount } from 'svelte';
  import { isAppError, type CaptureStatus, type ItemId, type TransferMode } from '$lib/ipc/types';
  import type { MainContext } from '$lib/stores/context';
  import { t } from '$lib/stores/i18n';
  import ItemList from './ItemList.svelte';
  import Notice from './Notice.svelte';
  import TransferOptions from './TransferOptions.svelte';

  interface Props {
    ctx: MainContext;
  }

  let { ctx }: Props = $props();
  // The context is created once per window and never swapped, so reading it eagerly is intended.
  // svelte-ignore state_referenced_locally
  const { items, selection, settings, notices, api } = ctx;

  const HIGHLIGHT_MS = 600;
  let highlightId: ItemId | null = $state(null);
  let highlightTimer: ReturnType<typeof setTimeout> | null = null;

  /** Persistent banner for a capture problem; null when capturing normally (11.5, 11.6). */
  let bannerKey: string | null = $state(null);

  function applyCaptureStatus(status: CaptureStatus): void {
    switch (status) {
      case 'unavailable':
      case 'limitedXWayland':
      case 'denied':
        bannerKey = `capture.${status}`;
        break;
      case 'readFailed':
        notices.show('error', 'notice.readFailed');
        break;
      case 'full':
        bannerKey = null;
        break;
    }
  }

  async function transfer(id: ItemId, mode: TransferMode): Promise<void> {
    try {
      const outcome = await api.transferItem(id, mode);
      if (highlightTimer) clearTimeout(highlightTimer);
      highlightId = id;
      highlightTimer = setTimeout(() => (highlightId = null), HIGHLIGHT_MS);
      if (outcome.skippedTransforms) notices.show('info', 'notice.transformsSkipped');
    } catch (e) {
      if (isAppError(e) && e.kind === 'itemNotFound') return; // list is refreshing
      notices.show('error', 'notice.writeFailed');
    }
  }

  async function remove(id: ItemId): Promise<void> {
    try {
      await api.removeItem(id);
    } catch {
      // Already gone; the items-changed event will reconcile the list.
    }
  }

  function isFormControl(target: EventTarget | null): boolean {
    return (
      target instanceof HTMLInputElement ||
      target instanceof HTMLButtonElement ||
      target instanceof HTMLSelectElement ||
      target instanceof HTMLTextAreaElement
    );
  }

  function onKeydown(e: KeyboardEvent): void {
    const selected = selection.selectedId;
    let current: ItemId | null = null;
    selected.subscribe((v) => (current = v))();
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        selection.moveDown();
        break;
      case 'ArrowUp':
        e.preventDefault();
        selection.moveUp();
        break;
      case 'Enter':
        if (isFormControl(e.target)) return;
        e.preventDefault();
        if (current !== null) void transfer(current, e.shiftKey ? 'plain' : 'options');
        break;
      case 'Delete':
      case 'Backspace':
        if (isFormControl(e.target)) return;
        e.preventDefault();
        if (current !== null) void remove(current);
        break;
      case 'Escape':
        e.preventDefault();
        void api.hideWindow();
        break;
    }
  }

  onMount(() => {
    const unlisteners: (() => void)[] = [];
    void (async () => {
      unlisteners.push(await api.onCaptureStatus(applyCaptureStatus));
      unlisteners.push(await api.onWindowShown(() => selection.selectNewest()));
      await Promise.all([items.start(), settings.start()]);
      applyCaptureStatus((await api.getPlatformInfo()).capture);
    })();
    return () => {
      for (const u of unlisteners) u();
      items.stop();
      settings.stop();
      if (highlightTimer) clearTimeout(highlightTimer);
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

<div class="main">
  <header class="header">
    <TransferOptions store={settings} {notices} />
  </header>
  {#if bannerKey}
    <p class="banner" role="note">{$t(bannerKey)}</p>
  {/if}
  <div class="notice-slot">
    <Notice store={notices} />
  </div>
  <ItemList
    items={items.items}
    {selection}
    {highlightId}
    onTransfer={transfer}
    loadPreview={(id) => api.previewTransfer(id)}
  />
  <footer class="footer">
    <button type="button" class="clear" onclick={() => void api.clearItems()}>
      {$t('list.clear')}
    </button>
  </footer>
</div>

<style>
  .main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    min-height: 0;
  }
  .header {
    flex: none;
    border-bottom: 1px solid color-mix(in srgb, currentColor 15%, transparent);
  }
  .banner {
    flex: none;
    margin: 0;
    padding: 0.4em 0.75em;
    font-size: 0.8em;
    color: #fff;
    background: #b45309;
  }
  .notice-slot {
    flex: none;
    padding: 0 0.25em;
  }
  .footer {
    flex: none;
    display: flex;
    justify-content: flex-end;
    padding: 0.25em 0.5em;
    border-top: 1px solid color-mix(in srgb, currentColor 15%, transparent);
  }
  .clear {
    font: inherit;
    font-size: 0.8em;
    padding: 0.2em 0.6em;
    border: 1px solid color-mix(in srgb, currentColor 30%, transparent);
    border-radius: 3px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
</style>
