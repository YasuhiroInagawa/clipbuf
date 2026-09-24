<script lang="ts">
  import type { TransferPreview } from '$lib/ipc/types';
  import {
    TOKEN_SYMBOL,
    newlineKind,
    newlineKindsIn,
    tokenKey,
    type NewlineKind,
  } from '$lib/preview/charset';
  import { tokenize, type PreviewToken } from '$lib/preview/tokenize';
  import { t } from '$lib/stores/i18n';

  interface Props {
    /** Fetches the text a transfer would produce with the current options (4.7). */
    load: () => Promise<TransferPreview>;
    /** Row element to position against; `null` renders without positioning (tests).
     *  Named `anchorEl` because `anchor` collides with a testing-library mount option. */
    anchorEl: HTMLElement | null;
  }

  let { load, anchorEl }: Props = $props();

  let preview: TransferPreview | null = $state(null);
  let box: HTMLDivElement | undefined = $state();

  /**
   * Tokens grouped into display lines: a newline token ends its line but stays visible, so the
   * reader sees both where the break is and which convention produced it (4.6).
   */
  const rendered = $derived.by((): { lines: PreviewToken[][]; truncated: boolean } => {
    if (preview === null) return { lines: [], truncated: false };
    const result = tokenize(preview.text);
    const lines: PreviewToken[][] = [[]];
    for (const token of result.tokens) {
      lines[lines.length - 1].push(token);
      // A trailing newline leaves an empty last line; keep it, it is real content.
      if (token.kind === 'newline') lines.push([]);
    }
    return { lines, truncated: result.truncated };
  });

  /** Newline conventions in the previewed text, for the colour legend (3.9). */
  const legend = $derived.by((): NewlineKind[] =>
    preview === null ? [] : newlineKindsIn(preview.text),
  );

  $effect(() => {
    let cancelled = false;
    void load().then((p) => {
      if (!cancelled) preview = p;
    });
    return () => {
      cancelled = true;
    };
  });

  /** Keep the popover inside the window, preferring just below the row. */
  $effect(() => {
    if (!box || !anchorEl) return;
    const rect = anchorEl.getBoundingClientRect();
    const size = box.getBoundingClientRect();
    const margin = 8;
    const left = Math.max(
      margin,
      Math.min(rect.left, document.documentElement.clientWidth - size.width - margin),
    );
    const below = rect.bottom + 4;
    const top =
      below + size.height + margin <= document.documentElement.clientHeight
        ? below
        : Math.max(margin, rect.top - size.height - 4);
    box.style.left = `${left}px`;
    box.style.top = `${top}px`;
  });
</script>

{#if preview}
  <div class="pv" bind:this={box} role="tooltip">
    {#if preview.text === ''}
      <p class="pv-empty">{$t('preview.empty')}</p>
    {:else}
      <div class="pv-body">
        {#each rendered.lines as line, i (i)}
          <div class="pv-line">
            {#each line as token, j (j)}
              {#if token.kind === 'text'}
                <span class="tok tok-text" data-kind="text">{token.value}</span>
              {:else}
                <span
                  class="tok tok-{token.kind}"
                  data-kind={token.kind}
                  data-newline={token.kind === 'newline' ? newlineKind(token.value) : undefined}
                  title={$t(tokenKey(token.kind, token.value))}
                  aria-label={$t(tokenKey(token.kind, token.value))}
                  >{TOKEN_SYMBOL[token.kind]}</span
                >
              {/if}
            {/each}
          </div>
        {/each}
      </div>
    {/if}
    {#if legend.length > 0}
      <p class="pv-legend">
        <span class="pv-legend-label">{$t('preview.newlines')}</span>
        {#each legend as kind (kind)}
          <span class="pv-legend-item">
            <span class="tok tok-newline" data-newline={kind} aria-hidden="true">↵</span>
            {kind.toUpperCase()}
          </span>
        {/each}
      </p>
    {/if}
    {#if rendered.truncated}
      <p class="pv-note">{$t('preview.truncated')}</p>
    {/if}
    {#if preview.skippedTransforms}
      <p class="pv-note">{$t('notice.transformsSkipped')}</p>
    {/if}
  </div>
{/if}

<style>
  .pv {
    position: fixed;
    z-index: 10;
    max-width: min(40em, 90vw);
    max-height: 50vh;
    overflow: auto;
    padding: 0.4em 0.6em;
    border-radius: 6px;
    border: 1px solid color-mix(in srgb, currentColor 25%, transparent);
    background: Canvas;
    color: CanvasText;
    box-shadow: 0 6px 24px rgb(0 0 0 / 35%);
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.85em;
    line-height: 1.5;
    user-select: none;
    pointer-events: none;
  }
  .pv-body {
    white-space: pre;
  }
  .pv-line {
    min-height: 1.5em;
  }
  .pv-legend {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.15em 0.6em;
    margin: 0.4em 0 0;
    padding-top: 0.35em;
    border-top: 1px solid color-mix(in srgb, currentColor 18%, transparent);
    font-family: system-ui, sans-serif;
    font-size: 0.8em;
    opacity: 0.85;
  }
  .pv-legend-label {
    opacity: 0.7;
  }
  .pv-legend-item {
    display: inline-flex;
    align-items: center;
    gap: 0.3em;
    font-family: ui-monospace, monospace;
  }
  .pv-empty,
  .pv-note {
    margin: 0.3em 0 0;
    font-family: system-ui, sans-serif;
    font-size: 0.9em;
    opacity: 0.75;
  }
  .pv-empty {
    margin: 0;
    font-style: italic;
  }
  .tok-space {
    color: #9aa0a6;
  }
  .tok-fullwidthSpace {
    color: #fff;
    background: #d97706;
  }
  .tok-tab {
    color: #fff;
    background: #2563eb;
  }
  .tok-newline[data-newline='crlf'] {
    color: #fff;
    background: #7c3aed;
  }
  .tok-newline[data-newline='lf'] {
    color: #fff;
    background: #0d9488;
  }
  .tok-newline[data-newline='cr'] {
    color: #fff;
    background: #db2777;
  }
  .tok-nbsp {
    color: #fff;
    background: #0891b2;
  }
  .tok-zeroWidth,
  .tok-bidi {
    color: #fff;
    background: #be185d;
  }
  .tok-control {
    color: #fff;
    background: #dc2626;
  }
</style>
