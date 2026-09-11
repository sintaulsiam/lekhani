//! Lekhani CLI Tool

mod converter;

use clap::{Parser, Subcommand};
use lekhani_core::{bijoy_to_unicode, unicode_to_bijoy, InputSession, UserStats};
use lekhani_settings::{ConfigManager, LayoutManager};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lekhani")]
#[command(about = "Lekhani Bengali Input Method & Productivity CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Transliterate phonetic English text to Unicode Bengali
    Test {
        /// Text to convert
        text: String,
    },
    /// Convert legacy Bijoy (ANSI) text to Unicode Bengali
    BijoyToUnicode {
        /// Text to convert
        text: String,
    },
    /// Convert Unicode Bengali text to Bijoy (ANSI)
    UnicodeToBijoy {
        /// Text to convert
        text: String,
    },
    /// List all discovered system and user keyboard layouts
    ListLayouts,
    /// Convert an Avro Keyboard 5 (.avrolayout) XML file to Lekhani JSON format
    ConvertLayout {
        /// Path to .avrolayout file
        input: PathBuf,
        /// Optional output path
        output: Option<PathBuf>,
    },
    /// Export or import user settings, custom layouts, autocorrect, and typing stats
    Sync {
        /// Export backup bundle to specified JSON file
        #[arg(short, long)]
        export: Option<PathBuf>,
        /// Import and restore backup bundle from specified JSON file
        #[arg(short, long)]
        import: Option<PathBuf>,
    },
    /// Manage custom user dictionary and autocorrect shortcuts
    Dict {
        #[command(subcommand)]
        subcommand: DictCommands,
    },
    /// View typing metrics, keystrokes saved, and top frequent words
    Stats {
        /// Reset user typing statistics
        #[arg(long)]
        reset: bool,
    },
    /// Benchmark typing engine performance
    Benchmark,
}

