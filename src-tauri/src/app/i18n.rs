//! Translations for the parts of the UI that Rust owns: the tray menu and the window titles
//! (12.1). The webview has its own copy of the same files through Vite.
//!
//! The strings are read from `src/locales/*.json` at compile time, so there is one source of
//! truth for every language string in the app, and the tray is already correct on the first
//! frame — before any webview has loaded.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::model::Language;

const JA: &str = include_str!("../../../src/locales/ja.json");
const EN: &str = include_str!("../../../src/locales/en.json");

/// Keys this side of the app renders. Kept as a list so a test can prove they all exist.
pub const TRAY_TOGGLE: &str = "tray.toggle";
pub const TRAY_SETTINGS: &str = "tray.settings";
pub const TRAY_ABOUT: &str = "tray.about";
pub const TRAY_QUIT: &str = "tray.quit";
pub const WINDOW_SETTINGS_TITLE: &str = "window.settings.title";
pub const WINDOW_ABOUT_TITLE: &str = "window.about.title";

/// Only the test below reads this; it exists so adding a key here is what proves it exists.
#[cfg(test)]
const USED_KEYS: [&str; 6] = [
    TRAY_TOGGLE,
    TRAY_SETTINGS,
    TRAY_ABOUT,
    TRAY_QUIT,
    WINDOW_SETTINGS_TITLE,
    WINDOW_ABOUT_TITLE,
];

fn table(language: Language) -> &'static HashMap<String, String> {
    static TABLES: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
    let tables = TABLES.get_or_init(|| {
        // The files ship with the binary and are checked by a test below, so a parse failure
        // here is a build-time mistake, not something a user can cause.
        HashMap::from([
            (
                "ja",
                serde_json::from_str(JA).expect("ja.json is not an object of strings"),
            ),
            (
                "en",
                serde_json::from_str(EN).expect("en.json is not an object of strings"),
            ),
        ])
    });
    &tables[match language {
        Language::Ja => "ja",
        Language::En => "en",
    }]
}

/// The string for `key`, falling back to English and then to the key itself, exactly as the
/// frontend's `t` does.
pub fn t(language: Language, key: &str) -> &'static str {
    table(language)
        .get(key)
        .or_else(|| table(Language::En).get(key))
        .map(String::as_str)
        .unwrap_or_else(|| leak_key(key))
}

/// A missing key is a bug we want visible rather than silent; the leak is bounded by the
/// number of distinct keys in the binary.
fn leak_key(key: &str) -> &'static str {
    Box::leak(key.to_string().into_boxed_str())
}

/// The saved setting wins; otherwise a Japanese OS gets Japanese and everything else English
/// (12.2, 12.3). Mirrors `resolveInitialLanguage` in the frontend, which reads
/// `navigator.language` instead of the OS locale.
pub fn resolve_language(setting: Option<Language>, os_locale: Option<&str>) -> Language {
    if let Some(language) = setting {
        return language;
    }
    match os_locale {
        Some(locale) if locale.to_ascii_lowercase().starts_with("ja") => Language::Ja,
        _ => Language::En,
    }
}

/// The language the native UI should use right now, from the stored setting and the OS locale.
pub fn current(setting: Option<Language>) -> Language {
    resolve_language(setting, sys_locale::get_locale().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_rust_renders_exists_in_both_languages() {
        for key in USED_KEYS {
            for language in [Language::Ja, Language::En] {
                assert!(
                    table(language).contains_key(key),
                    "{key} is missing from the {language:?} locale file"
                );
            }
        }
    }

    #[test]
    fn the_two_locale_files_carry_the_same_keys() {
        let ja: Vec<&String> = {
            let mut keys: Vec<&String> = table(Language::Ja).keys().collect();
            keys.sort();
            keys
        };
        let en: Vec<&String> = {
            let mut keys: Vec<&String> = table(Language::En).keys().collect();
            keys.sort();
            keys
        };
        assert_eq!(ja, en, "the locale files have drifted apart");
    }

    #[test]
    fn translations_differ_per_language() {
        assert_ne!(t(Language::Ja, TRAY_QUIT), t(Language::En, TRAY_QUIT));
        assert_eq!(t(Language::En, TRAY_TOGGLE), "Show / Hide");
    }

    #[test]
    fn an_unknown_key_falls_back_to_itself() {
        assert_eq!(t(Language::Ja, "no.such.key"), "no.such.key");
    }

    #[test]
    fn the_saved_setting_wins_over_the_os_locale() {
        assert_eq!(
            resolve_language(Some(Language::En), Some("ja_JP.UTF-8")),
            Language::En
        );
        assert_eq!(
            resolve_language(Some(Language::Ja), Some("en_US")),
            Language::Ja
        );
    }

    #[test]
    fn without_a_setting_only_a_japanese_os_gets_japanese() {
        assert_eq!(resolve_language(None, Some("ja_JP.UTF-8")), Language::Ja);
        assert_eq!(resolve_language(None, Some("ja")), Language::Ja);
        assert_eq!(resolve_language(None, Some("JA-JP")), Language::Ja);
        for other in ["en_US", "de_DE", "zh_CN", "ko_KR", "fr", ""] {
            assert_eq!(
                resolve_language(None, Some(other)),
                Language::En,
                "{other} should fall back to English (12.3)"
            );
        }
        assert_eq!(resolve_language(None, None), Language::En);
    }
}
