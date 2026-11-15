//! System Diagnostics & Environment Doctor for Lekhani
//!
//! Validates IME framework daemons (Fcitx5 / IBus), display server, environment
//! variables (GTK_IM_MODULE, QT_IM_MODULE, XMODIFIERS), installed Bengali fonts,
//! dictionary files, autostart entries, and outputs actionable fix suggestions.

use lekhani_settings::{ConfigManager, LayoutManager};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum Status {
    Ok,
    Warn,
    Fail,
    Info,
}

struct CheckItem {
    status: Status,
    category: &'static str,
    title: String,
    details: Vec<String>,
    fix: Option<String>,
}

impl CheckItem {
    fn ok(category: &'static str, title: impl Into<String>, details: Vec<String>) -> Self {
        Self {
            status: Status::Ok,
            category,
            title: title.into(),
            details,
            fix: None,
        }
    }

    fn warn(
        category: &'static str,
        title: impl Into<String>,
        details: Vec<String>,
        fix: Option<String>,
    ) -> Self {
        Self {
            status: Status::Warn,
            category,
            title: title.into(),
            details,
            fix,
        }
    }

    fn fail(
        category: &'static str,
        title: impl Into<String>,
        details: Vec<String>,
        fix: Option<String>,
    ) -> Self {
        Self {
            status: Status::Fail,
            category,
            title: title.into(),
            details,
            fix,
        }
    }

    fn info(category: &'static str, title: impl Into<String>, details: Vec<String>) -> Self {
        Self {
            status: Status::Info,
            category,
            title: title.into(),
            details,
            fix: None,
        }
    }
}

pub fn run_doctor() -> anyhow::Result<()> {
    println!("\x1b[1;36m===============================================================\x1b[0m");
    println!("\x1b[1;36m   Lekhani System Diagnostics & Health Check (Doctor)          \x1b[0m");
    println!("\x1b[1;36m   Modern Bengali Input Method Framework & Desktop Suite      \x1b[0m");
    println!("\x1b[1;36m===============================================================\x1b[0m\n");

    let mut checks: Vec<CheckItem> = Vec::new();

    // 1. Operating System & Session
    check_session(&mut checks);

    // 2. IME Daemons & Processes
    check_daemons(&mut checks);

    // 3. Environment Variables
    check_environment(&mut checks);

    // 4. IME Framework Plugins / Addons
    check_ime_plugins(&mut checks);

    // 5. Bengali Fonts
    check_fonts(&mut checks);

    // 6. Data Directories & Dictionaries
    check_data_files(&mut checks);

    // 7. Autostart Configuration
    check_autostart(&mut checks);

    // Print Results Grouped by Category
    let mut current_category = "";
    let mut warn_count = 0;
    let mut fail_count = 0;

    for item in &checks {
        if item.category != current_category {
            current_category = item.category;
            println!("\x1b[1;34m[ {} ]\x1b[0m", current_category);
        }

        let symbol = match item.status {
            Status::Ok => "\x1b[1;32m[✓]\x1b[0m",
            Status::Info => "\x1b[1;34m[i]\x1b[0m",
            Status::Warn => {
                warn_count += 1;
                "\x1b[1;33m[!]\x1b[0m"
            }
            Status::Fail => {
                fail_count += 1;
                "\x1b[1;31m[✗]\x1b[0m"
            }
        };

        println!("  {} {}", symbol, item.title);
        for detail in &item.details {
            println!("      \x1b[90m↳\x1b[0m {}", detail);
        }
        if let Some(fix) = &item.fix {
            println!("      \x1b[1;33mHint:\x1b[0m {}", fix);
        }
        println!();
    }

    // Summary Banner
    println!("\x1b[1;36m---------------------------------------------------------------\x1b[0m");
    if fail_count == 0 && warn_count == 0 {
        println!(
            "\x1b[1;32m✓ All system checks passed! Lekhani is ready for optimal typing.\x1b[0m"
        );
    } else if fail_count == 0 {
        println!(
            "\x1b[1;33m! System is functional with {} warning(s). Check hints above.\x1b[0m",
            warn_count
        );
    } else {
        println!(
            "\x1b[1;31m✗ {} failure(s) and {} warning(s) detected. Follow hints to resolve.\x1b[0m",
            fail_count, warn_count
        );
    }
    println!("\x1b[1;36m---------------------------------------------------------------\x1b[0m");

    Ok(())
}

