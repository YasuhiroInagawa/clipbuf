<script lang="ts">
  import { createAppContext } from '$lib/app';
  import { currentWindowLabel } from '$lib/ipc/window';
  import MainWindow from '$lib/components/MainWindow.svelte';
  import { t } from '$lib/stores/i18n';

  const label = currentWindowLabel();
  const ctx = label === 'main' ? createAppContext() : null;
</script>

{#if ctx}
  <MainWindow {ctx} />
{:else}
  <!-- Settings window content arrives with task 5.5 -->
  <main class="placeholder"><p>{$t('settings.title')}</p></main>
{/if}

<style>
  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    opacity: 0.6;
  }
</style>
