/**
 * What the about window needs from the backend (8.8): the running version, the repository
 * link, and closing itself. Wrapped like the other IPC so the component can be tested
 * without Tauri.
 */

import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import { commands } from './commands';

/** Where the source, the licence and the releases live (13.1). */
export const REPOSITORY_URL = 'https://github.com/YasuhiroInagawa/clipbuf';

/** The MIT copyright holder, as named in LICENSE. */
export const AUTHOR = 'Yasuhiro Inagawa';

export interface AboutApi {
  getVersion(): Promise<string>;
  /** Opens the repository in the user's browser, not in a clipbuf window. */
  openRepository(): Promise<void>;
  close(): Promise<void>;
}

export const aboutApi: AboutApi = {
  getVersion: () => getVersion(),
  openRepository: () => openUrl(REPOSITORY_URL),
  close: () => commands.closeAbout(),
};
