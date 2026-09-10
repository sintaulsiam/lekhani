//! Lekhani Settings & Configuration Engine

pub mod layout;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use layout::{LayoutInfo, LayoutManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub phonetic: PhoneticConfig,
    pub fixed: FixedConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub active_layout: String,
    pub check_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneticConfig {
    pub use_dictionary: bool,
    pub include_english: bool,
    pub enter_key_closes_candidate_window: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedConfig {
    pub auto_vowel_forming: bool,
    pub auto_chandra_position: bool,
    pub traditional_kar: bool,
    pub old_reph: bool,
    pub numberpad: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub horizontal_candidates: bool,
    pub topbar_x: i32,
    pub topbar_y: i32,
    pub dark_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                active_layout: "Avro Phonetic".to_string(),
                check_updates: true,
            },
            phonetic: PhoneticConfig {
                use_dictionary: true,
                include_english: true,
                enter_key_closes_candidate_window: false,
            },
            fixed: FixedConfig {
                auto_vowel_forming: true,
                auto_chandra_position: true,
                traditional_kar: false,
                old_reph: true,
                numberpad: true,
            },
            ui: UiConfig {
                horizontal_candidates: true,
                topbar_x: 200,
                topbar_y: 50,
                dark_mode: true,
            },
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    data_dir: PathBuf,
    pub config: AppConfig,
}

impl ConfigManager {
    pub fn new() -> Self {
        let proj = ProjectDirs::from("io.github", "openbangla", "lekhani")
            .or_else(|| ProjectDirs::from("com", "openbangla", "keyboard"));

        let (config_path, data_dir) = if let Some(p) = proj {
            let conf_dir = p.config_dir().to_path_buf();
            let _ = std::fs::create_dir_all(&conf_dir);
            let d_dir = p.data_dir().to_path_buf();
            let _ = std::fs::create_dir_all(&d_dir);
            let _ = std::fs::create_dir_all(d_dir.join("layouts"));
            (conf_dir.join("config.toml"), d_dir)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let conf = PathBuf::from(&home).join(".config/lekhani");
            let data = PathBuf::from(&home).join(".local/share/lekhani");
            let _ = std::fs::create_dir_all(&conf);
            let _ = std::fs::create_dir_all(&data);
            let _ = std::fs::create_dir_all(data.join("layouts"));
            (conf.join("config.toml"), data)
        };

        let mut mgr = Self {
            config_path,
            data_dir,
            config: AppConfig::default(),
        };

        mgr.load();
        mgr
    }

    pub fn load(&mut self) {
        if self.config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&self.config_path) {
                if let Ok(conf) = toml::from_str::<AppConfig>(&content) {
                    self.config = conf;
                }
            }
        } else {
            self.save();
        }
    }

    pub fn save(&self) {
        if let Ok(toml_str) = toml::to_string_pretty(&self.config) {
            let _ = std::fs::write(&self.config_path, toml_str);
        }
    }

    pub fn get_data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    pub fn get_user_layout_dir(&self) -> PathBuf {
        self.data_dir.join("layouts")
    }

    pub fn get_user_autocorrect_path(&self) -> PathBuf {
        self.data_dir.join("autocorrect.json")
    }

    pub fn get_system_layout_dir() -> PathBuf {
        let candidates = [
            PathBuf::from("/usr/share/lekhani/layouts"),
            PathBuf::from("/usr/share/openbangla-keyboard/layouts"),
            PathBuf::from("/usr/local/share/lekhani/layouts"),
            PathBuf::from("./data/layouts"),
            PathBuf::from("../data/layouts"),
            PathBuf::from("../../data/layouts"),
        ];
        for c in candidates {
            if c.exists() {
                return c;
            }
        }
        PathBuf::from("/usr/share/lekhani/layouts")
    }

    pub fn get_system_data_dir() -> PathBuf {
        let candidates = [
            PathBuf::from("/usr/share/lekhani/data"),
            PathBuf::from("/usr/share/openbangla-keyboard"),
            PathBuf::from("/usr/local/share/lekhani/data"),
            PathBuf::from("./data"),
            PathBuf::from("../data"),
            PathBuf::from("../../data"),
        ];
        for c in candidates {
            if c.exists() {
                return c;
            }
        }
        PathBuf::from("/usr/share/lekhani/data")
    }
}
