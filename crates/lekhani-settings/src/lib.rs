//! Lekhani Settings & Configuration Engine

pub mod layout;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use layout::{LayoutInfo, LayoutManager};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub phonetic: PhoneticConfig,
    pub fixed: FixedConfig,
    pub ui: UiConfig,
}

fn default_toggle_key() -> String {
    "F12".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub active_layout: String,
    pub check_updates: bool,
    #[serde(default = "default_toggle_key")]
    pub toggle_key: String,
    #[serde(default = "default_true")]
    pub show_osd: bool,
    #[serde(default = "default_true")]
    pub auto_dari: bool,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

pub use lekhani_core::phonetic::suggestion::AiProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneticConfig {
    #[serde(default)]
    pub ai_profile: AiProfile,
    #[serde(default = "default_true")]
    pub use_dictionary: bool,
    #[serde(default = "default_true")]
    pub include_english: bool,
    #[serde(default = "default_false")]
    pub enter_key_closes_candidate_window: bool,
    #[serde(default = "default_true")]
    pub enable_predictive_next_words: bool,
    #[serde(default = "default_false")]
    pub enable_code_shield: bool,
    #[serde(default = "default_false")]
    pub enable_word_segmentation: bool,
    #[serde(default = "default_true")]
    pub enable_colloquial_dialects: bool,
    #[serde(default = "default_true")]
    pub enable_banglish_shorthand: bool,
    #[serde(default = "default_true")]
    pub enable_reduplication: bool,
    #[serde(default = "default_true")]
    pub enable_phrase_prediction: bool,
    #[serde(default = "default_true")]
    pub enable_dynamic_macros: bool,
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

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            active_layout: "Avro Phonetic".to_string(),
            check_updates: true,
            toggle_key: "F12".to_string(),
            show_osd: true,
            auto_dari: true,
        }
    }
}

impl Default for PhoneticConfig {
    fn default() -> Self {
        Self {
            ai_profile: AiProfile::default(),
            use_dictionary: true,
            include_english: true,
            enter_key_closes_candidate_window: false,
            enable_predictive_next_words: true,
            enable_code_shield: false,
            enable_word_segmentation: false,
            enable_colloquial_dialects: true,
            enable_banglish_shorthand: true,
            enable_reduplication: true,
            enable_phrase_prediction: true,
            enable_dynamic_macros: true,
        }
    }
}

impl Default for FixedConfig {
    fn default() -> Self {
        Self {
            auto_vowel_forming: true,
            auto_chandra_position: true,
            traditional_kar: false,
            old_reph: true,
            numberpad: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            horizontal_candidates: true,
            topbar_x: 200,
            topbar_y: 50,
            dark_mode: true,
        }
    }
}


impl AppConfig {
    pub fn to_suggestion_config(&self) -> lekhani_core::PhoneticSuggestionConfig {
        lekhani_core::PhoneticSuggestionConfig {
            ai_profile: self.phonetic.ai_profile,
            use_dictionary: self.phonetic.use_dictionary,
            include_english: self.phonetic.include_english,
            enable_code_shield: self.phonetic.enable_code_shield,
            enable_word_segmentation: self.phonetic.enable_word_segmentation,
            enable_colloquial_dialects: self.phonetic.enable_colloquial_dialects,
            enable_banglish_shorthand: self.phonetic.enable_banglish_shorthand,
            enable_reduplication: self.phonetic.enable_reduplication,
            enable_phrase_prediction: self.phonetic.enable_phrase_prediction,
            enable_dynamic_macros: self.phonetic.enable_dynamic_macros,
            auto_dari: self.general.auto_dari,
        }
    }

