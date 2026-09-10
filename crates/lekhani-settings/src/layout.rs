//! Dynamic Layout Discovery and Indexer

use hashbrown::HashMap;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LayoutInfo {
    pub name: String,
    pub path: PathBuf,
    pub layout_type: String, // "phonetic" or "fixed"
    pub version: String,
    pub developer: String,
    pub comment: String,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutManager {
    layouts: HashMap<String, LayoutInfo>,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Discover layouts in system and user directories
    pub fn discover_layouts<P: AsRef<Path>>(&mut self, system_dir: P, user_dir: P) {
        self.layouts.clear();
        self.scan_dir(system_dir.as_ref());
        self.scan_dir(user_dir.as_ref());
    }

    fn scan_dir(&mut self, dir: &Path) {
        if !dir.exists() || !dir.is_dir() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(val) = serde_json::from_str::<Value>(&content) {
                            if let Some(info) = parse_layout_info(&path, &val) {
                                self.layouts.insert(info.name.clone(), info);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_layout_list(&self) -> Vec<String> {
        let mut list: Vec<String> = self.layouts.keys().cloned().collect();
        list.sort();
        list
    }

    pub fn get_layout(&self, name: &str) -> Option<&LayoutInfo> {
        self.layouts.get(name)
    }

    pub fn load_layout_json(&self, name: &str) -> Option<Value> {
        let info = self.layouts.get(name)?;
        let content = std::fs::read_to_string(&info.path).ok()?;
        serde_json::from_str(&content).ok()
    }
}

fn parse_layout_info(path: &Path, val: &Value) -> Option<LayoutInfo> {
    let info = val.get("info")?;
    let layout_type = info.get("type")?.as_str()?.to_string();
    let layout_obj = info.get("layout")?;
    let name = layout_obj.get("name")?.as_str()?.to_string();
    let version = layout_obj.get("version").and_then(|v| v.as_str()).unwrap_or("1.0").to_string();
    let developer = layout_obj
        .get("developer")
        .and_then(|d| d.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let comment = layout_obj
        .get("developer")
        .and_then(|d| d.get("comment"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(LayoutInfo {
        name,
        path: path.to_path_buf(),
        layout_type,
        version,
        developer,
        comment,
    })
}
