/**
 * Generates THIRD-PARTY.md: every Rust crate and npm package the shipped app depends on,
 * with its licence (13.9).
 *
 * Both lists come from data that is already on disk after a normal build — `cargo metadata`
 * and the lockfile plus each package's own `package.json` — so the release workflow needs no
 * extra tooling, and the list is accurate for the platform it was generated on (the crate
 * graph differs between Windows, macOS and Linux).
 *
 * Usage: node scripts/third-party.mjs [output-path]
 */

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const UNKNOWN = 'UNKNOWN';

/** @typedef {{ name: string, version: string, license: string, url?: string }} Entry */

/** The triple cargo resolves dependencies for; crates for other platforms are not shipped. */
function hostTriple() {
  const out = execFileSync('rustc', ['-vV'], { encoding: 'utf8' });
  const host = /^host:\s*(\S+)$/m.exec(out);
  if (!host) throw new Error('could not read the host triple from `rustc -vV`');
  return host[1];
}

/**
 * Crates that end up in the binary: the normal-dependency closure of clipbuf, resolved for
 * this platform. Dev- and build-dependencies are followed nowhere, because a test harness or
 * a build script is not part of what is distributed.
 */
export function rustEntries() {
  const raw = execFileSync(
    'cargo',
    [
      'metadata',
      '--format-version',
      '1',
      '--locked',
      '--filter-platform',
      hostTriple(),
      '--manifest-path',
      join(ROOT, 'src-tauri/Cargo.toml'),
    ],
    { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  );
  const meta = JSON.parse(raw);
  const shipped = shippedCrateIds(meta);
  return meta.packages
    .filter((p) => shipped.has(p.id))
    .map((p) => ({
      name: p.name,
      version: p.version,
      license: p.license || (p.license_file ? 'see the licence file in the crate' : UNKNOWN),
      url: p.repository ?? undefined,
    }));
}

/** Breadth-first over normal dependency edges from the workspace roots, which are excluded. */
export function shippedCrateIds(meta) {
  const nodes = new Map((meta.resolve?.nodes ?? []).map((n) => [n.id, n]));
  const roots = meta.workspace_members ?? [];
  const shipped = new Set();
  const queue = [...roots];
  while (queue.length > 0) {
    const id = queue.shift();
    for (const dep of nodes.get(id)?.deps ?? []) {
      // `dep_kinds` is empty on very old cargo output; treat that as a normal dependency.
      const normal =
        dep.dep_kinds.length === 0 ||
        dep.dep_kinds.some((k) => k.kind === null || k.kind === undefined);
      if (!normal || shipped.has(dep.pkg)) continue;
      shipped.add(dep.pkg);
      queue.push(dep.pkg);
    }
  }
  for (const root of roots) shipped.delete(root);
  return shipped;
}

/**
 * Production npm packages. The lockfile says which ones are dev-only; the licence itself is
 * only stated in each package's own manifest.
 */
export function npmEntries() {
  const lock = JSON.parse(readFileSync(join(ROOT, 'package-lock.json'), 'utf8'));
  const entries = [];
  for (const [path, info] of Object.entries(lock.packages ?? {})) {
    if (path === '' || info.dev || info.devOptional) continue;
    const manifestPath = join(ROOT, path, 'package.json');
    if (!existsSync(manifestPath)) continue;
    const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
    entries.push({
      name: manifest.name ?? path.replace(/^node_modules\//, ''),
      version: manifest.version ?? info.version ?? '',
      license: normaliseNpmLicense(manifest) ?? UNKNOWN,
      url: repositoryUrl(manifest.repository),
    });
  }
  return entries;
}

function normaliseNpmLicense(manifest) {
  if (typeof manifest.license === 'string') return manifest.license;
  if (manifest.license?.type) return manifest.license.type;
  // The `licenses` array is deprecated but still present in older packages.
  if (Array.isArray(manifest.licenses)) {
    return manifest.licenses.map((l) => l.type ?? l).join(' OR ') || undefined;
  }
  return undefined;
}

function repositoryUrl(repository) {
  const url = typeof repository === 'string' ? repository : repository?.url;
  if (!url) return undefined;
  return url.replace(/^git\+/, '').replace(/\.git$/, '');
}

/** Sorted, de-duplicated and rendered as Markdown. */
export function render(rust, npm, generatedFor) {
  const lines = [
    '# Third-party licences',
    '',
    'clipbuf itself is MIT licensed (see LICENSE). It is built on the components below, each',
    'under its own licence. This list was generated from the dependency graph of the build for',
    `${generatedFor}.`,
    '',
  ];
  for (const [title, entries] of [
    ['Rust crates', rust],
    ['npm packages', npm],
  ]) {
    lines.push(`## ${title}`, '', '| Package | Version | Licence |', '| --- | --- | --- |');
    for (const entry of dedupe(entries)) {
      const name = entry.url ? `[${entry.name}](${entry.url})` : entry.name;
      lines.push(`| ${name} | ${entry.version} | ${entry.license} |`);
    }
    lines.push('');
  }
  return lines.join('\n');
}

/** One row per name+version; the same crate can be reached by several paths. */
export function dedupe(entries) {
  const seen = new Map();
  for (const entry of entries) seen.set(`${entry.name}@${entry.version}`, entry);
  return [...seen.values()].sort(
    (a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version),
  );
}

function main() {
  const out = process.argv[2] ?? join(ROOT, 'THIRD-PARTY.md');
  const document = render(rustEntries(), npmEntries(), `${process.platform}-${process.arch}`);
  writeFileSync(out, document);
  console.log(`wrote ${out}`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
