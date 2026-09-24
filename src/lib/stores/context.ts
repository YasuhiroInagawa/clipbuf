/**
 * Everything the main window needs, bundled so components receive one prop and tests can
 * supply fakes. `createMainContext` wires the stores to an API object; the app passes the
 * real `commands` + `events`, tests pass in-memory fakes.
 */

import type { UnlistenFn } from '$lib/ipc/events';
import type {
  CaptureStatus,
  ItemDto,
  ItemId,
  PlatformInfo,
  Settings,
  TransferMode,
  TransferOutcome,
  TransferPreview,
} from '$lib/ipc/types';
import { createItemsStore, type ItemsStore } from './items';
import { createNoticeStore, type NoticeStore } from './notice';
import { createSelectionStore, type SelectionStore } from './selection';
import { createSettingsStore, type SettingsStore } from './settings';

export interface MainApi {
  listItems(): Promise<ItemDto[]>;
  onItemAdded(cb: (item: ItemDto) => void): Promise<UnlistenFn>;
  onItemsChanged(cb: (items: ItemDto[]) => void): Promise<UnlistenFn>;
  getSettings(): Promise<Settings>;
  setSettings(settings: Settings): Promise<Settings>;
  onSettingsChanged(cb: (settings: Settings) => void): Promise<UnlistenFn>;
  transferItem(id: ItemId, mode: TransferMode): Promise<TransferOutcome>;
  previewTransfer(id: ItemId): Promise<TransferPreview>;
  removeItem(id: ItemId): Promise<void>;
  clearItems(): Promise<void>;
  hideWindow(): Promise<void>;
  getPlatformInfo(): Promise<PlatformInfo>;
  onCaptureStatus(cb: (status: CaptureStatus) => void): Promise<UnlistenFn>;
  onWindowShown(cb: () => void): Promise<UnlistenFn>;
}

export interface MainContext {
  api: MainApi;
  items: ItemsStore;
  selection: SelectionStore;
  settings: SettingsStore;
  notices: NoticeStore;
}

export function createMainContext(
  api: MainApi,
  notices: NoticeStore = createNoticeStore(),
): MainContext {
  const items = createItemsStore(api);
  return {
    api,
    items,
    selection: createSelectionStore(items.items),
    settings: createSettingsStore(api),
    notices,
  };
}
