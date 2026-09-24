<script lang="ts">
  import { onMount } from 'svelte';
  import type { UpdateApi, UpdateHandle } from '$lib/ipc/update';
  import { t } from '$lib/stores/i18n';

  interface Props {
    api: UpdateApi;
  }

  let { api }: Props = $props();

  type Phase = 'idle' | 'offered' | 'installing' | 'installed' | 'failed';

  let update: UpdateHandle | null = $state(null);
  let phase: Phase = $state('idle');

  onMount(() => {
    let alive = true;
    void api
      .check()
      .then((found) => {
        if (!alive || !found) return;
        update = found;
        phase = 'offered';
      })
      // Being offline is normal; the app just keeps running (13.8).
      .catch((e) => console.warn('update check failed', e));
    return () => {
      alive = false;
    };
  });

  async function install(): Promise<void> {
    if (!update) return;
    phase = 'installing';
    try {
      await update.downloadAndInstall();
      phase = 'installed';
    } catch (e) {
      console.warn('update install failed', e);
      phase = 'failed';
    }
  }

  function dismiss(): void {
    // The current version keeps running; the offer returns at the next start (13.8).
    phase = 'idle';
    update = null;
  }
</script>

{#if update && phase !== 'idle'}
  <div class="update" role="status" aria-live="polite">
    {#if phase === 'installed'}
      <span class="text">{$t('update.installed')}</span>
      <button type="button" class="primary" onclick={() => void api.relaunch()}>
        {$t('update.restart')}
      </button>
    {:else}
      <span class="text">
        {$t('update.available').replace('{version}', update.version)}
        {#if phase === 'failed'}<span class="failed">{$t('update.failed')}</span>{/if}
      </span>
      <button
        type="button"
        class="primary"
        disabled={phase === 'installing'}
        onclick={() => void install()}
      >
        {phase === 'installing' ? $t('update.installing') : $t('update.install')}
      </button>
      <button type="button" onclick={dismiss}>{$t('update.dismiss')}</button>
    {/if}
  </div>
{/if}

<style>
  .update {
    /* Sits above the header in the main window's flex column and keeps its full height. */
    flex: none;
    display: flex;
    align-items: center;
    gap: 0.5em;
    padding: 0.35em 0.75em;
    font-size: 0.85em;
    color: #fff;
    background: #1d4ed8;
  }
  .text {
    flex: 1 1 auto;
  }
  .failed {
    display: block;
    opacity: 0.9;
  }
  button {
    font: inherit;
    font-size: 0.95em;
    padding: 0.2em 0.7em;
    border: 1px solid rgb(255 255 255 / 55%);
    border-radius: 3px;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  button.primary {
    background: rgb(255 255 255 / 20%);
  }
  button:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
