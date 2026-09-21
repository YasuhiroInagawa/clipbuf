import { describe, expect, it } from 'vitest';
import { appName } from './app';

describe('app', () => {
  it('exposes the application name', () => {
    expect(appName).toBe('clipbuf');
  });
});
