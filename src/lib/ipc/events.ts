/**
 * Typed wrappers over Tauri `listen` for the events emitted by the Rust core
 * (`src-tauri/src/app/events.rs`). See `commands.ts` for the import rule.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { EVENT, type CaptureStatus, type ItemDto, type Settings } from './types';

export type { UnlistenFn };

export function onItemAdded(cb: (item: ItemDto) => void): Promise<UnlistenFn> {
  return listen<ItemDto>(EVENT.itemAdded, (e) => cb(e.payload));
}

export function onItemsChanged(cb: (items: ItemDto[]) => void): Promise<UnlistenFn> {
  return listen<ItemDto[]>(EVENT.itemsChanged, (e) => cb(e.payload));
}

export function onSettingsChanged(cb: (settings: Settings) => void): Promise<UnlistenFn> {
  return listen<Settings>(EVENT.settingsChanged, (e) => cb(e.payload));
}

export function onWindowShown(cb: () => void): Promise<UnlistenFn> {
  return listen<null>(EVENT.windowShown, () => cb());
}

export function onCaptureStatus(cb: (status: CaptureStatus) => void): Promise<UnlistenFn> {
  return listen<CaptureStatus>(EVENT.captureStatus, (e) => cb(e.payload));
}

export const events = {
  onItemAdded,
  onItemsChanged,
  onSettingsChanged,
  onWindowShown,
  onCaptureStatus,
} as const;
