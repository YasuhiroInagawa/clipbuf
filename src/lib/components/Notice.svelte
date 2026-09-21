<script lang="ts">
  import { t } from '$lib/stores/i18n';
  import type { NoticeStore } from '$lib/stores/notice';

  interface Props {
    store: NoticeStore;
  }

  let { store }: Props = $props();
  const notice = $derived(store.notice);
</script>

{#if $notice}
  <button
    type="button"
    class="notice notice-{$notice.kind}"
    role={$notice.kind === 'error' ? 'alert' : 'status'}
    aria-live={$notice.kind === 'error' ? 'assertive' : 'polite'}
    onclick={() => store.dismiss()}
  >
    {$t($notice.message)}
  </button>
{/if}

<style>
  .notice {
    display: block;
    width: 100%;
    box-sizing: border-box;
    padding: 0.35em 0.75em;
    border: 0;
    border-radius: 4px;
    font: inherit;
    font-size: 0.85em;
    text-align: left;
    cursor: pointer;
    color: #fff;
    background: #2563eb;
  }
  .notice-success {
    background: #15803d;
  }
  .notice-info {
    background: #b45309;
  }
  .notice-error {
    background: #b91c1c;
  }
</style>
