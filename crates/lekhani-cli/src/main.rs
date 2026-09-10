//! Lekhani CLI Tool

mod converter;

use clap::{Parser, Subcommand};
use lekhani_core::{bijoy_to_unicode, unicode_to_bijoy, InputSession};
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
    /// Benchmark typing engine performance
    Benchmark,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_mgr = ConfigManager::new();
    let mut layout_mgr = LayoutManager::new();
    layout_mgr.discover_layouts(
        ConfigManager::get_system_layout_dir(),
        config_mgr.get_user_layout_dir(),
    );

    match cli.command {
        Commands::Test { text } => {
            let mut session = InputSession::new();
            if let Some(json) = layout_mgr.load_layout_json("Avro Phonetic") {
                session.set_layout(lekhani_core::ActiveLayoutType::Phonetic, &json);
            }
            let converted = session.phonetic.suggestion_engine.convert_phonetic(&text);
            println!("Input:     {}", text);
            println!("Output:    {}", converted);
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
