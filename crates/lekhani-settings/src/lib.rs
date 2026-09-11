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

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneticConfig {
    pub use_dictionary: bool,
    pub include_english: bool,
    pub enter_key_closes_candidate_window: bool,
    #[serde(default = "default_true")]
    pub enable_predictive_next_words: bool,
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
                enable_predictive_next_words: true,
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

#[derive(Debug, Clone)]
pub struct ConfigManager {
    config_path: PathBuf,
    data_dir: PathBuf,
    pub config: AppConfig,
    last_config_mtime: Option<std::time::SystemTime>,
    last_autocorrect_mtime: Option<std::time::SystemTime>,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
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
            last_config_mtime: None,
            last_autocorrect_mtime: None,
        };

        mgr.load();
        mgr
    }

    pub fn load(&mut self) {
        if self.config_path.exists() {
            if let Ok(metadata) = self.config_path.metadata() {
                self.last_config_mtime = metadata.modified().ok();
            }
            if let Ok(content) = std::fs::read_to_string(&self.config_path) {
                if let Ok(conf) = toml::from_str::<AppConfig>(&content) {
                    self.config = conf;
                }
            }
        } else {
            self.save();
        }

        let ac_path = self.get_user_autocorrect_path();
        if ac_path.exists() {
            if let Ok(metadata) = ac_path.metadata() {
                self.last_autocorrect_mtime = metadata.modified().ok();
            }
        }
    }

    /// Check if config.toml or autocorrect.json has been modified by external GUI/editor
    pub fn check_and_reload(&mut self) -> bool {
        let mut changed = false;

        if let Ok(meta) = self.config_path.metadata() {
            if let Ok(mtime) = meta.modified() {
                if self.last_config_mtime.is_none_or(|last| mtime > last) {
                    changed = true;
                }
            }
        }

        let ac_path = self.get_user_autocorrect_path();
        if let Ok(meta) = ac_path.metadata() {
            if let Ok(mtime) = meta.modified() {
                if self.last_autocorrect_mtime.is_none_or(|last| mtime > last) {
                    changed = true;
                }
            }
        }

        if changed {
            self.load();
        }
        changed
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

    pub fn get_user_learned_path(&self) -> PathBuf {
        self.data_dir.join("user_learned.json")
    }

    pub fn get_system_layout_dir() -> PathBuf {
        let mut candidates = Vec::new();
        if let Some(proj) = ProjectDirs::from("org", "lekhani", "lekhani") {
            candidates.push(proj.data_local_dir().join("layouts"));
        }
        if let Some(home) = std::env::var_os("HOME") {
            candidates.push(PathBuf::from(home).join(".local/share/lekhani/layouts"));
        }
        candidates.extend([
            PathBuf::from("./data/layouts"),
            PathBuf::from("../data/layouts"),
            PathBuf::from("../../data/layouts"),
            PathBuf::from("/usr/share/lekhani/layouts"),
            PathBuf::from("/usr/share/openbangla-keyboard/layouts"),
            PathBuf::from("/usr/local/share/lekhani/layouts"),
        ]);
        for c in candidates {
            if c.exists() && (c.join("avrophonetic.json").exists() || c.join("Probhat.json").exists()) {
                return c;
            }
        }
        PathBuf::from("./data/layouts")
    }

    pub fn get_system_data_dir() -> PathBuf {
        let mut candidates = Vec::new();
        if let Some(proj) = ProjectDirs::from("org", "lekhani", "lekhani") {
            candidates.push(proj.data_local_dir().join("data"));
            candidates.push(proj.data_local_dir().to_path_buf());
        }
        if let Some(home) = std::env::var_os("HOME") {
            candidates.push(PathBuf::from(&home).join(".local/share/lekhani/data"));
            candidates.push(PathBuf::from(home).join(".local/share/lekhani"));
        }
        candidates.extend([
            PathBuf::from("./data/dictionaries"),
            PathBuf::from("./data"),
            PathBuf::from("../data/dictionaries"),
            PathBuf::from("../data"),
            PathBuf::from("../../data/dictionaries"),
            PathBuf::from("../../data"),
            PathBuf::from("/usr/share/lekhani/data"),
            PathBuf::from("/usr/share/lekhani"),
            PathBuf::from("/usr/share/openbangla-keyboard"),
            PathBuf::from("/usr/local/share/lekhani/data"),
            PathBuf::from("/usr/local/share/lekhani"),
        ]);
        for c in candidates {
            if c.exists() && (c.join("dictionary.json").exists() || c.join("dictionary.bin").exists() || c.join("autocorrect.json").exists()) {
                return c;
            }
        }
        PathBuf::from("/usr/share/lekhani/data")
    }

    /// Export a complete backup bundle (config, autocorrect, learned words, custom layouts, user stats)
    pub fn export_backup<P: AsRef<std::path::Path>>(&self, dest: P) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut user_autocorrect = std::collections::HashMap::new();
        let ac_path = self.get_user_autocorrect_path();
        if ac_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&ac_path) {
                if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&content) {
                    user_autocorrect = map;
                }
            }
        }

        let mut user_learned = None;
        let learned_path = self.get_user_learned_path();
        if learned_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&learned_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    user_learned = Some(val);
                }
            }
        }

        let mut custom_layouts = std::collections::HashMap::new();
        let layout_dir = self.get_user_layout_dir();
        if layout_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&layout_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                custom_layouts.insert(file_name.to_string(), content);
                            }
                        }
                    }
                }
            }
        }

        let mut user_stats = None;
        let stats_path = self.data_dir.join("stats.json");
        if stats_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&stats_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    user_stats = Some(val);
                }
            }
        }

        let bundle = BackupBundle {
            version: "3.0.0".to_string(),
            exported_at: chrono::Local::now().to_rfc3339(),
            config: self.config.clone(),
            user_autocorrect,
            user_learned,
            custom_layouts,
            user_stats,
        };

        let data = serde_json::to_string_pretty(&bundle)?;
        if let Some(parent) = dest.as_ref().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(dest, data)?;
        Ok(())
    }

    /// Import and restore a complete backup bundle
    pub fn import_backup<P: AsRef<std::path::Path>>(&mut self, src: P) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string(src)?;
        let bundle: BackupBundle = serde_json::from_str(&content)?;

        self.config = bundle.config;
        self.save();

        let ac_path = self.get_user_autocorrect_path();
        let ac_json = serde_json::to_string_pretty(&bundle.user_autocorrect)?;
        let _ = std::fs::write(ac_path, ac_json);

        if let Some(learned_val) = bundle.user_learned {
            let learned_path = self.get_user_learned_path();
            let learned_json = serde_json::to_string_pretty(&learned_val)?;
            let _ = std::fs::write(learned_path, learned_json);
        }

        let layout_dir = self.get_user_layout_dir();
        let _ = std::fs::create_dir_all(&layout_dir);
        for (name, content) in bundle.custom_layouts {
            let p = layout_dir.join(name);
            let _ = std::fs::write(p, content);
        }

        if let Some(stats_val) = bundle.user_stats {
            let stats_path = self.data_dir.join("stats.json");
            let stats_json = serde_json::to_string_pretty(&stats_val)?;
            let _ = std::fs::write(stats_path, stats_json);
        }

        self.load();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupBundle {
    pub version: String,
    pub exported_at: String,
    pub config: AppConfig,
    pub user_autocorrect: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub user_learned: Option<serde_json::Value>,
    pub custom_layouts: std::collections::HashMap<String, String>,
    pub user_stats: Option<serde_json::Value>,
}
