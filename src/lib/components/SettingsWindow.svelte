<script lang="ts">
  import { onMount } from 'svelte';
  import {
    SETTINGS_RANGES,
    isAppError,
    type Language,
    type PlatformInfo,
    type RangedSettingsField,
    type Settings,
  } from '$lib/ipc/types';
  import { i18n, resolveInitialLanguage, t } from '$lib/stores/i18n';
  import type { SettingsStore } from '$lib/stores/settings';

  interface Props {
    store: SettingsStore;
    getPlatformInfo: () => Promise<PlatformInfo>;
    /** Called after a successful save; the window closes like an OK button (9.7). */
    closeWindow: () => Promise<void>;
    /** Silences the active global hotkey while recording, so it cannot swallow the keys (9.5.1). */
    suspendHotkey: () => Promise<void>;
    resumeHotkey: () => Promise<void>;
  }

  let { store, getPlatformInfo, closeWindow, suspendHotkey, resumeHotkey }: Props = $props();
  const settings = $derived(store.settings);

  /** Editable copy; only written back on save, so a rejected save keeps the input (9.4). */
  let draft: Settings | null = $state(null);
  let platform: PlatformInfo | null = $state(null);
  let recording = $state(false);
  let fieldErrors: Partial<Record<RangedSettingsField | 'hotkey', string>> = $state({});
  let saveError: string | null = $state(null);

  onMount(() => {
    // The window can be closed before the answer arrives; do not touch state after that.
    let alive = true;
    void getPlatformInfo().then((p) => {
      if (alive) platform = p;
    });
    return () => {
      alive = false;
    };
  });

  // Take the stored settings once they arrive, and follow later backend changes while idle.
  $effect(() => {
    const current = $settings;
    if (current && draft === null) draft = { ...current };
  });

  /** The clipboard is polled on macOS and on Wayland; elsewhere the interval has no effect. */
  const showPollInterval = $derived.by(
    (): boolean =>
      platform !== null && (platform.os === 'macos' || platform.displayServer === 'wayland'),
  );

  const languageValue = $derived.by((): string => draft?.language ?? 'system');

  function setLanguage(value: string): void {
    if (!draft) return;
    draft.language = value === 'system' ? null : (value as Language);
  }

  function checkRanges(next: Settings): boolean {
    const errors: typeof fieldErrors = {};
    for (const field of Object.keys(SETTINGS_RANGES) as RangedSettingsField[]) {
      const { min, max } = SETTINGS_RANGES[field];
      const value = next[field];
      if (!Number.isFinite(value) || value < min || value > max) {
        errors[field] = $t('settings.invalid');
      }
    }
    fieldErrors = errors;
    return Object.keys(errors).length === 0;
  }

  async function save(): Promise<void> {
    if (!draft) return;
    saveError = null;
    if (!checkRanges(draft)) return;
    try {
      const applied = await store.update({ ...draft });
      draft = { ...applied };
      // The chosen language takes effect without a restart (12.4); `null` follows the OS.
      i18n.setLanguage(resolveInitialLanguage(applied.language, navigator.language));
      fieldErrors = {};
      await closeWindow();
    } catch (e) {
      if (isAppError(e) && e.kind === 'hotkeyUnavailable') {
        fieldErrors = { hotkey: $t('settings.hotkey.unavailable') };
      } else if (isAppError(e) && e.kind === 'invalidSettings') {
        saveError = $t('settings.invalid');
      } else {
        saveError = $t('settings.saveFailed');
      }
    }
  }

  function startRecording(): void {
    recording = true;
    void suspendHotkey();
  }

  function stopRecording(): void {
    recording = false;
    void resumeHotkey();
  }

  /** Tauri accelerator built from the physical key plus its modifiers. */
  function onRecordKey(e: KeyboardEvent): void {
    if (!recording || !draft) return;
    e.preventDefault();
    if (e.code === 'Escape') {
      stopRecording();
      return;
    }
    if (['ShiftLeft', 'ShiftRight', 'ControlLeft', 'ControlRight'].includes(e.code)) return;
    if (['AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(e.code)) return;

    const modifiers: string[] = [];
    if (e.metaKey) modifiers.push('Command');
    if (e.ctrlKey) modifiers.push('Control');
    if (e.altKey) modifiers.push('Alt');
    if (e.shiftKey) modifiers.push('Shift');
    if (modifiers.length === 0) {
      // A global shortcut without modifiers would swallow ordinary typing.
      fieldErrors = { ...fieldErrors, hotkey: $t('settings.hotkey.needsModifier') };
      return;
    }
    draft.hotkey = [...modifiers, e.code].join('+');
    fieldErrors = { ...fieldErrors, hotkey: undefined };
    stopRecording();
  }

  // Closing the window mid-recording must not leave the hotkey disabled.
  onMount(() => () => {
    if (recording) void resumeHotkey();
  });
</script>

<!-- Recording listens on the window: on macOS a click does not move keyboard focus to a
     button, so a keydown handler on the button itself would never fire. -->
<svelte:window onkeydown={onRecordKey} />

{#if draft}
  <main class="settings">
    <h1>{$t('settings.title')}</h1>

    <label class="field">
      <span class="label">{$t('settings.capacity')}</span>
      <input
        type="number"
        min={SETTINGS_RANGES.capacity.min}
        max={SETTINGS_RANGES.capacity.max}
        value={draft.capacity}
        oninput={(e) => draft && (draft.capacity = e.currentTarget.valueAsNumber)}
      />
    </label>
    {#if fieldErrors.capacity}
      <p class="error" data-testid="error-capacity">{fieldErrors.capacity}</p>
    {/if}

    <div class="field">
      <span class="label" id="hotkey-label">{$t('settings.hotkey')}</span>
      <button
        type="button"
        class="hotkey"
        class:recording
        aria-labelledby="hotkey-label"
        onclick={startRecording}
      >
        {recording ? $t('settings.hotkey.record') : draft.hotkey}
      </button>
    </div>
    {#if fieldErrors.hotkey}
      <p class="error" data-testid="error-hotkey">{fieldErrors.hotkey}</p>
    {/if}

    <label class="field">
      <span class="label">{$t('settings.tabWidth')}</span>
      <input
        type="number"
        min={SETTINGS_RANGES.tabWidth.min}
        max={SETTINGS_RANGES.tabWidth.max}
        value={draft.tabWidth}
        oninput={(e) => draft && (draft.tabWidth = e.currentTarget.valueAsNumber)}
      />
    </label>
    <p class="hint">{$t('settings.tabWidth.hint')}</p>
    {#if fieldErrors.tabWidth}
      <p class="error" data-testid="error-tabWidth">{fieldErrors.tabWidth}</p>
    {/if}

    {#if showPollInterval}
      <label class="field">
        <span class="label">{$t('settings.pollIntervalMs')}</span>
        <input
          type="number"
          min={SETTINGS_RANGES.pollIntervalMs.min}
          max={SETTINGS_RANGES.pollIntervalMs.max}
          step="50"
          value={draft.pollIntervalMs}
          oninput={(e) => draft && (draft.pollIntervalMs = e.currentTarget.valueAsNumber)}
        />
      </label>
      {#if fieldErrors.pollIntervalMs}
        <p class="error" data-testid="error-pollIntervalMs">{fieldErrors.pollIntervalMs}</p>
      {/if}
    {/if}

    <label class="field checkbox">
      <input
        type="checkbox"
        checked={draft.autostart}
        onchange={(e) => draft && (draft.autostart = e.currentTarget.checked)}
      />
      <span class="label">{$t('settings.autostart')}</span>
    </label>

    <label class="field checkbox" title={$t('settings.previewWrap.hint')}>
      <input
        type="checkbox"
        checked={draft.previewWrap}
        onchange={(e) => draft && (draft.previewWrap = e.currentTarget.checked)}
      />
      <span class="label">{$t('settings.previewWrap')}</span>
    </label>

    <label class="field">
      <span class="label">{$t('settings.language')}</span>
      <select value={languageValue} onchange={(e) => setLanguage(e.currentTarget.value)}>
        <option value="system">{$t('settings.language.system')}</option>
        <option value="ja">{$t('settings.language.ja')}</option>
        <option value="en">{$t('settings.language.en')}</option>
      </select>
    </label>

    {#if saveError}
      <p class="error" role="alert">{saveError}</p>
    {/if}

    <div class="actions">
      <button type="button" class="save" onclick={save}>{$t('settings.save')}</button>
    </div>
  </main>
{/if}

<style>
  .settings {
    padding: 1em 1.25em;
    display: flex;
    flex-direction: column;
    gap: 0.6em;
    font-size: 0.95em;
  }
  h1 {
    margin: 0 0 0.3em;
    font-size: 1.1em;
  }
  .field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1em;
  }
  .field.checkbox {
    justify-content: flex-start;
    gap: 0.5em;
  }
  .label {
    flex: 0 1 auto;
  }
  input[type='number'] {
    font: inherit;
    width: 8em;
    padding: 0.2em 0.4em;
  }
  /* Wide enough for the longest option ("システムに従う") without clipping. */
  select {
    font: inherit;
    min-width: 12em;
    padding: 0.2em 0.4em;
  }
  .hotkey {
    font: inherit;
    font-family: ui-monospace, monospace;
    min-width: 12em;
    padding: 0.3em 0.6em;
    border: 1px solid color-mix(in srgb, currentColor 35%, transparent);
    border-radius: 4px;
    background: color-mix(in srgb, currentColor 8%, transparent);
    color: inherit;
    cursor: pointer;
  }
  .hotkey.recording {
    border-color: #2563eb;
    background: color-mix(in srgb, #2563eb 20%, transparent);
  }
  .hint {
    margin: -0.4em 0 0;
    font-size: 0.8em;
    opacity: 0.7;
  }
  .error {
    margin: -0.3em 0 0;
    color: #dc2626;
    font-size: 0.85em;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 0.5em;
  }
  .save {
    font: inherit;
    padding: 0.35em 1.2em;
    border: 1px solid color-mix(in srgb, currentColor 35%, transparent);
    border-radius: 4px;
    background: color-mix(in srgb, currentColor 10%, transparent);
    color: inherit;
    cursor: pointer;
  }
</style>