    pub fn apply_to_session(&self, session: &mut lekhani_core::InputSession) {
        session.update_suggestion_config(self.to_suggestion_config());
        session.update_fixed_config(
            self.fixed.auto_vowel_forming,
            self.fixed.auto_chandra_position,
            self.fixed.traditional_kar,
            self.fixed.old_reph,
            self.fixed.numberpad,
        );
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
        let proj = ProjectDirs::from("io.github", "lekhani", "lekhani");

        let (config_path, data_dir) = if let Some(p) = proj {
            let conf_dir = p.config_dir().to_path_buf();
            let _ = std::fs::create_dir_all(&conf_dir);
            let d_dir = p.data_dir().to_path_buf();
            let _ = std::fs::create_dir_all(&d_dir);
            let _ = std::fs::create_dir_all(d_dir.join("layouts"));
            (conf_dir.join("config.toml"), d_dir)
        } else {
            let base_dir = std::env::var("APPDATA")
                .or_else(|_| std::env::var("USERPROFILE"))
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            let (conf, data) = if cfg!(windows) {
                let lekhani_dir = PathBuf::from(&base_dir).join("Lekhani");
                (lekhani_dir.clone(), lekhani_dir)
            } else {
                (
                    PathBuf::from(&base_dir).join(".config/lekhani"),
                    PathBuf::from(&base_dir).join(".local/share/lekhani"),
                )
            };
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
        self.data_dir.join("user_learned.bin")
    }

    pub fn get_user_stats_path(&self) -> PathBuf {
        self.data_dir.join("stats.json")
    }

    pub fn get_system_layout_dir() -> PathBuf {
        let mut candidates = Vec::new();

        // 1. Check relative to current executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                candidates.push(exe_dir.join("data").join("layouts"));
                candidates.push(exe_dir.join("layouts"));
                if let Some(parent) = exe_dir.parent() {
                    candidates.push(parent.join("data").join("layouts"));
                    candidates.push(parent.join("layouts"));
                }
            }
        }

        // 2. Windows specific system/app paths
        if let Some(progdata) = std::env::var_os("ProgramData") {
            candidates.push(PathBuf::from(progdata).join("Lekhani").join("layouts"));
        }
        if let Some(localapp) = std::env::var_os("LOCALAPPDATA") {
            candidates.push(PathBuf::from(localapp).join("Lekhani").join("layouts"));
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            candidates.push(PathBuf::from(appdata).join("Lekhani").join("layouts"));
        }

        // 3. Local/relative development paths
        candidates.push(PathBuf::from("./data/layouts"));
        candidates.push(PathBuf::from("../data/layouts"));
        candidates.push(PathBuf::from("../../data/layouts"));

        // 4. Linux system paths
        candidates.push(PathBuf::from("/usr/share/lekhani/layouts"));
        candidates.push(PathBuf::from("/usr/local/share/lekhani/layouts"));
        
        for c in &candidates {
            if c.exists()
                && (c.join("avrophonetic.json").exists() || c.join("Probhat.json").exists())
            {
                return c.clone();
            }
        }

