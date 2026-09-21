<script lang="ts">
  import { TOKEN_SYMBOL, type TokenKind } from '$lib/preview/charset';
  import { tokenize } from '$lib/preview/tokenize';
  import { t } from '$lib/stores/i18n';

  interface Props {
    text: string;
  }

  let { text }: Props = $props();
  let line: HTMLDivElement | undefined = $state();

  const result = $derived(tokenize(text));

  /** Accessible name for a non-text token. */
  const LABEL: Record<Exclude<TokenKind, 'text'>, string> = {
    space: 'space',
    fullwidthSpace: 'full-width space',
    tab: 'tab',
    newline: 'line break',
    nbsp: 'no-break space',
    zeroWidth: 'zero-width character',
    bidi: 'bidirectional control',
    control: 'control character',
  };

  function resetScroll(): void {
    if (line) line.scrollLeft = 0;
  }

  /** Mouse users scroll on hover; put the line back unless it keeps keyboard focus. */
  function onMouseLeave(): void {
    if (line && document.activeElement !== line) resetScroll();
  }
</script>

<div
  class="preview"
  bind:this={line}
  role="textbox"
  aria-readonly="true"
  aria-multiline="false"
  tabindex="0"
  oncopy={(e) => e.preventDefault()}
  oncut={(e) => e.preventDefault()}
  onfocusout={resetScroll}
  onmouseleave={onMouseLeave}
>
  {#each result.tokens as token, i (i)}
    {#if token.kind === 'text'}
      <span class="tok tok-text" data-kind="text">{token.value}</span>
    {:else}
      <span class="tok tok-{token.kind}" data-kind={token.kind} aria-label={LABEL[token.kind]}
        >{TOKEN_SYMBOL[token.kind]}</span
      >
    {/if}
  {/each}
  {#if result.truncated}
    <span class="truncated" aria-label="truncated">{$t('list.truncated')}</span>
  {/if}
</div>

<style>
  .preview {
    flex: 1 1 auto;
    min-width: 0;
    height: 1.6em;
    line-height: 1.6em;
    overflow-x: auto;
    overflow-y: hidden;
    white-space: nowrap;
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.9em;
    user-select: none;
    -webkit-user-select: none;
    cursor: default;
    outline: none;
    scrollbar-width: thin;
  }
  .preview:focus-visible {
    box-shadow: inset 0 0 0 1px #5b6ee1;
    border-radius: 3px;
  }
  .tok {
    white-space: pre;
  }
  .tok-space,
  .tok-fullwidthSpace,
  .tok-tab,
  .tok-newline,
  .tok-nbsp,
  .tok-zeroWidth,
  .tok-bidi,
  .tok-control {
    opacity: 0.85;
    border-radius: 2px;
    padding: 0 0.05em;
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
  .tok-newline {
    color: #fff;
    background: #7c3aed;
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
  .truncated {
    color: #9aa0a6;
    padding-left: 0.3em;
  }
</style>
