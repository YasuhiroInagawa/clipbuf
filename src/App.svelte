<script lang="ts">
  import { onMount } from 'svelte';
  import {
    createAppContext,
    createLanguageOnlyStore,
    createSettingsContext,
    syncLanguage,
  } from '$lib/app';
  import { aboutApi } from '$lib/ipc/about';
  import { updateApi } from '$lib/ipc/update';
  import { currentWindowLabel } from '$lib/ipc/window';
  import AboutWindow from '$lib/components/AboutWindow.svelte';
  import MainWindow from '$lib/components/MainWindow.svelte';
  import SettingsWindow from '$lib/components/SettingsWindow.svelte';

  const label = currentWindowLabel();
  const main = label === 'main' ? createAppContext() : null;
  const settings = label === 'settings' ? createSettingsContext() : null;
  // The about window shows no settings, but still follows the saved language (12.4).
  const aboutLanguage = label === 'about' ? createLanguageOnlyStore() : null;

  onMount(() => {
    if (aboutLanguage) {
      void aboutLanguage.start();
      const stop = syncLanguage(aboutLanguage);
      return () => {
        stop();
        aboutLanguage.stop();
      };
    }
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
{:else if aboutLanguage}
  <AboutWindow api={aboutApi} />
{/if}