        if let Some(home) = std::env::var_os("HOME") {
            let u = PathBuf::from(home).join(".local/share/lekhani/layouts");
            if u.exists() {
                return u;
            }
        }
        PathBuf::from("./data/layouts")
    }

    pub fn get_system_data_dir() -> PathBuf {
        let mut candidates = Vec::new();

        // 1. Check relative to current executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                candidates.push(exe_dir.join("data").join("dictionaries"));
                candidates.push(exe_dir.join("data"));
                candidates.push(exe_dir.join("dictionaries"));
                if let Some(parent) = exe_dir.parent() {
                    candidates.push(parent.join("data").join("dictionaries"));
                    candidates.push(parent.join("data"));
                }
            }
        }

        // 2. Windows specific system/app paths
        if let Some(progdata) = std::env::var_os("ProgramData") {
            let p = PathBuf::from(progdata).join("Lekhani");
            candidates.push(p.join("data"));
            candidates.push(p.join("dictionaries"));
        }
        if let Some(localapp) = std::env::var_os("LOCALAPPDATA") {
            let p = PathBuf::from(localapp).join("Lekhani");
            candidates.push(p.join("data"));
            candidates.push(p.join("dictionaries"));
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let p = PathBuf::from(appdata).join("Lekhani");
            candidates.push(p.join("data"));
            candidates.push(p.join("dictionaries"));
        }

        // 3. User local share
        if let Some(home) = std::env::var_os("HOME") {
            let u_data = PathBuf::from(&home).join(".local/share/lekhani/data");
            if u_data.exists()
                && (u_data.join("dictionary.json").exists()
                    || u_data.join("dictionary.bin").exists()
                    || u_data.join("autocorrect.json").exists())
            {
                return u_data;
            }
            let u_dict = PathBuf::from(&home).join(".local/share/lekhani/dictionaries");
            if u_dict.exists()
                && (u_dict.join("dictionary.json").exists()
                    || u_dict.join("dictionary.bin").exists()
                    || u_dict.join("autocorrect.json").exists())
            {
                return u_dict;
            }
        }

        // 4. Local/relative development paths
        candidates.push(PathBuf::from("./data/dictionaries"));
        candidates.push(PathBuf::from("./data"));
        candidates.push(PathBuf::from("../data/dictionaries"));
        candidates.push(PathBuf::from("../data"));
        candidates.push(PathBuf::from("../../data/dictionaries"));
        candidates.push(PathBuf::from("../../data"));

        // 5. Linux system paths
        candidates.push(PathBuf::from("/usr/share/lekhani/data"));
        candidates.push(PathBuf::from("/usr/share/lekhani"));
        candidates.push(PathBuf::from("/usr/local/share/lekhani/data"));
        candidates.push(PathBuf::from("/usr/local/share/lekhani"));
        
        for c in &candidates {
            if c.exists()
                && (c.join("dictionary.json").exists()
                    || c.join("dictionary.bin").exists()
                    || c.join("autocorrect.json").exists())
            {
                return c.clone();
            }
        }
        PathBuf::from("/usr/share/lekhani/data")
    }

    /// Export a complete backup bundle (config, autocorrect, learned words, custom layouts, user stats)
    pub fn export_backup<P: AsRef<std::path::Path>>(
        &self,
        dest: P,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut user_autocorrect = std::collections::HashMap::new();
        let ac_path = self.get_user_autocorrect_path();
        if ac_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&ac_path) {
                if let Ok(map) =
                    serde_json::from_str::<std::collections::HashMap<String, String>>(&content)
                {
                    user_autocorrect = map;
                }
            }
        }

        let mut user_learned = None;
        let learned_path = self.get_user_learned_path();
        if learned_path.exists() {
            let learner = lekhani_core::AutonomousLearner::load_from_path(&learned_path);
            if let Ok(val) = serde_json::to_value(&learner) {
                user_learned = Some(val);
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
            version: env!("CARGO_PKG_VERSION").to_string(),
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
    pub fn import_backup<P: AsRef<std::path::Path>>(
        &mut self,
        src: P,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string(src)?;
        let bundle: BackupBundle = serde_json::from_str(&content)?;

        self.config = bundle.config;
        self.save();

        let ac_path = self.get_user_autocorrect_path();
        let ac_json = serde_json::to_string_pretty(&bundle.user_autocorrect)?;
        std::fs::write(&ac_path, ac_json)?;

        if let Some(learned_val) = bundle.user_learned {
            if let Ok(mut learner) = serde_json::from_value::<lekhani_core::AutonomousLearner>(learned_val) {
                let learned_path = self.get_user_learned_path();
                learner.dirty = true;
                learner.save_to_path(&learned_path)?;
            }
        }

        let layout_dir = self.get_user_layout_dir();
        std::fs::create_dir_all(&layout_dir)?;
        for (name, content) in bundle.custom_layouts {
            if name.contains("..") || name.contains('/') || name.contains('\\') || name.trim().is_empty() {
                return Err(format!("Invalid or suspicious layout file name in backup: {}", name).into());
            }
            if !name.ends_with(".json") {
                return Err(format!("Layout file must have .json extension: {}", name).into());
            }
            let path_name = std::path::Path::new(&name);
            let file_name = match path_name.file_name() {
                Some(f) if f == std::ffi::OsStr::new(&name) => f,
                _ => return Err(format!("Invalid layout file name in backup: {}", name).into()),
            };
            let p = layout_dir.join(file_name);
            std::fs::write(&p, content)?;
        }

        if let Some(stats_val) = bundle.user_stats {
            let stats_path = self.data_dir.join("stats.json");
            let stats_json = serde_json::to_string_pretty(&stats_val)?;
            std::fs::write(&stats_path, stats_json)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_config_defaults_and_backward_compatibility() {
        let legacy_toml = r#"
            [general]
            active_layout = "Avro Phonetic"
            check_updates = true

            [phonetic]
            use_dictionary = true
            include_english = true
            enter_key_closes_candidate_window = false

            [fixed]
            auto_vowel_forming = true
            auto_chandra_position = true
            traditional_kar = false
            old_reph = true
            numberpad = true

            [ui]
            horizontal_candidates = true
            topbar_x = 200
            topbar_y = 50
            dark_mode = true
        "#;

        let config: AppConfig = toml::from_str(legacy_toml).expect("Should parse legacy config");
        assert_eq!(config.general.toggle_key, "F12");
        assert!(config.general.show_osd);
        assert!(config.general.auto_dari);
        assert!(!config.phonetic.enable_code_shield);
        assert!(!config.phonetic.enable_word_segmentation);
        assert!(config.phonetic.enable_colloquial_dialects);
        assert!(config.phonetic.enable_banglish_shorthand);
        assert!(config.phonetic.enable_reduplication);
        assert!(config.phonetic.enable_phrase_prediction);
        assert!(config.phonetic.enable_dynamic_macros);
    }

    #[test]
    fn test_layout_discovery_and_distinct_mappings() {
        let mut layout_mgr = LayoutManager::new();
        let system_dir = ConfigManager::get_system_layout_dir();
        layout_mgr.discover_layouts(system_dir, std::path::PathBuf::from("/nonexistent"));
        let layouts = layout_mgr.get_layout_list();
        assert!(layouts.len() >= 6, "Should discover at least 6 layouts, got: {:?}", layouts);
        for name in &layouts {
            let json = layout_mgr.load_layout_json(name);
            assert!(json.is_some(), "Layout '{}' should have valid JSON", name);
        }
    }

    #[test]
    fn test_backup_bundle_export_import() {
        let temp_dir = std::env::temp_dir().join(format!(
            "lekhani_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let conf_dir = temp_dir.join("config");
        let data_dir = temp_dir.join("data");
        let _ = std::fs::create_dir_all(&conf_dir);
        let _ = std::fs::create_dir_all(&data_dir);
        let _ = std::fs::create_dir_all(data_dir.join("layouts"));

        let mut cm = ConfigManager {
            config_path: conf_dir.join("config.toml"),
            data_dir: data_dir.clone(),
            config: AppConfig::default(),
            last_config_mtime: None,
            last_autocorrect_mtime: None,
        };

        cm.config.general.toggle_key = "F11".to_string();
        cm.config.phonetic.enable_phrase_prediction = false;
        cm.save();

        // Populate user autocorrect
        let ac_path = cm.get_user_autocorrect_path();
        let mut map = std::collections::HashMap::new();
        map.insert("amr".to_string(), "আমার".to_string());
        let _ = std::fs::write(&ac_path, serde_json::to_string(&map).unwrap());

        // Populate user learned
        let learned_path = cm.get_user_learned_path();
        let mut learner = lekhani_core::AutonomousLearner::new();
        learner.learned_words.insert("বাংলাদেশ".to_string());
        learner.dirty = true;
        learner.save_to_path(&learned_path).unwrap();

        // Export backup
        let backup_path = temp_dir.join("backup.json");
        cm.export_backup(&backup_path).expect("Export should succeed");
        assert!(backup_path.exists());

        // Mutate current state
        cm.config.general.toggle_key = "F12".to_string();
        cm.config.phonetic.enable_phrase_prediction = true;
        cm.save();
        let _ = std::fs::write(&ac_path, "{}");
        let _ = std::fs::write(&learned_path, "{}");

        // Import backup
        cm.import_backup(&backup_path).expect("Import should succeed");

        // Validate restored state
        assert_eq!(cm.config.general.toggle_key, "F11");
        assert!(!cm.config.phonetic.enable_phrase_prediction);

        let restored_ac: std::collections::HashMap<String, String> =
            serde_json::from_str(&std::fs::read_to_string(&ac_path).unwrap()).unwrap();
        assert_eq!(restored_ac.get("amr"), Some(&"আমার".to_string()));

        let restored_learner = lekhani_core::AutonomousLearner::load_from_path(&learned_path);
        assert!(restored_learner.learned_words.contains("বাংলাদেশ"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_backup_import_rejects_path_traversal() {
        let temp_dir = std::env::temp_dir().join(format!("lekhani_settings_test_traversal_{}", std::process::id()));
        let conf_dir = temp_dir.join("config");
        let data_dir = temp_dir.join("data");
        let _ = std::fs::create_dir_all(&conf_dir);
        let _ = std::fs::create_dir_all(&data_dir);

        let mut cm = ConfigManager {
            config_path: conf_dir.join("config.toml"),
            data_dir: data_dir.clone(),
            config: AppConfig::default(),
            last_config_mtime: None,
            last_autocorrect_mtime: None,
        };

        let mut malicious_layouts = std::collections::HashMap::new();
        malicious_layouts.insert("../../evil.json".to_string(), "{}".to_string());

        let bundle = BackupBundle {
            version: "1.0.0".to_string(),
            exported_at: "2026-09-24T00:00:00Z".to_string(),
            config: AppConfig::default(),
            user_autocorrect: std::collections::HashMap::new(),
            user_learned: None,
            custom_layouts: malicious_layouts,
            user_stats: None,
        };

        let backup_path = temp_dir.join("malicious_backup.json");
        std::fs::write(&backup_path, serde_json::to_string(&bundle).unwrap()).unwrap();

        let result = cm.import_backup(&backup_path);
        assert!(result.is_err(), "Backup import must reject path traversal attempt");
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
