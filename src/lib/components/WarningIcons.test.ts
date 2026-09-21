/** @vitest-environment jsdom */
import { render, screen } from '@testing-library/svelte';
import { beforeAll, describe, expect, it } from 'vitest';
import en from '../../locales/en.json';
import { i18n } from '$lib/stores/i18n';
import WarningIcons from './WarningIcons.svelte';

beforeAll(() => i18n.setLanguage('en'));

describe('WarningIcons', () => {
  it('renders nothing when there are no warnings (5.9)', () => {
    const { container } = render(WarningIcons, { warnings: [] });
    expect(container.querySelectorAll('[data-warning]')).toHaveLength(0);
  });

  it('renders only the given warnings, in the fixed display order', () => {
    const { container } = render(WarningIcons, {
      warnings: ['mixedNewlines', 'hasStyle', 'hasTab'],
    });
    const kinds = [...container.querySelectorAll('[data-warning]')].map((el) =>
      el.getAttribute('data-warning'),
    );
    expect(kinds).toEqual(['hasStyle', 'hasTab', 'mixedNewlines']);
  });

  it('exposes the i18n label and hint for hover / assistive tech (5.8)', () => {
    render(WarningIcons, { warnings: ['platformDependent'] });
    const icon = screen.getByLabelText(en['warning.platformDependent']);
    expect(icon).toHaveAttribute('title', en['warning.platformDependent.hint']);
  });

  it('switches text when the language changes', async () => {
    i18n.setLanguage('ja');
    try {
      render(WarningIcons, { warnings: ['hasTab'] });
      expect(screen.getByLabelText('タブあり')).toBeInTheDocument();
    } finally {
      i18n.setLanguage('en');
    }
  });
});
