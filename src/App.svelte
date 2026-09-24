<script lang="ts">
  import { onMount } from 'svelte';
  import { createAppContext, createSettingsContext, syncLanguage } from '$lib/app';
  import { updateApi } from '$lib/ipc/update';
  import { currentWindowLabel } from '$lib/ipc/window';
  import MainWindow from '$lib/components/MainWindow.svelte';
  import SettingsWindow from '$lib/components/SettingsWindow.svelte';

  const isMain = currentWindowLabel() === 'main';
  const main = isMain ? createAppContext() : null;
  const settings = isMain ? null : createSettingsContext();

  onMount(() => {
    if (settings) {
      void settings.store.start();
      const stop = syncLanguage(settings.store);
      return () => {
        stop();
        settings.store.stop();
      };
    }
    // The main window starts its own stores in MainWindow; only the language link is added here.
    return main ? syncLanguage(main.settings) : undefined;
  });
</script>

{#if main}
  <MainWindow ctx={main} updates={updateApi} />
{:else if settings}
  <SettingsWindow
    store={settings.store}
    getPlatformInfo={settings.getPlatformInfo}
    closeWindow={settings.closeWindow}
    suspendHotkey={settings.suspendHotkey}
    resumeHotkey={settings.resumeHotkey}
  />
{/if}