fn check_session(checks: &mut Vec<CheckItem>) {
    let os_name = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".to_string());
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unknown".to_string());

    checks.push(CheckItem::info(
        "System & Display Session",
        format!("Platform: {} ({})", os_name, arch),
        vec![
            format!("Display Server: {}", session_type),
            format!("Desktop Environment: {}", desktop),
        ],
    ));
}

fn is_process_running(name: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(comm) = std::fs::read_to_string(path.join("comm")) {
                        if comm.trim() == name {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn check_daemons(checks: &mut Vec<CheckItem>) {
    let fcitx_running = is_process_running("fcitx5");
    let ibus_running = is_process_running("ibus-daemon");
    let gui_running = is_process_running("lekhani-gui");

    if fcitx_running {
        checks.push(CheckItem::ok(
            "IME Daemons & Processes",
            "Fcitx5 daemon is active and running",
            vec!["Process 'fcitx5' found in active session".to_string()],
        ));
    } else if ibus_running {
        checks.push(CheckItem::ok(
            "IME Daemons & Processes",
            "IBus daemon is active and running",
            vec!["Process 'ibus-daemon' found in active session".to_string()],
        ));
    } else {
        checks.push(CheckItem::warn(
            "IME Daemons & Processes",
            "No active IME daemon detected (neither Fcitx5 nor IBus)",
            vec![
                "Lekhani requires Fcitx5 or IBus to inject keystrokes into desktop apps"
                    .to_string(),
            ],
            Some("Start Fcitx5 with `fcitx5 -d` or configure your desktop autostart".to_string()),
        ));
    }

    if gui_running {
        checks.push(CheckItem::ok(
            "IME Daemons & Processes",
            "Lekhani TopBar & GUI is running",
            vec!["Process 'lekhani-gui' is active".to_string()],
        ));
    } else {
        checks.push(CheckItem::info(
            "IME Daemons & Processes",
            "Lekhani TopBar & GUI is not currently running",
            vec![
                "Launch `lekhani-gui` to show floating topbar, layout switcher, and settings"
                    .to_string(),
            ],
        ));
    }
}

fn check_environment(checks: &mut Vec<CheckItem>) {
    let gtk_im = std::env::var("GTK_IM_MODULE").ok();
    let qt_im = std::env::var("QT_IM_MODULE").ok();
    let xmods = std::env::var("XMODIFIERS").ok();

    let mut details = Vec::new();
    details.push(format!(
        "GTK_IM_MODULE: {}",
        gtk_im.as_deref().unwrap_or("<unset>")
    ));
    details.push(format!(
        "QT_IM_MODULE:  {}",
        qt_im.as_deref().unwrap_or("<unset>")
    ));
    details.push(format!(
        "XMODIFIERS:    {}",
        xmods.as_deref().unwrap_or("<unset>")
    ));

    let fcitx_running = is_process_running("fcitx5");
    let ibus_running = is_process_running("ibus-daemon");

    let is_fcitx_ok = xmods.as_deref() == Some("@im=fcitx")
        || (fcitx_running && qt_im.as_deref() == Some("fcitx"));
    let is_ibus_ok =
        xmods.as_deref() == Some("@im=ibus") || (ibus_running && qt_im.as_deref() == Some("ibus"));

    if is_fcitx_ok || is_ibus_ok {
        checks.push(CheckItem::ok(
            "Environment Variables",
            "Input method environment variables configured correctly",
            details,
        ));
    } else {
        checks.push(CheckItem::warn(
            "Environment Variables",
            "Input method environment variables may not be configured",
            details,
            Some("Add to ~/.profile or /etc/environment:\n        export GTK_IM_MODULE=fcitx\n        export QT_IM_MODULE=fcitx\n        export XMODIFIERS=@im=fcitx".to_string()),
        ));
    }
}

fn check_ime_plugins(checks: &mut Vec<CheckItem>) {
    let home = std::env::var("HOME").unwrap_or_default();
    let home_path = Path::new(&home);

    let addon_so_candidates = [
        PathBuf::from("/usr/lib/fcitx5/fcitx5-lekhani.so"),
        PathBuf::from("/usr/lib64/fcitx5/fcitx5-lekhani.so"),
        PathBuf::from("/usr/lib/x86_64-linux-gnu/fcitx5/fcitx5-lekhani.so"),
        PathBuf::from("/usr/local/lib/fcitx5/fcitx5-lekhani.so"),
        home_path.join(".local/lib/fcitx5/fcitx5-lekhani.so"),
    ];

    let mut found_addon = None;
    for p in &addon_so_candidates {
        if p.exists() {
            found_addon = Some(p.clone());
            break;
        }
    }

    let addon_conf_candidates = [
        PathBuf::from("/usr/share/fcitx5/addon/lekhani.conf"),
        PathBuf::from("/usr/local/share/fcitx5/addon/lekhani.conf"),
        home_path.join(".local/share/fcitx5/addon/lekhani.conf"),
    ];
    let found_conf = addon_conf_candidates.iter().find(|p| p.exists());

    let ibus_component_candidates = [
        PathBuf::from("/usr/share/ibus/component/lekhani.xml"),
        PathBuf::from("/usr/local/share/ibus/component/lekhani.xml"),
        home_path.join(".local/share/ibus/component/lekhani.xml"),
    ];
    let found_ibus = ibus_component_candidates.iter().find(|p| p.exists());

    if let Some(so) = found_addon {
        let mut details = vec![format!("Fcitx5 shared library: {}", so.display())];
        if let Some(conf) = found_conf {
            details.push(format!("Addon descriptor: {}", conf.display()));
        }
        checks.push(CheckItem::ok(
            "Lekhani Addons & Integrations",
            "Fcitx5 Lekhani addon is installed",
            details,
        ));
    } else if let Some(ibus) = found_ibus {
        checks.push(CheckItem::ok(
            "Lekhani Addons & Integrations",
            "IBus Lekhani component is installed",
            vec![format!("Component XML: {}", ibus.display())],
        ));
    } else {
        checks.push(CheckItem::warn(
            "Lekhani Addons & Integrations",
            "No installed Fcitx5 or IBus Lekhani addon detected in system paths",
            vec![
                "Build and install lekhani-fcitx5 or lekhani-ibus to integrate with system IME"
                    .to_string(),
            ],
            Some(
                "Run `cmake -B build crates/lekhani-fcitx5 && sudo cmake --install build`"
                    .to_string(),
            ),
        ));
    }
}

fn check_fonts(checks: &mut Vec<CheckItem>) {
    let output = Command::new("fc-list")
        .args([":lang=bn", "family"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut families = std::collections::BTreeSet::new();
            for line in text.lines() {
                for name in line.split(',') {
                    let trimmed = name.trim();
                    if !trimmed.is_empty() {
                        families.insert(trimmed.to_string());
                    }
                }
            }

            if families.is_empty() {
                checks.push(CheckItem::warn(
                    "Bengali Typography & Fonts",
                    "No Bengali fonts detected by fontconfig",
                    vec![
                        "Bengali conjuncts and text may not render cleanly in applications"
                            .to_string(),
                    ],
                    Some(
                        "Install Bengali fonts: `sudo apt install fonts-noto-core fonts-kalpurush`"
                            .to_string(),
                    ),
                ));
            } else {
                let popular = [
                    "Noto Sans Bengali",
                    "Noto Serif Bengali",
                    "Kalpurush",
                    "SolaimanLipi",
                    "Siyam Rupali",
                ];
                let recommended_found: Vec<_> = popular
                    .iter()
                    .filter(|f| families.contains(**f))
                    .copied()
                    .collect();

                let mut details = vec![format!("Total Bengali font families: {}", families.len())];
                if !recommended_found.is_empty() {
                    details.push(format!(
                        "Recommended fonts: {}",
                        recommended_found.join(", ")
                    ));
                } else {
                    let sample: Vec<_> = families.iter().take(3).cloned().collect();
                    details.push(format!("Installed fonts: {}", sample.join(", ")));
                }

                checks.push(CheckItem::ok(
                    "Bengali Typography & Fonts",
                    "High-quality Bengali fonts found",
                    details,
                ));
            }
        }
        _ => {
            checks.push(CheckItem::info(
                "Bengali Typography & Fonts",
                "fc-list utility not available to verify fonts",
                vec!["Install `fontconfig` to enable font validation".to_string()],
            ));
        }
    }
}

fn check_data_files(checks: &mut Vec<CheckItem>) {
    let config_mgr = ConfigManager::new();
    let sys_data = ConfigManager::get_system_data_dir();
    let sys_layout = ConfigManager::get_system_layout_dir();
    let user_layout = config_mgr.get_user_layout_dir();

    let mut layout_mgr = LayoutManager::new();
    layout_mgr.discover_layouts(&sys_layout, &user_layout);

    let mut details = Vec::new();
    details.push(format!(
        "Discovered Layouts: {}",
        layout_mgr.get_layout_list().len()
    ));
    details.push(format!(
        "Active Layout: {}",
        config_mgr.config.general.active_layout
    ));

    let lm_bin = sys_data.join("bengali_lm.bin");
    let avro_dict = sys_data.join("avro_dict.json");
    if lm_bin.exists() {
        details.push(format!("Language Model: {} (Binary)", lm_bin.display()));
    } else if avro_dict.exists() {
        details.push(format!("Phonetic Dict: {} (JSON)", avro_dict.display()));
    }

    let ac_path = config_mgr.get_user_autocorrect_path();
    if ac_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&ac_path) {
            if let Ok(entries) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(map) = entries.as_object() {
                    details.push(format!("Custom AutoCorrect: {} entries", map.len()));
                }
            }
        }
    }

    let learned_path = config_mgr.get_user_learned_path();
    if learned_path.exists() {
        if let Ok(meta) = learned_path.metadata() {
            let kb = meta.len() / 1024;
            details.push(format!("Personal Learned Memory: {} KB", kb));
        }
    }

    if !lm_bin.exists() && !avro_dict.exists() {
        checks.push(CheckItem::fail(
            "Data Directories & Dictionaries",
            "Core Bengali dictionaries missing",
            vec![format!("Neither {} nor {} could be located", lm_bin.display(), avro_dict.display())],
            Some("Ensure Lekhani system data assets are installed in /usr/share/lekhani/data or current workspace data/".to_string()),
        ));
    } else {
        checks.push(CheckItem::ok(
            "Data Directories & Dictionaries",
            "Lekhani dictionaries and layout stores accessible",
            details,
        ));
    }
}

fn check_autostart(checks: &mut Vec<CheckItem>) {
    let home = std::env::var("HOME").unwrap_or_default();
    let autostart_dir = Path::new(&home).join(".config/autostart");
    let candidates = [
        autostart_dir.join("lekhani.desktop"),
        autostart_dir.join("lekhani-gui.desktop"),
        autostart_dir.join("io.github.lekhani.desktop"),
    ];

    let found = candidates.iter().find(|p| p.exists());
    if let Some(desktop_file) = found {
        checks.push(CheckItem::ok(
            "Autostart & Launch",
            "Desktop autostart entry configured",
            vec![format!("Desktop file: {}", desktop_file.display())],
        ));
    } else {
        checks.push(CheckItem::info(
            "Autostart & Launch",
            "Desktop autostart entry not present in ~/.config/autostart",
            vec![
                "Lekhani TopBar can be started manually or configured in desktop settings"
                    .to_string(),
            ],
        ));
    }
}
