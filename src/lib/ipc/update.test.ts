import { readFileSync, readdirSync } from 'node:fs';
import { join, relative } from 'node:path';
import { describe, expect, it } from 'vitest';

const SRC = new URL('../../', import.meta.url).pathname;

function sources(): string[] {
  const out: string[] = [];
  const walk = (dir: string): void => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) walk(path);
      else if (/\.(ts|svelte)$/.test(entry.name) && !entry.name.endsWith('.test.ts'))
        out.push(path);
    }
  };
  walk(SRC);
  return out;
}

/**
 * The update check is the app's only network access (10.3, 10.4). These guards are cheap and
 * would catch a later change that starts talking to the network from somewhere else.
 */
describe('network access', () => {
  it('scans the whole frontend, so the guards below cannot pass vacuously', () => {
    expect(sources().length).toBeGreaterThan(20);
  });

  it('never calls the network directly (10.3, 10.4)', () => {
    const offenders = sources().filter((path) =>
      /\bfetch\s*\(|XMLHttpRequest|new WebSocket|navigator\.sendBeacon/.test(
        readFileSync(path, 'utf8'),
      ),
    );
    expect(offenders.map((p) => relative(SRC, p))).toEqual([]);
  });

  it('reaches the updater only through lib/ipc/update.ts (10.4)', () => {
    const importers = sources()
      .filter((path) => /@tauri-apps\/plugin-(updater|process)/.test(readFileSync(path, 'utf8')))
      .map((p) => relative(SRC, p));
    expect(importers).toEqual(['lib/ipc/update.ts']);
  });
});
