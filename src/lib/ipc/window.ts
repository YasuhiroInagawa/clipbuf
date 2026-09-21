/** Which Tauri window this frontend instance runs in (`main` or `settings`). */
import { getCurrentWindow } from '@tauri-apps/api/window';

export function currentWindowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    // Not inside Tauri (e.g. plain Vite in a browser): behave as the main window.
    return 'main';
  }
}
