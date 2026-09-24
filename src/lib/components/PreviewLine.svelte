<script lang="ts">
  import { TOKEN_SYMBOL, newlineKind, tokenKey } from '$lib/preview/charset';
  import { tokenize } from '$lib/preview/tokenize';
  import { t } from '$lib/stores/i18n';

  interface Props {
    text: string;
  }

  let { text }: Props = $props();
  const result = $derived(tokenize(text));
</script>

<div
  class="preview"
  role="textbox"
  aria-readonly="true"
  aria-multiline="false"
  oncopy={(e) => e.preventDefault()}
  oncut={(e) => e.preventDefault()}
>
  {#each result.tokens as token, i (i)}
    {#if token.kind === 'text'}
      <span class="tok tok-text" data-kind="text">{token.value}</span>
    {:else}
      <span
        class="tok tok-{token.kind}"
        data-kind={token.kind}
        data-newline={token.kind === 'newline' ? newlineKind(token.value) : undefined}
        title={$t(tokenKey(token.kind, token.value))}
        aria-label={$t(tokenKey(token.kind, token.value))}>{TOKEN_SYMBOL[token.kind]}</span
      >
    {/if}
  {/each}
  {#if result.truncated}
    <span class="truncated" aria-label="truncated">{$t('list.truncated')}</span>
  {/if}
</div>

<style>
  /* Long text is simply clipped; the full text is read in the hover preview (4.1, 4.4). */
  .preview {
    flex: 1 1 auto;
    min-width: 0;
    height: 1.6em;
    line-height: 1.6em;
    overflow: hidden;
    white-space: nowrap;
    /* Fade the right edge so it is obvious that the line continues. */
    mask-image: linear-gradient(to right, #000 calc(100% - 2em), transparent 100%);
    font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace;
    font-size: 0.9em;
    user-select: none;
    -webkit-user-select: none;
    cursor: default;
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
  /* Newline conventions are told apart by colour (3.7). */
  .tok-newline {
    color: #fff;
    background: #7c3aed;
  }
  .tok-newline[data-newline='crlf'] {
    background: #7c3aed;
  }
  .tok-newline[data-newline='lf'] {
    background: #0d9488;
  }
  .tok-newline[data-newline='cr'] {
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
  .truncated {
    color: #9aa0a6;
    padding-left: 0.3em;
  }
</style>
