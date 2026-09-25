import { describe, expect, it } from 'vitest';
import { dedupe, render, shippedCrateIds } from './third-party.mjs';

describe('shippedCrateIds', () => {
  const meta = {
    workspace_members: ['root'],
    resolve: {
      nodes: [
        {
          id: 'root',
          deps: [
            { pkg: 'lib', dep_kinds: [{ kind: null }] },
            { pkg: 'test-only', dep_kinds: [{ kind: 'dev' }] },
            { pkg: 'build-only', dep_kinds: [{ kind: 'build' }] },
          ],
        },
        { id: 'lib', deps: [{ pkg: 'deep', dep_kinds: [{ kind: null }] }] },
        { id: 'deep', deps: [] },
        { id: 'test-only', deps: [{ pkg: 'never', dep_kinds: [{ kind: null }] }] },
        { id: 'build-only', deps: [] },
      ],
    },
  };

  it('follows normal dependencies all the way down', () => {
    expect(shippedCrateIds(meta)).toEqual(new Set(['lib', 'deep']));
  });

  it('leaves out what is not distributed: dev- and build-dependencies, and their own deps', () => {
    const shipped = shippedCrateIds(meta);
    for (const id of ['test-only', 'build-only', 'never']) expect(shipped.has(id)).toBe(false);
  });

  it('excludes the workspace crates themselves', () => {
    expect(shippedCrateIds(meta).has('root')).toBe(false);
  });

  it('survives a dependency cycle', () => {
    const cyclic = {
      workspace_members: ['a'],
      resolve: {
        nodes: [
          { id: 'a', deps: [{ pkg: 'b', dep_kinds: [{ kind: null }] }] },
          { id: 'b', deps: [{ pkg: 'a', dep_kinds: [{ kind: null }] }] },
        ],
      },
    };
    expect(shippedCrateIds(cyclic)).toEqual(new Set(['b']));
  });
});

describe('dedupe', () => {
  it('keeps one row per name and version, sorted', () => {
    const entries = [
      { name: 'b', version: '1.0.0', license: 'MIT' },
      { name: 'a', version: '2.0.0', license: 'MIT' },
      { name: 'a', version: '1.0.0', license: 'MIT' },
      { name: 'a', version: '1.0.0', license: 'MIT' },
    ];
    expect(dedupe(entries).map((e) => `${e.name}@${e.version}`)).toEqual([
      'a@1.0.0',
      'a@2.0.0',
      'b@1.0.0',
    ]);
  });
});

describe('render', () => {
  const doc = render(
    [
      {
        name: 'serde',
        version: '1.0.0',
        license: 'MIT OR Apache-2.0',
        url: 'https://example.test',
      },
    ],
    [{ name: 'svelte', version: '5.0.0', license: 'MIT' }],
    'darwin-arm64',
  );

  it('states clipbuf own licence and names both ecosystems (13.9)', () => {
    expect(doc).toContain('clipbuf itself is MIT licensed');
    expect(doc).toContain('## Rust crates');
    expect(doc).toContain('## npm packages');
  });

  it('lists every dependency with its licence', () => {
    expect(doc).toContain('| [serde](https://example.test) | 1.0.0 | MIT OR Apache-2.0 |');
    expect(doc).toContain('| svelte | 5.0.0 | MIT |');
  });

  it('records which platform the list was generated for', () => {
    expect(doc).toContain('darwin-arm64');
  });
});
