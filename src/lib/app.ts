/** Application wiring: real IPC into the stores. Components never import this; App does. */
import { commands } from '$lib/ipc/commands';
import { events } from '$lib/ipc/events';
import { createMainContext, type MainContext } from '$lib/stores/context';
import { notices } from '$lib/stores/notice';

export const appName = 'clipbuf';

export function createAppContext(): MainContext {
  return createMainContext({ ...commands, ...events }, notices);
}
