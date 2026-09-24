<script lang="ts">
  import { WARNINGS, type Warning } from '$lib/ipc/types';
  import type { NewlineKind } from '$lib/preview/charset';
  import { t } from '$lib/stores/i18n';

  interface Props {
    warnings: Warning[];
    /** Newline conventions in the item, to name them in the mixed-newlines hint (5.6). */
    newlineKinds?: NewlineKind[];
  }

  let { warnings, newlineKinds = [] }: Props = $props();

  /** Glyph per warning kind; the meaning is carried by aria-label / title. */
  const GLYPH: Record<Warning, string> = {
    hasStyle: 'S',
    edgeWhitespace: '⎵',
    hasTab: '⇥',
    platformDependent: '㊙',
    controlOrBinary: '�',
    mixedNewlines: '⏎',
    encodingNotice: 'Ⓤ',
  };

  // Fixed display order regardless of the order the backend sent (5.9 / design).
  const shown = $derived(WARNINGS.filter((w) => warnings.includes(w)));

  /** Name the actual conventions when we know them, else the generic hint. */
  function hint(warning: Warning): string {
    if (warning === 'mixedNewlines' && newlineKinds.length > 0) {
      return $t('warning.mixedNewlines.hintWith').replace(
        '{kinds}',
        newlineKinds.map((k) => k.toUpperCase()).join(', '),
      );
    }
    return $t(`warning.${warning}.hint`);
  }
</script>

<span class="warnings">
  {#each shown as warning (warning)}
    <span
      class="warning warning-{warning}"
      data-warning={warning}
      role="img"
      aria-label={$t(`warning.${warning}`)}
      title={hint(warning)}>{GLYPH[warning]}</span
    >
  {/each}
</span>

<style>
  .warnings {
    display: inline-flex;
    gap: 0.25em;
    flex: none;
    align-items: center;
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.85em;
    line-height: 1;
  }
  .warning {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.5em;
    height: 1.5em;
    padding: 0 0.2em;
    border-radius: 0.3em;
    cursor: help;
    user-select: none;
    color: #fff;
    background: #8a6d00;
  }
  .warning-hasStyle {
    background: #5b6ee1;
    font-weight: 700;
  }
  .warning-edgeWhitespace,
  .warning-hasTab {
    background: #8a6d00;
  }
  .warning-platformDependent,
  .warning-encodingNotice {
    background: #b05a00;
  }
  .warning-controlOrBinary {
    background: #c62828;
  }
  .warning-mixedNewlines {
    background: #7a4fa3;
  }
</style>
