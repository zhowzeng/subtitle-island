use crate::session::{AudioSource, SourceLanguage, StartSessionOptions};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, path::PathBuf};
use tauri::Manager;

const SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DisplayMode {
    #[default]
    Original,
    Chinese,
    Bilingual,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Settings {
    pub(crate) audio_source: AudioSource,
    pub(crate) source_language: SourceLanguage,
    pub(crate) display_mode: DisplayMode,
}

impl Settings {
    pub(crate) fn session_options(self) -> StartSessionOptions {
        StartSessionOptions {
            audio_source: self.audio_source,
            source_language: self.source_language,
            translate: self.display_mode != DisplayMode::Original,
        }
    }
}

pub(crate) fn load_settings(app: &tauri::AppHandle) -> Settings {
    match settings_path(app) {
        Ok(path) => read_settings(&path),
        Err(error) => {
            tracing::warn!("Using default settings: {error}");
            Settings::default()
        }
    }
}

pub(crate) fn save_settings(app: &tauri::AppHandle, settings: Settings) -> Result<(), String> {
    write_settings(&settings_path(app)?, settings)
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(SETTINGS_FILE_NAME))
        .map_err(|error| format!("Could not find the settings directory: {error}"))
}

/// Missing or invalid settings fall back to defaults so the app always starts.
fn read_settings(path: &Path) -> Settings {
    let Ok(contents) = fs::read_to_string(path) else {
        return Settings::default();
    };

    serde_json::from_str(&contents).unwrap_or_else(|error| {
        tracing::warn!("Using default settings because the settings file is invalid: {error}");
        Settings::default()
    })
}

fn write_settings(path: &Path, settings: Settings) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|error| format!("Could not save settings: {error}"))?;
    }
    let contents = serde_json::to_string_pretty(&settings)
        .map_err(|error| format!("Could not save settings: {error}"))?;
    fs::write(path, contents).map_err(|error| format!("Could not save settings: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("subis-settings-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn missing_file_uses_defaults() {
        let path = test_dir("missing").join(SETTINGS_FILE_NAME);

        assert_eq!(read_settings(&path), Settings::default());
    }

    #[test]
    fn invalid_file_uses_defaults() {
        let dir = test_dir("invalid");
        let path = dir.join(SETTINGS_FILE_NAME);
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, r#"{"displayMode":"unknown"}"#).unwrap();

        assert_eq!(read_settings(&path), Settings::default());
    }

    #[test]
    fn missing_fields_use_defaults() {
        let dir = test_dir("partial");
        let path = dir.join(SETTINGS_FILE_NAME);
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, r#"{"displayMode":"chinese"}"#).unwrap();

        assert_eq!(
            read_settings(&path),
            Settings {
                display_mode: DisplayMode::Chinese,
                ..Settings::default()
            }
        );
    }

    #[test]
    fn settings_round_trip() {
        let path = test_dir("round-trip").join(SETTINGS_FILE_NAME);
        let settings = Settings {
            audio_source: AudioSource::System,
            source_language: SourceLanguage::En,
            display_mode: DisplayMode::Bilingual,
        };

        write_settings(&path, settings).unwrap();

        assert_eq!(read_settings(&path), settings);
    }

    #[test]
    fn only_original_mode_skips_translation() {
        let translate = |display_mode| {
            Settings {
                display_mode,
                ..Settings::default()
            }
            .session_options()
            .translate
        };

        assert!(!translate(DisplayMode::Original));
        assert!(translate(DisplayMode::Chinese));
        assert!(translate(DisplayMode::Bilingual));
    }
}
