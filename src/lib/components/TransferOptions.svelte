<script lang="ts">
  import type { NewlineMode, TransferOptions } from '$lib/ipc/types';
  import { t } from '$lib/stores/i18n';
  import type { NoticeStore } from '$lib/stores/notice';
  import type { SettingsStore } from '$lib/stores/settings';

  interface Props {
    store: SettingsStore;
    notices: NoticeStore;
  }

  let { store, notices }: Props = $props();
  const settings = $derived(store.settings);
  const transfer = $derived($settings?.transfer);

  const NEWLINE_MODES: NewlineMode[] = ['keep', 'remove', 'space'];

  async function change(patch: Partial<TransferOptions>): Promise<void> {
    const error = await store.setTransfer(patch);
    if (error) notices.show('error', 'settings.saveFailed');
  }

  /** While keep-style is on, text transforms only apply to items without formatting (7.6). */
  const hint = $derived(transfer?.keepStyle ? $t('transfer.keepStyle.hint') : undefined);
</script>

{#if transfer}
  <div class="options" role="group" aria-label={$t('transfer.title')}>
    <span class="radios" title={$t('transfer.style.hint')}>
      <span class="radios-label">{$t('transfer.style')}:</span>
      <label class="opt">
        <input
          type="radio"
          name="style"
          checked={transfer.keepStyle}
          onchange={() => change({ keepStyle: true })}
        />
        {$t('transfer.style.keep')}
      </label>
      <label class="opt">
        <input
          type="radio"
          name="style"
          checked={!transfer.keepStyle}
          onchange={() => change({ keepStyle: false })}
        />
        {$t('transfer.style.strip')}
      </label>
    </span>

    <span class="sep" aria-hidden="true"></span>

    <span class="radios" title={hint ?? $t('transfer.newline.hint')}>
      <span class="radios-label">{$t('transfer.newline')}:</span>
      {#each NEWLINE_MODES as mode (mode)}
        <label class="opt">
          <input
            type="radio"
            name="newline"
            value={mode}
            checked={transfer.newline === mode}
            onchange={() => change({ newline: mode })}
          />
          {$t(`transfer.newline.${mode}`)}
        </label>
      {/each}
    </span>

    <span class="sep" aria-hidden="true"></span>

    <label class="opt" title={hint ?? $t('transfer.trim.hint')}>
      <input
        type="checkbox"
        checked={transfer.trim}
        onchange={(e) => change({ trim: e.currentTarget.checked })}
      />
      {$t('transfer.trim')}
    </label>

    <span class="sep" aria-hidden="true"></span>

    <label class="opt" title={hint ?? $t('transfer.tabsToSpaces.hint')}>
      <input
        type="checkbox"
        checked={transfer.tabsToSpaces}
        onchange={(e) => change({ tabsToSpaces: e.currentTarget.checked })}
      />
      {$t('transfer.tabsToSpaces')}
    </label>

    <span class="sep" aria-hidden="true"></span>

    <label class="opt" title={hint ?? $t('transfer.fullwidthToSpace.hint')}>
      <input
        type="checkbox"
        checked={transfer.fullwidthToSpace}
        onchange={(e) => change({ fullwidthToSpace: e.currentTarget.checked })}
      />
      {$t('transfer.fullwidthToSpace')}
    </label>
  </div>
  {#if hint}
    <p class="hint">{hint}</p>
  {/if}
{/if}

<style>
  .options {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.25em 0.9em;
    font-size: 0.85em;
    padding: 0.35em 0.5em;
  }
  .opt {
    display: inline-flex;
    align-items: center;
    gap: 0.3em;
    white-space: nowrap;
    cursor: pointer;
    user-select: none;
  }
  .opt input {
    margin: 0;
  }
  .radios {
    display: inline-flex;
    align-items: center;
    gap: 0.5em;
  }
  .radios-label {
    opacity: 0.75;
  }
  .sep {
    width: 1px;
    height: 1.2em;
    background: currentColor;
    opacity: 0.25;
  }
  .hint {
    margin: 0 0.5em 0.35em;
    font-size: 0.75em;
    opacity: 0.75;
  }
</style>
