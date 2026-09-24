/**
 * IPC type contract shared with the Rust core (`src-tauri/src/model/`).
 *
 * The wire format is camelCase JSON. `tests/fixtures/contract.json` pins the representation
 * and is checked by both `types.test.ts` and the Rust contract tests, so a change on either
 * side that is not mirrored fails a test.
 */

export type ItemId = number;

/** Paste-hazard warnings, in display order (requirements 5.1–5.7). */
export type Warning =
  | 'hasStyle'
  | 'edgeWhitespace'
  | 'hasTab'
  | 'platformDependent'
  | 'controlOrBinary'
  | 'mixedNewlines'
  | 'encodingNotice';

export const WARNINGS: readonly Warning[] = [
  'hasStyle',
  'edgeWhitespace',
  'hasTab',
  'platformDependent',
  'controlOrBinary',
  'mixedNewlines',
  'encodingNotice',
];

export type NewlineMode = 'keep' | 'remove' | 'space';
export const NEWLINE_MODES: readonly NewlineMode[] = ['keep', 'remove', 'space'];

export type TransferMode = 'options' | 'plain' | 'raw';
export const TRANSFER_MODES: readonly TransferMode[] = ['options', 'plain', 'raw'];

export interface TransferOptions {
  keepStyle: boolean;
  newline: NewlineMode;
  trim: boolean;
  tabsToSpaces: boolean;
  fullwidthToSpace: boolean;
}

export type Language = 'ja' | 'en';

export interface Settings {
  version: number;
  capacity: number;
  hotkey: string;
  tabWidth: number;
  pollIntervalMs: number;
  autostart: boolean;
  /** `null` follows the OS language. */
  language: Language | null;
  transfer: TransferOptions;
}

export interface ItemDto {
  id: ItemId;
  capturedAtMs: number;
  /** Full text; the preview truncates for display. */
  text: string;
  hasStyle: boolean;
  warnings: Warning[];
}

export interface TransferOutcome {
  skippedTransforms: boolean;
}

/** Text a transfer would place on the clipboard, for the full-text preview (4.7). */
export interface TransferPreview {
  text: string;
  skippedTransforms: boolean;
}

export type ErrorKind =
  | 'itemNotFound'
  | 'writeFailed'
  | 'readFailed'
  | 'hotkeyUnavailable'
  | 'invalidSettings'
  | 'captureUnavailable'
  | 'settingsIo';

export const ERROR_KINDS: readonly ErrorKind[] = [
  'itemNotFound',
  'writeFailed',
  'readFailed',
  'hotkeyUnavailable',
  'invalidSettings',
  'captureUnavailable',
  'settingsIo',
];

export interface AppError {
  kind: ErrorKind;
}

export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === 'object' &&
    value !== null &&
    'kind' in value &&
    typeof (value as { kind: unknown }).kind === 'string'
  );
}

export type CaptureCapability = 'full' | 'limitedXWayland' | 'unavailable' | 'denied';
export const CAPTURE_CAPABILITIES: readonly CaptureCapability[] = [
  'full',
  'limitedXWayland',
  'unavailable',
  'denied',
];

export type CaptureStatus = CaptureCapability | 'readFailed';
export const CAPTURE_STATUSES: readonly CaptureStatus[] = [...CAPTURE_CAPABILITIES, 'readFailed'];

export interface PlatformInfo {
  os: string;
  displayServer: string;
  capture: CaptureCapability;
}

/** Event names emitted by the Rust core; must match `src-tauri/src/app/events.rs`. */
export const EVENT = {
  itemAdded: 'clipbuf://item-added',
  itemsChanged: 'clipbuf://items-changed',
  settingsChanged: 'clipbuf://settings-changed',
  windowShown: 'clipbuf://window-shown',
  captureStatus: 'clipbuf://capture-status',
} as const;

/** Payload carried by each event. */
export interface EventPayload {
  [EVENT.itemAdded]: ItemDto;
  [EVENT.itemsChanged]: ItemDto[];
  [EVENT.settingsChanged]: Settings;
  [EVENT.windowShown]: null;
  [EVENT.captureStatus]: CaptureStatus;
}
