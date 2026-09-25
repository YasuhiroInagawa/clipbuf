/**
 * Typed wrappers over Tauri `invoke`. This module (and `events.ts`) are the only places
 * that import `@tauri-apps/api`; components go through `commands` so the IPC surface stays
 * in one file and can be faked in tests.
 *
 * Rejections are `AppError` objects (`{ kind }`) when the backend refused the call; anything
 * else is passed through unchanged.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  ItemDto,
  ItemId,
  PlatformInfo,
  Settings,
  TransferMode,
  TransferOutcome,
  TransferPreview,
} from './types';

export const commands = {
  listItems: (): Promise<ItemDto[]> => invoke('list_items'),
  transferItem: (id: ItemId, mode: TransferMode): Promise<TransferOutcome> =>
    invoke('transfer_item', { id, mode }),
  previewTransfer: (id: ItemId): Promise<TransferPreview> => invoke('preview_transfer', { id }),
  removeItem: (id: ItemId): Promise<void> => invoke('remove_item', { id }),
  clearItems: (): Promise<void> => invoke('clear_items'),
  getSettings: (): Promise<Settings> => invoke('get_settings'),
  setSettings: (settings: Settings): Promise<Settings> => invoke('set_settings', { settings }),
  getPlatformInfo: (): Promise<PlatformInfo> => invoke('get_platform_info'),
  hideWindow: (): Promise<void> => invoke('hide_window'),
  openSettings: (): Promise<void> => invoke('open_settings'),
  closeSettings: (): Promise<void> => invoke('close_settings'),
  closeAbout: (): Promise<void> => invoke('close_about'),
  suspendHotkey: (): Promise<void> => invoke('suspend_hotkey'),
  resumeHotkey: (): Promise<void> => invoke('resume_hotkey'),
} as const;

export type Commands = typeof commands;