#[derive(Subcommand)]
enum DictCommands {
    /// List all custom user autocorrect entries
    List,
    /// Add or update a custom autocorrect entry
    Add {
        /// Trigger string (Latin phonetic or shortcut)
        trigger: String,
        /// Replacement string (Bengali Unicode or phrase)
        replacement: String,
    },
    /// Remove a custom autocorrect entry
    Remove {
        /// Trigger string to delete
        trigger: String,
    },
    /// Export custom dictionary to a JSON file
    Export {
        /// Destination JSON path
        path: PathBuf,
    },
    /// Import custom dictionary from a JSON file
    Import {
        /// Source JSON path
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut config_mgr = ConfigManager::new();
    let mut layout_mgr = LayoutManager::new();
    layout_mgr.discover_layouts(
        ConfigManager::get_system_layout_dir(),
        config_mgr.get_user_layout_dir(),
    );

    match cli.command {
        Commands::Test { text } => {
            let mut session = InputSession::new();
            let system_data = ConfigManager::get_system_data_dir();
            let user_ac = config_mgr.get_user_autocorrect_path();
            session.load_database(&system_data);
            session.load_user_autocorrect(&user_ac);

            if let Some(json) = layout_mgr.load_layout_json("Avro Phonetic") {
                session.set_layout(lekhani_core::ActiveLayoutType::Phonetic, &json);
            }
            let converted = session.phonetic.suggestion_engine.transliterate_phrase_or_sentence(&text);
            let empty_mem = hashbrown::HashMap::new();
            let (cands, _) = session.phonetic.suggestion_engine.suggest(&text, true, true, &empty_mem);
            println!("Input:       {}", text);
            println!("Output:      {}", converted);
            println!("Candidates:  {:?}", cands);
        }
        Commands::BijoyToUnicode { text } => {
            let res = bijoy_to_unicode(&text);
            println!("{}", res);
        }
        Commands::UnicodeToBijoy { text } => {
            let res = unicode_to_bijoy(&text);
            println!("{}", res);
        }
        Commands::ListLayouts => {
            println!("Installed Lekhani Keyboard Layouts:");
            for layout in layout_mgr.get_layout_list() {
                if let Some(info) = layout_mgr.get_layout(&layout) {
                    println!("  • {} (v{}, type: {})", info.name, info.version, info.layout_type);
                }
            }
        }
        Commands::ConvertLayout { input, output } => {
            let out = output.unwrap_or_else(|| input.with_extension("json"));
            let json_val = converter::convert_avro_layout_file(&input)?;
            let json_str = serde_json::to_string_pretty(&json_val)?;
            std::fs::write(&out, json_str)?;
            println!("Converted {:?} -> {:?}", input, out);
            println!("Conversion completed successfully!");
        }
        Commands::Sync { export, import } => {
            if let Some(dest) = export {
                config_mgr.export_backup(&dest).map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("Backup successfully exported to: {:?}", dest);
            } else if let Some(src) = import {
                config_mgr.import_backup(&src).map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("Backup successfully imported from: {:?}", src);
            } else {
                eprintln!("Error: Please specify either --export <file.json> or --import <file.json>");
            }
        }
        Commands::Dict { subcommand } => {
            let ac_path = config_mgr.get_user_autocorrect_path();
            let mut map: std::collections::BTreeMap<String, String> = if ac_path.exists() {
                let content = std::fs::read_to_string(&ac_path).unwrap_or_default();
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                std::collections::BTreeMap::new()
            };

            match subcommand {
                DictCommands::List => {
                    println!("Custom User AutoCorrect & Dictionary Entries ({} total):", map.len());
                    for (trigger, rep) in &map {
                        println!("  {}  ➔  {}", trigger, rep);
                    }
                }
                DictCommands::Add { trigger, replacement } => {
                    map.insert(trigger.clone(), replacement.clone());
                    let json = serde_json::to_string_pretty(&map)?;
                    if let Some(p) = ac_path.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    std::fs::write(&ac_path, json)?;
                    println!("Added entry: {} ➔ {}", trigger, replacement);
                }
                DictCommands::Remove { trigger } => {
                    if map.remove(&trigger).is_some() {
                        let json = serde_json::to_string_pretty(&map)?;
                        std::fs::write(&ac_path, json)?;
                        println!("Removed entry: {}", trigger);
                    } else {
                        println!("Entry '{}' not found in custom dictionary.", trigger);
                    }
                }
                DictCommands::Export { path } => {
                    let json = serde_json::to_string_pretty(&map)?;
                    if let Some(p) = path.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    std::fs::write(&path, json)?;
                    println!("Exported {} entries to {:?}", map.len(), path);
                }
                DictCommands::Import { path } => {
                    let content = std::fs::read_to_string(&path)?;
                    let imported: std::collections::BTreeMap<String, String> = serde_json::from_str(&content)?;
                    let count = imported.len();
                    for (k, v) in imported {
                        map.insert(k, v);
                    }
                    let json = serde_json::to_string_pretty(&map)?;
                    if let Some(p) = ac_path.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    std::fs::write(&ac_path, json)?;
                    println!("Imported {} entries from {:?}. Total active: {}", count, path, map.len());
                }
            }
        }
        Commands::Stats { reset } => {
            let stats_path = config_mgr.get_data_dir().join("stats.json");
            if reset {
                let stats = UserStats::new();
                let _ = stats.save_to_path(&stats_path);
                println!("User typing statistics have been reset.");
            } else {
                let stats = UserStats::load_from_path(&stats_path);
                println!("╔══════════════════════════════════════════════════════╗");
                println!("║         📊 Lekhani Personal Typing Dashboard         ║");
                println!("╠══════════════════════════════════════════════════════╣");
                println!("║ Total Words Typed:      {:>28} ║", stats.total_words_typed);
                println!("║ Total Keystrokes:       {:>28} ║", stats.total_keystrokes);
                println!("║ Keystrokes Saved:       {:>28} ║", stats.keystrokes_saved);
                println!("║ Typing Efficiency Gain: {:>27.1}% ║", stats.savings_percentage());
                println!("╠══════════════════════════════════════════════════════╣");
                println!("║ Top Frequent Words:                                  ║");
                let top = stats.get_top_words(5);
                if top.is_empty() {
                    println!("║   (No typing history recorded yet)                   ║");
                } else {
                    for (i, (word, count)) in top.iter().enumerate() {
                        println!("║   {}. {:<20} ({:>5} times)             ║", i + 1, word, count);
                    }
                }
                println!("╚══════════════════════════════════════════════════════╝");
            }
        }
        Commands::Benchmark => {
            println!("Running Lekhani typing engine benchmark...");
            let start = std::time::Instant::now();
            let session = InputSession::new();
            for _ in 0..10_000 {
                let _ = session.phonetic.suggestion_engine.convert_phonetic("amader bangladesh");
            }
            let elapsed = start.elapsed();
            println!("10,000 sentences converted in {:?}", elapsed);
            println!("Average latency per sentence: {:?}", elapsed / 10_000);
        }
    }

    Ok(())
}
