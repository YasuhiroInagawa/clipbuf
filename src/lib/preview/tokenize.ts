/**
 * Turns item text into display tokens for the one-line preview (requirements 3.2, 3.4–3.6).
 *
 * Plain characters are grouped into `text` tokens; every whitespace / invisible character
 * becomes its own token so the component can render a symbol for it. CRLF is one `newline`
 * token. The data itself is never altered: joining `tokens[].value` reproduces the input
 * (up to the truncation point).
 */

import { classify, type TokenKind } from './charset';

export interface PreviewToken {
  kind: TokenKind;
  value: string;
}

export interface TokenizeResult {
  tokens: PreviewToken[];
  /** True when the input exceeded `PREVIEW_LIMIT` code points and was cut. */
  truncated: boolean;
}

/** Maximum number of code points rendered for one item. */
export const PREVIEW_LIMIT = 20_000;

export function tokenize(text: string): TokenizeResult {
  const tokens: PreviewToken[] = [];
  let pending = '';
  let count = 0;
  let truncated = false;

  const flush = (): void => {
    if (pending.length > 0) {
      tokens.push({ kind: 'text', value: pending });
      pending = '';
    }
  };

  const chars = Array.from(text);
  for (let i = 0; i < chars.length; i++) {
    if (count >= PREVIEW_LIMIT) {
      truncated = true;
      break;
    }
    const char = chars[i];
    const kind = classify(char);
    if (kind === null) {
      pending += char;
      count += 1;
      continue;
    }
    flush();
    // CRLF is one newline token; it counts as one code point toward the limit so the
    // pair is never split at the truncation boundary.
    if (char === '\r' && chars[i + 1] === '\n') {
      tokens.push({ kind: 'newline', value: '\r\n' });
      i += 1;
    } else {
      tokens.push({ kind, value: char });
    }
    count += 1;
  }
  flush();

  return { tokens, truncated };
}
