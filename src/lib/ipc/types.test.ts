import { describe, expect, it } from 'vitest';
import contract from '../../../tests/fixtures/contract.json';
import {
  CAPTURE_CAPABILITIES,
  CAPTURE_STATUSES,
  ERROR_KINDS,
  EVENT,
  NEWLINE_MODES,
  TRANSFER_MODES,
  WARNINGS,
  isAppError,
  type ItemDto,
  type PlatformInfo,
  type Settings,
  type TransferOptions,
  type TransferOutcome,
} from './types';

// The fixture is the single source of truth shared with the Rust contract tests
// (src-tauri/src/model/contract_test.rs). Enum members are checked at runtime against the
// exported const arrays, and object shapes against the expected key sets, so drift on either
// side fails here.

const keysOf = (value: object): string[] => Object.keys(value).sort();

function expectTransferOptions(value: TransferOptions): void {
  expect(keysOf(value)).toEqual([
    'fullwidthToSpace',
    'keepStyle',
    'newline',
    'tabsToSpaces',
    'trim',
  ]);
  expect(NEWLINE_MODES).toContain(value.newline);
  expect(typeof value.keepStyle).toBe('boolean');
  expect(typeof value.trim).toBe('boolean');
  expect(typeof value.tabsToSpaces).toBe('boolean');
  expect(typeof value.fullwidthToSpace).toBe('boolean');
}

function expectSettings(value: Settings): void {
  expect(keysOf(value)).toEqual([
    'autostart',
    'capacity',
    'hotkey',
    'language',
    'pollIntervalMs',
    'tabWidth',
    'transfer',
    'version',
  ]);
  expect(typeof value.version).toBe('number');
  expect(typeof value.capacity).toBe('number');
  expect(typeof value.hotkey).toBe('string');
  expect(typeof value.tabWidth).toBe('number');
  expect(typeof value.pollIntervalMs).toBe('number');
  expect(typeof value.autostart).toBe('boolean');
  expect([null, 'ja', 'en']).toContain(value.language);
  expectTransferOptions(value.transfer);
}

describe('IPC type contract', () => {
  it('enumerates the same variants as the fixture', () => {
    expect(contract.warnings).toEqual(WARNINGS);
    expect(contract.newlineModes).toEqual(NEWLINE_MODES);
    expect(contract.transferModes).toEqual(TRANSFER_MODES);
    expect(contract.errorKinds).toEqual(ERROR_KINDS);
    expect(contract.captureCapabilities).toEqual(CAPTURE_CAPABILITIES);
    expect(contract.captureStatuses).toEqual(CAPTURE_STATUSES);
  });

  it('matches ItemDto', () => {
    const dto = contract.itemDto as ItemDto;
    expect(keysOf(dto)).toEqual(['capturedAtMs', 'hasStyle', 'id', 'text', 'warnings']);
    expect(typeof dto.id).toBe('number');
    expect(typeof dto.capturedAtMs).toBe('number');
    expect(typeof dto.text).toBe('string');
    expect(typeof dto.hasStyle).toBe('boolean');
    for (const warning of dto.warnings) expect(WARNINGS).toContain(warning);
  });

  it('matches Settings, TransferOptions and the default settings', () => {
    expectTransferOptions(contract.transferOptions as TransferOptions);
    expectSettings(contract.settings as Settings);
    expectSettings(contract.defaultSettings as Settings);
    expect(contract.defaultSettings).toEqual({
      version: 1,
      capacity: 20,
      hotkey: 'Alt+Shift+V',
      tabWidth: 4,
      pollIntervalMs: 200,
      autostart: false,
      language: null,
      transfer: {
        keepStyle: false,
        newline: 'keep',
        trim: false,
        tabsToSpaces: false,
        fullwidthToSpace: false,
      },
    });
  });

  it('matches TransferOutcome, AppError and PlatformInfo', () => {
    const outcome = contract.transferOutcome as TransferOutcome;
    expect(keysOf(outcome)).toEqual(['skippedTransforms']);
    expect(typeof outcome.skippedTransforms).toBe('boolean');

    expect(isAppError(contract.appError)).toBe(true);
    expect(ERROR_KINDS).toContain(contract.appError.kind);
    expect(isAppError({ message: 'not an AppError' })).toBe(false);
    expect(isAppError(null)).toBe(false);

    const info = contract.platformInfo as PlatformInfo;
    expect(keysOf(info)).toEqual(['capture', 'displayServer', 'os']);
    expect(CAPTURE_CAPABILITIES).toContain(info.capture);
  });

  it('uses the same event names as the Rust side', () => {
    expect(EVENT).toEqual(contract.events);
  });
});
