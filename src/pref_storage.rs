use crate::errors::Error;
use gettextrs::gettext;
use gio::{Settings, SettingsExt, SettingsSchemaSource};
use log::{debug, trace, warn};
use serde::{de::DeserializeOwned, Serialize};
use serde_derive::*;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::prelude::*;
use tempfile::tempdir;

/// Generic wrapper struct around XiConfig
#[derive(Clone, Debug)]
pub struct Config<T> {
    pub path: String,
    pub config: T,
}

/// For stuff that goes into preferences.xiconfig
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct XiConfig {
    pub tab_size: u32,
    pub translate_tabs_to_spaces: bool,
    pub use_tab_stops: bool,
    pub plugin_search_path: Vec<String>,
    pub font_face: String,
    pub font_size: u32,
    pub auto_indent: bool,
    pub scroll_past_end: bool,
    pub wrap_width: u32,
    pub word_wrap: bool,
    pub autodetect_whitespace: bool,
    pub line_ending: String,
    pub surrounding_pairs: Vec<Vec<String>>,
}

impl Default for XiConfig {
    fn default() -> XiConfig {
        #[cfg(windows)]
        const LINE_ENDING: &str = "\r\n";
        #[cfg(not(windows))]
        const LINE_ENDING: &str = "\n";

        let surrounding_pairs = vec![
            vec!["\"".to_string(), "\"".to_string()],
            vec!["'".to_string(), "'".to_string()],
            vec!["{".to_string(), "}".to_string()],
            vec!["[".to_string(), "]".to_string()],
        ];

        // Default valuess as dictated by https://github.com/xi-editor/xi-editor/blob/master/rust/core-lib/assets/client_example.toml
        XiConfig {
            tab_size: 4,
            translate_tabs_to_spaces: false,
            use_tab_stops: true,
            plugin_search_path: vec![String::new()],
            font_face: get_default_monospace_font_schema(),
            font_size: 12,
            auto_indent: true,
            scroll_past_end: false,
            wrap_width: 0,
            word_wrap: false,
            autodetect_whitespace: true,
            line_ending: LINE_ENDING.to_string(),
            surrounding_pairs,
        }
    }
}

impl<T> Config<T> {
    pub fn new(path: String) -> Config<T>
    where
        T: Default,
    {
        Config {
            config: T::default(),
            path,
        }
    }

    pub fn open(&mut self) -> Result<&mut Config<T>, Error>
    where
        T: Clone + Debug + DeserializeOwned,
    {
        trace!("{}", gettext("Opening config file"));
        let mut config_file = OpenOptions::new().read(true).open(&self.path)?;
        let mut config_string = String::new();

        trace!("{}", gettext("Reading config file"));
        config_file.read_to_string(&mut config_string)?;

        let config_toml: T = toml::from_str(&config_string)?;
        debug!("{}: {:?}", gettext("Xi-Config"), config_toml);

        self.config = config_toml.clone();

        Ok(self)
    }

    /// Atomically write the config. First writes the config to a tmp_file (non-atomic) and then
    /// copies that (atomically). This ensures that the config files stay valid
    pub fn save(&self) -> Result<(), Error>
    where
        T: Serialize,
    {
        let tmp_dir = tempdir()?;
        let tmp_file_path = tmp_dir.path().join(".gxi-atomic");
        let mut tmp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&tmp_file_path)?;

        tmp_file.write_all(toml::to_string(&self.config).unwrap().as_bytes())?;
        std::fs::copy(&tmp_file_path, &self.path)?;
        OpenOptions::new().read(true).open(&self.path)?.sync_all()?;

        Ok(())
    }
}

pub fn get_theme_schema() -> String {
    SettingsSchemaSource::get_default()
        .and_then(|settings_source| settings_source.lookup("com.github.Cogitri.gxi", true))
        .and_then(|_| {
            Settings::new(crate::globals::APP_ID.unwrap_or("com.github.Cogitri.gxi"))
                .get_string("theme-name")
        })
        .unwrap_or_else(|| {
            warn!("Couldn't find GSchema! Defaulting to default theme.");
            "InspiredGitHub".to_string()
        })
}

pub fn set_theme_schema(theme_name: &str) {
    SettingsSchemaSource::get_default()
        .and_then(|settings_source| settings_source.lookup("com.github.Cogitri.gxi", true))
        .map(|_| {
            Settings::new(crate::globals::APP_ID.unwrap_or("com.github.Cogitri.gxi"))
                .set_string("theme-name", theme_name);
        });
}

pub fn get_default_monospace_font_schema() -> String {
    SettingsSchemaSource::get_default()
        .and_then(|settings_source| settings_source.lookup("org.gnome.desktop.interface", true))
        .and_then(|_| {
            Settings::new("org.gnome.desktop.interface").get_string("monospace-font-name")
        })
        .unwrap_or_else(|| {
            warn!("Couldn't find GSchema! Defaulting to default monospace font.");
            "Monospace".to_string()
        })
}

pub fn get_draw_spaces_schema() -> bool {
    SettingsSchemaSource::get_default()
        .and_then(|settings_source| settings_source.lookup("com.github.Cogitri.gxi", true))
        .map(|_| {
            Settings::new(crate::globals::APP_ID.unwrap_or("com.github.Cogitri.gxi"))
                .get_boolean("draw-spaces")
        })
        .unwrap_or_else(|| {
            warn!("Couldn't find GSchema! Defaulting to not drawing tabs!");
            false
        })
}

pub fn set_draw_spaces_schema(val: bool) {
    SettingsSchemaSource::get_default()
        .and_then(|settings_source| settings_source.lookup("com.github.Cogitri.gxi", true))
        .map(|_| {
            Settings::new(crate::globals::APP_ID.unwrap_or("com.github.Cogitri.gxi"))
                .set_boolean("draw-spaces", val);
        });
}

pub fn get_draw_tabs_schema() -> bool {
    SettingsSchemaSource::get_default()
        .and_then(|settings_source| settings_source.lookup("com.github.Cogitri.gxi", true))
        .map(|_| {
            Settings::new(crate::globals::APP_ID.unwrap_or("com.github.Cogitri.gxi"))
                .get_boolean("draw-tabs")
        })
        .unwrap_or_else(|| {
            warn!("Couldn't find GSchema! Defaulting to not drawing tabs!");
            false
        })
}

pub fn set_draw_tabs_schema(val: bool) {
    SettingsSchemaSource::get_default()
        .and_then(|settings_source| settings_source.lookup("com.github.Cogitri.gxi", true))
        .map(|_| {
            Settings::new(crate::globals::APP_ID.unwrap_or("com.github.Cogitri.gxi"))
                .set_boolean("draw-tabs", val);
        });
}
