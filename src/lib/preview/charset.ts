/**
 * Classification of whitespace and invisible characters for the preview line.
 *
 * Mirrors `src-tauri/src/analysis/charset.rs`; both are checked against
 * `tests/fixtures/charset.json`. Keep the two in sync.
 */

export type TokenKind =
  | 'text'
  | 'space'
  | 'fullwidthSpace'
  | 'tab'
  | 'newline'
  | 'nbsp'
  | 'zeroWidth'
  | 'bidi'
  | 'control';

/** Display symbol for each non-text class (design §analysis charset table). */
export const TOKEN_SYMBOL: Readonly<Record<Exclude<TokenKind, 'text'>, string>> = {
  space: '·',
  fullwidthSpace: '□',
  tab: '→',
  newline: '↵',
  nbsp: '⍽',
  zeroWidth: '∅',
  bidi: '⇄',
  control: '�',
};

/** Which newline convention a `newline` token holds (3.7). */
export type NewlineKind = 'crlf' | 'lf' | 'cr';

export function newlineKind(value: string): NewlineKind {
  if (value === '\r\n') return 'crlf';
  return value === '\r' ? 'cr' : 'lf';
}

/** Newline conventions present in `text`, in CRLF → LF → CR order. */
export function newlineKindsIn(text: string): NewlineKind[] {
  const seen = new Set<NewlineKind>();
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '\r') {
      if (text[i + 1] === '\n') {
        seen.add('crlf');
        i += 1;
      } else {
        seen.add('cr');
      }
    } else if (c === '\n') {
      seen.add('lf');
    }
  }
  return (['crlf', 'lf', 'cr'] as const).filter((k) => seen.has(k));
}

/** i18n key naming a non-text token; newlines resolve to their convention (3.8). */
export function tokenKey(kind: Exclude<TokenKind, 'text'>, value: string): string {
  return kind === 'newline' ? `token.newline.${newlineKind(value)}` : `token.${kind}`;
}

function isBidi(code: number): boolean {
  return (
    code === 0x200e ||
    code === 0x200f ||
    (code >= 0x202a && code <= 0x202e) ||
    (code >= 0x2066 && code <= 0x2069)
  );
}

function isControl(code: number): boolean {
  return (
    code <= 0x08 ||
    code === 0x0b ||
    code === 0x0c ||
    (code >= 0x0e && code <= 0x1f) ||
    (code >= 0x7f && code <= 0x9f) ||
    code === 0xfffd
  );
}

/** Classify one code point (a single-character string). `null` means ordinary visible text. */
export function classify(char: string): Exclude<TokenKind, 'text'> | null {
  const code = char.codePointAt(0);
  if (code === undefined) return null;
  switch (code) {
    case 0x20:
      return 'space';
    case 0x3000:
      return 'fullwidthSpace';
    case 0x09:
      return 'tab';
    case 0x0a:
    case 0x0d:
      return 'newline';
    case 0xa0:
    case 0x202f:
      return 'nbsp';
    case 0x200b:
    case 0x200c:
    case 0x200d:
    case 0x2060:
    case 0xfeff:
      return 'zeroWidth';
    default:
      if (isBidi(code)) return 'bidi';
      if (isControl(code)) return 'control';
      return null;
  }
}
