<script lang="ts">
  import { onMount } from 'svelte';
  import { REPOSITORY_URL, type AboutApi } from '$lib/ipc/about';
  import { t } from '$lib/stores/i18n';

  interface Props {
    api: AboutApi;
  }

  let { api }: Props = $props();

  /** `null` until the backend answers; the window is useful without it. */
  let version: string | null = $state(null);

  onMount(() => {
    let alive = true;
    void api
      .getVersion()
      .then((v) => {
        if (alive) version = v;
      })
      .catch((e) => console.warn('about: could not read the version', e));
    return () => {
      alive = false;
    };
  });
</script>

<main class="about">
  <h1>clipbuf</h1>
  <p class="tagline">{$t('about.tagline')}</p>

  <dl class="facts">
    <dt>{$t('about.version')}</dt>
    <dd data-testid="version">{version ?? '—'}</dd>
    <dt>{$t('about.license')}</dt>
    <dd>{$t('about.license.value')}</dd>
  </dl>

  <p class="privacy">{$t('about.privacy')}</p>

  <!-- A plain link would navigate the webview itself; the backend hands the URL to the
       browser instead, and the address is shown so it can be checked before clicking. -->
  <p class="repo">
    <button type="button" class="link" onclick={() => void api.openRepository()}>
      {$t('about.repository')}
    </button>
    <span class="url">{REPOSITORY_URL}</span>
  </p>

  <div class="actions">
    <button type="button" class="close" onclick={() => void api.close()}>
      {$t('about.close')}
    </button>
  </div>
</main>

<style>
  .about {
    display: flex;
    flex-direction: column;
    gap: 0.6em;
    padding: 1em 1.25em;
    font-size: 0.95em;
  }
  h1 {
    margin: 0;
    font-size: 1.3em;
  }
  .tagline {
    margin: -0.4em 0 0;
    opacity: 0.8;
  }
  .facts {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.2em 0.8em;
    margin: 0;
  }
  .facts dt {
    opacity: 0.7;
  }
  .facts dd {
    margin: 0;
    font-family: ui-monospace, monospace;
  }
  .privacy {
    margin: 0;
    font-size: 0.85em;
    opacity: 0.75;
  }
  .repo {
    display: flex;
    flex-direction: column;
    gap: 0.2em;
    margin: 0;
  }
  .url {
    font-family: ui-monospace, monospace;
    font-size: 0.8em;
    opacity: 0.6;
    user-select: text;
    overflow-wrap: anywhere;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 0.2em;
  }
  button {
    font: inherit;
    cursor: pointer;
  }
  .link {
    align-self: flex-start;
    padding: 0;
    border: 0;
    background: none;
    color: #2563eb;
    text-decoration: underline;
  }
  .close {
    padding: 0.35em 1.2em;
    border: 1px solid color-mix(in srgb, currentColor 35%, transparent);
    border-radius: 4px;
    background: color-mix(in srgb, currentColor 10%, transparent);
    color: inherit;
  }
</style>
