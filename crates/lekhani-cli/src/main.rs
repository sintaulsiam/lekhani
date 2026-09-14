//! Lekhani CLI Tool

mod converter;

use clap::{Parser, Subcommand};
use lekhani_core::{
    bijoy_to_unicode, unicode_to_bijoy, AutonomousLearner, InputSession, UserStats,
};
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
    /// Convert a text file or standard input between Bijoy (ANSI) and Unicode
    ConvertFile {
        /// Input file path (use '-' for standard input)
        #[arg(short, long)]
        input: PathBuf,
        /// Output file path (defaults to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Reverse direction: convert Unicode to Bijoy (ANSI)
        #[arg(short, long)]
        reverse: bool,
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
    /// On-Device Bengali AI & Language Model Tools
    Ai {
        #[command(subcommand)]
        subcommand: AiCommands,
    },
    /// Show version, author, and special acknowledgments
    About,
}

#[derive(Subcommand)]
enum AiCommands {
    /// Predict the top next words given preceding sentence context
    Predict {
        /// Preceding sentence or phrase (e.g. "আমি ভাত" or "বাংলাদেশ একটি")
        context: String,
        /// Number of predictions to return (default: 5)
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
    /// Score sentence perplexity and conditional probabilities
    Score {
        /// Bengali sentence or phrase to score
        sentence: String,
    },
    /// Disambiguate and decode a sequence of candidate options
    Decode {
        /// JSON array of candidate vectors (e.g. '[["আমি"],["শার্ট"],["পড়া","পরা"]]')
        sequence_json: String,
    },
    /// Pretrain or seed the user's personal model with rich baseline conversational phrases
    Pretrain,
    /// Train the statistical N-gram language model or user personal memory on a raw Bengali text corpus
    Train {
        /// Input text file containing Bengali corpus
        #[arg(short, long)]
        input: PathBuf,
        /// Optional output path to save compiled model JSON
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Ingest directly into active user personal vocabulary and bigram memory
        #[arg(short, long)]
        user: bool,
    },
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
            let converted = session
                .phonetic
                .suggestion_engine
                .transliterate_phrase_or_sentence(&text);
            let empty_mem = hashbrown::HashMap::new();
            let cands = if text.contains(' ') {
                vec![converted.clone(), text.clone()]
            } else {
                let (c, _) = session
                    .phonetic
                    .suggestion_engine
                    .suggest(&text, true, true, &empty_mem);
                c
            };
            println!("Input:       {}", text);
            println!("Output:      {}", converted);
            let formatted_cands = format!(
                "[{}]",
                cands
                    .iter()
                    .map(|c| format!("\"{}\"", c))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!("Candidates:  {}", formatted_cands);
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
                    println!(
                        "  • {} (v{}, type: {})",
                        info.name, info.version, info.layout_type
                    );
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
        Commands::ConvertFile {
            input,
            output,
            reverse,
        } => {
            let input_content = if input.to_str() == Some("-") {
                use std::io::Read;
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                std::fs::read_to_string(&input)?
            };

            let converted = if reverse {
                unicode_to_bijoy(&input_content)
            } else {
                bijoy_to_unicode(&input_content)
            };

            if let Some(out_path) = output {
                std::fs::write(&out_path, &converted)?;
                println!(
                    "Converted {:?} -> {:?} ({} bytes)",
                    input,
                    out_path,
                    converted.len()
                );
            } else {
                print!("{}", converted);
            }
        }
        Commands::Sync { export, import } => {
            if let Some(dest) = export {
                config_mgr
                    .export_backup(&dest)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("Backup successfully exported to: {:?}", dest);
            } else if let Some(src) = import {
                config_mgr
                    .import_backup(&src)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("Backup successfully imported from: {:?}", src);
            } else {
                eprintln!(
                    "Error: Please specify either --export <file.json> or --import <file.json>"
                );
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
                    println!(
                        "Custom User AutoCorrect & Dictionary Entries ({} total):",
                        map.len()
                    );
                    for (trigger, rep) in &map {
                        println!("  {}  ➔  {}", trigger, rep);
                    }
                }
                DictCommands::Add {
                    trigger,
                    replacement,
                } => {
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
                    let imported: std::collections::BTreeMap<String, String> =
                        serde_json::from_str(&content)?;
                    let count = imported.len();
                    for (k, v) in imported {
                        map.insert(k, v);
                    }
                    let json = serde_json::to_string_pretty(&map)?;
                    if let Some(p) = ac_path.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    std::fs::write(&ac_path, json)?;
                    println!(
                        "Imported {} entries from {:?}. Total active: {}",
                        count,
                        path,
                        map.len()
                    );
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
                println!(
                    "║ Total Words Typed:      {:>28} ║",
                    stats.total_words_typed
                );
                println!("║ Total Keystrokes:       {:>28} ║", stats.total_keystrokes);
                println!("║ Keystrokes Saved:       {:>28} ║", stats.keystrokes_saved);
                println!(
                    "║ Typing Efficiency Gain: {:>27.1}% ║",
                    stats.savings_percentage()
                );
                println!("╠══════════════════════════════════════════════════════╣");
                println!("║ Top Frequent Words:                                  ║");
                let top = stats.get_top_words(5);
                if top.is_empty() {
                    println!("║   (No typing history recorded yet)                   ║");
                } else {
                    for (i, (word, count)) in top.iter().enumerate() {
                        println!(
                            "║   {}. {:<20} ({:>5} times)             ║",
                            i + 1,
                            word,
                            count
                        );
                    }
                }
                println!("╠══════════════════════════════════════════════════════╣");
                println!("║ 🧠 Autonomous Intelligence & Personalization:        ║");
                let learned_path = config_mgr.get_user_learned_path();
                let learner = AutonomousLearner::load_from_path(&learned_path);
                println!(
                    "║ Learned Vocabulary:     {:>28} ║",
                    learner.learned_words.len()
                );
                println!(
                    "║ Personal Bigram Pairs:  {:>28} ║",
                    learner.user_bigrams.len()
                );
                println!(
                    "║ Distinct Words Tracked: {:>28} ║",
                    learner.observed_counts.len()
                );
                if !learner.learned_words.is_empty() {
                    println!("║ Recent Learned Vocabulary:                           ║");
                    let mut recent: Vec<_> = learner.learned_words.iter().collect();
                    recent.sort();
                    for (i, w) in recent.iter().take(5).enumerate() {
                        println!("║   {}. {:<44} ║", i + 1, w);
                    }
                }
                println!("╚══════════════════════════════════════════════════════╝");
                println!("\nStorage Locations:");
                println!("  • User Vocabulary & Bigrams: {}", learned_path.display());
                println!("  • Raw Typing Metrics:        {}", stats_path.display());
            }
        }
        Commands::Benchmark => {
            println!("Running Lekhani typing engine benchmark...");
            let start = std::time::Instant::now();
            let session = InputSession::new();
            for _ in 0..10_000 {
                let _ = session
                    .phonetic
                    .suggestion_engine
                    .convert_phonetic("amader bangladesh");
            }
            let elapsed = start.elapsed();
            println!("10,000 sentences converted in {:?}", elapsed);
            println!("Average latency per sentence: {:?}", elapsed / 10_000);
        }
        Commands::Ai { subcommand } => {
            let lm = lekhani_ai::LanguageModel::new();
            let predictor = lekhani_ai::NextWordPredictor::new();
            let decoder = lekhani_ai::BeamSearchDecoder::new();

            match subcommand {
                AiCommands::Predict { context, limit } => {
                    let tokens: Vec<&str> = context.split_whitespace().collect();
                    let preds = predictor.predict_next(&tokens, limit);
                    println!("╔══════════════════════════════════════════════════════╗");
                    println!("║        🔮 Lekhani AI Next-Word Predictor             ║");
                    println!("╠══════════════════════════════════════════════════════╣");
                    println!("║ Context: \"{}\"", context);
                    println!("║ Top Predictions:                                     ║");
                    for (i, p) in preds.iter().enumerate() {
                        println!("║   {}. {:<44} ║", i + 1, p);
                    }
                    println!("╚══════════════════════════════════════════════════════╝");
                }
                AiCommands::Score { sentence } => {
                    let tokens: Vec<&str> = sentence.split_whitespace().collect();
                    println!("╔══════════════════════════════════════════════════════╗");
                    println!("║        🧠 Lekhani AI Language Model Scorer           ║");
                    println!("╠══════════════════════════════════════════════════════╣");
                    println!("║ Sentence: \"{}\"", sentence);
                    println!("║ Conditional Probabilities:                           ║");
                    let mut total_log_p = 0.0f32;
                    for (i, &w) in tokens.iter().enumerate() {
                        let prev1 = if i >= 1 { Some(tokens[i - 1]) } else { None };
                        let prev2 = if i >= 2 { Some(tokens[i - 2]) } else { None };
                        let log_p = lm.score_candidate(prev2, prev1, w);
                        total_log_p += log_p;
                        println!("║   • {:<16} (P = 10^{:<6.2})                  ║", w, log_p);
                    }
                    println!("╠══════════════════════════════════════════════════════╣");
                    println!("║ Total Sequence Log-Likelihood: {:>17.2} ║", total_log_p);
                    println!("╚══════════════════════════════════════════════════════╝");
                }
                AiCommands::Decode { sequence_json } => {
                    let seq: Vec<Vec<String>> = serde_json::from_str(&sequence_json)?;
                    let path = decoder.decode(&seq);
                    println!("Optimal Decoded Sequence: {:?}", path);
                    println!("Sentence: \"{}\"", path.join(" "));
                }
                AiCommands::Pretrain => {
                    let learned_path = config_mgr.get_user_learned_path();
                    let mut learner = AutonomousLearner::load_from_path(&learned_path);
                    learner.pretrain_baseline();
                    let _ = learner.save_to_path(&learned_path);

                    println!("╔══════════════════════════════════════════════════════╗");
                    println!("║      🚀 Lekhani Personal Pre-Training Complete       ║");
                    println!("╠══════════════════════════════════════════════════════╣");
                    println!("║ Baseline Vocabulary:    {:>28} ║", learner.learned_words.len());
                    println!("║ Baseline Bigram Pairs:  {:>28} ║", learner.user_bigrams.len());
                    println!("║ Stored Path:            {:<28} ║", learned_path.display());
                    println!("╚══════════════════════════════════════════════════════╝");
                }
                AiCommands::Train {
                    input,
                    output,
                    user,
                } => {
                    let text = std::fs::read_to_string(&input)?;
                    let mut trainer = lekhani_ai::CorpusTrainer::new();
                    trainer.train_text(&text);
                    let compiled = trainer.compile();

                    if user {
                        let learned_path = config_mgr.get_user_learned_path();
                        let mut learner = AutonomousLearner::load_from_path(&learned_path);
                        learner.train_text(&text);
                        let _ = learner.save_to_path(&learned_path);
                        println!(
                            "Ingested directly into user memory: {} vocabulary, {} bigrams.",
                            learner.learned_words.len(),
                            learner.user_bigrams.len()
                        );
                    }

                    println!("╔══════════════════════════════════════════════════════╗");
                    println!("║        🎓 Lekhani AI Corpus Training Complete        ║");
                    println!("╠══════════════════════════════════════════════════════╣");
                    println!("║ Corpus Source:      {:<32} ║", input.display());
                    println!("║ Total Words:        {:>32} ║", compiled.total_words);
                    println!("║ Unique Unigrams:    {:>32} ║", compiled.unigrams.len());
                    println!("║ Unique Bigrams:     {:>32} ║", compiled.bigrams.len());
                    println!("║ Unique Trigrams:    {:>32} ║", compiled.trigrams.len());
                    if let Some(out_path) = output {
                        let json = serde_json::to_string_pretty(&compiled)?;
                        if let Some(p) = out_path.parent() {
                            let _ = std::fs::create_dir_all(p);
                        }
                        std::fs::write(&out_path, json)?;
                        println!("║ Exported Model To:  {:<32} ║", out_path.display());
                    }
                    println!("╚══════════════════════════════════════════════════════╝");
                }
            }
        }
        Commands::About => {
            println!("╔══════════════════════════════════════════════════════════════════╗");
            println!("║                       Lekhani (লেখনী)                            ║");
            println!("║       Pure Rust Bengali Input Method & Desktop Suite             ║");
            println!("╠══════════════════════════════════════════════════════════════════╣");
            println!("║ Version:    v{:<51} ║", env!("CARGO_PKG_VERSION"));
            println!("║ Author:     Sintaul Mahdi Siam <sintaulsiam@gmail.com>           ║");
            println!("║ Company:    Syntenieum                                           ║");
            println!("║ Year:       2026                                                 ║");
            println!("║ Repository: https://github.com/sintaulsiam/lekhani               ║");
            println!("╠══════════════════════════════════════════════════════════════════╣");
            println!("║ Special Thanks & Acknowledgments:                                ║");
            println!("║   • Avro Keyboard (Mehdi Hasan Khan & OmicronLab):               ║");
            println!("║     For the pioneering Bengali phonetic layout and rules         ║");
            println!("║     that revolutionized digital Bengali computing.               ║");
            println!("║   • OpenBangla Keyboard (Muhammad Mominul Huq & OpenBangla Team):║");
            println!("║     For pioneering native Linux Bengali input methods            ║");
            println!("║     and laying foundational open-source architecture.            ║");
            println!("╚══════════════════════════════════════════════════════════════════╝");
        }
    }

    Ok(())
}
