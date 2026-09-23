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
    /// Train an on-device statistical language model from raw Bengali text
    Train {
        /// Input text file or directory containing Bengali corpus
        #[arg(short, long)]
        input: PathBuf,
        /// Optional output path to save compiled model (e.g. data/dictionaries/bengali_lm.bin)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Minimum frequency threshold for unigrams (default: 2)
        #[arg(long, default_value_t = 2)]
        min_unigram_freq: usize,
        /// Minimum frequency threshold for bigrams (default: 3)
        #[arg(long, default_value_t = 3)]
        min_bigram_freq: usize,
        /// Minimum frequency threshold for trigrams (default: 4)
        #[arg(long, default_value_t = 4)]
        min_trigram_freq: usize,
        /// Maximum vocabulary unigrams to retain (default: 120000)
        #[arg(long, default_value_t = 120000)]
        max_unigrams: usize,
        /// Maximum bigrams to retain (default: 600000)
        #[arg(long, default_value_t = 600000)]
        max_bigrams: usize,
        /// Maximum trigrams to retain (default: 1200000)
        #[arg(long, default_value_t = 1200000)]
        max_trigrams: usize,
        /// Ingest directly into active user personal vocabulary and bigram memory
        #[arg(short, long)]
        user: bool,
    },
    /// Evaluate the language model on Bengali test benchmarks
    Eval {
        /// Path to compiled binary model (optional, defaults to system/dev paths)
        #[arg(short, long)]
        model: Option<PathBuf>,
    },
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
        /// Input text file or directory containing Bengali corpus
        #[arg(short, long)]
        input: PathBuf,
        /// Optional output path to save compiled model (e.g. data/dictionaries/bengali_lm.bin)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Minimum frequency threshold for unigrams (default: 2)
        #[arg(long, default_value_t = 2)]
        min_unigram_freq: usize,
        /// Minimum frequency threshold for bigrams (default: 3)
        #[arg(long, default_value_t = 3)]
        min_bigram_freq: usize,
        /// Minimum frequency threshold for trigrams (default: 4)
        #[arg(long, default_value_t = 4)]
        min_trigram_freq: usize,
        /// Maximum vocabulary unigrams to retain (default: 120000)
        #[arg(long, default_value_t = 120000)]
        max_unigrams: usize,
        /// Maximum bigrams to retain (default: 600000)
        #[arg(long, default_value_t = 600000)]
        max_bigrams: usize,
        /// Maximum trigrams to retain (default: 1200000)
        #[arg(long, default_value_t = 1200000)]
        max_trigrams: usize,
        /// Ingest directly into active user personal vocabulary and bigram memory
        #[arg(short, long)]
        user: bool,
    },
    /// Evaluate the language model on Bengali test benchmarks
    Eval {
        /// Path to compiled binary model (optional, defaults to system/dev paths)
        #[arg(short, long)]
        model: Option<PathBuf>,
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
    #[cfg(windows)]
    unsafe {
        extern "system" {
            fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
            fn SetConsoleCP(wCodePageID: u32) -> i32;
        }
        let _ = SetConsoleOutputCP(65001);
        let _ = SetConsoleCP(65001);
    }

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
        Commands::Train {
            input,
            output,
            min_unigram_freq,
            min_bigram_freq,
            min_trigram_freq,
            max_unigrams,
            max_bigrams,
            max_trigrams,
            user,
        } => {
            run_train(
                input,
                output,
                min_unigram_freq,
                min_bigram_freq,
                min_trigram_freq,
                max_unigrams,
                max_bigrams,
                max_trigrams,
                user,
                &config_mgr,
            )?;
        }
        Commands::Eval { model } => {
            run_eval(model)?;
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
                    println!(
                        "║ Baseline Vocabulary:    {:>28} ║",
                        learner.learned_words.len()
                    );
                    println!(
                        "║ Baseline Bigram Pairs:  {:>28} ║",
                        learner.user_bigrams.len()
                    );
                    println!("║ Stored Path:            {:<28} ║", learned_path.display());
                    println!("╚══════════════════════════════════════════════════════╝");
                }
                AiCommands::Train {
                    input,
                    output,
                    min_unigram_freq,
                    min_bigram_freq,
                    min_trigram_freq,
                    max_unigrams,
                    max_bigrams,
                    max_trigrams,
                    user,
                } => {
                    run_train(
                        input,
                        output,
                        min_unigram_freq,
                        min_bigram_freq,
                        min_trigram_freq,
                        max_unigrams,
                        max_bigrams,
                        max_trigrams,
                        user,
                        &config_mgr,
                    )?;
                }
                AiCommands::Eval { model } => {
                    run_eval(model)?;
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

fn collect_text_files(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_text_files(&path, files);
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "txt" || ext == "corpus" || ext == "md" {
                        files.push(path);
                    }
                }
            }
        }
    }
}

fn run_train(
    input: PathBuf,
    output: Option<PathBuf>,
    min_unigram_freq: usize,
    min_bigram_freq: usize,
    min_trigram_freq: usize,
    max_unigrams: usize,
    max_bigrams: usize,
    max_trigrams: usize,
    user: bool,
    config_mgr: &ConfigManager,
) -> anyhow::Result<()> {
    use std::time::Instant;

    let start = Instant::now();
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║       🎓 Lekhani AI Corpus Training Engine           ║");
    println!("╠══════════════════════════════════════════════════════╣");
    println!("║ Ingesting source:   {:<32} ║", input.display());

    let mut corpus_files = Vec::new();
    if input.is_dir() {
        collect_text_files(&input, &mut corpus_files);
        corpus_files.sort();
    } else {
        corpus_files.push(input.clone());
    }

    if corpus_files.is_empty() {
        anyhow::bail!("No valid text files (.txt, .corpus, .md) found in {:?}", input);
    }

    let total_bytes: u64 = corpus_files
        .iter()
        .map(|f| std::fs::metadata(f).map(|m| m.len()).unwrap_or(0))
        .sum();

    println!("║ Total Files Read:   {:>32} ║", corpus_files.len());
    println!(
        "║ Total Size:         {:>29.2} MB ║",
        total_bytes as f64 / (1024.0 * 1024.0)
    );
    println!("║ Low-Memory Streaming Tokenizing (Rayon)...           ║");

    let config = lekhani_ai::TrainingConfig {
        min_unigram_freq,
        min_bigram_freq,
        min_trigram_freq,
        max_unigrams,
        max_bigrams,
        max_trigrams,
    };

    let compiled = lekhani_ai::train_files_streaming(&corpus_files, &config)?;

    if user {
        let learned_path = config_mgr.get_user_learned_path();
        let mut learner = AutonomousLearner::load_from_path(&learned_path);
        for f in &corpus_files {
            if let Ok(file) = std::fs::File::open(f) {
                let reader = std::io::BufReader::new(file);
                use std::io::BufRead;
                for line in reader.lines().flatten() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        learner.train_text(trimmed);
                    }
                }
            }
        }
        let _ = learner.save_to_path(&learned_path);
        println!(
            "║ User Memory:        {:>24} words ║",
            learner.learned_words.len()
        );
    }

    println!("╠══════════════════════════════════════════════════════╣");
    println!("║ Training Completed in {:>28.2?} ║", start.elapsed());
    println!("║ Total Tokens Processed:{:>29} ║", compiled.total_words);
    println!("║ Vocabulary (Unigrams): {:>29} ║", compiled.unigrams.len());
    println!("║ Transitions (Bigrams): {:>29} ║", compiled.bigrams.len());
    println!("║ Contexts (Trigrams):   {:>29} ║", compiled.trigrams.len());

    if let Some(out_path) = output {
        if let Some(p) = out_path.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        let is_binary = out_path.extension().and_then(|e| e.to_str()) == Some("bin")
            || out_path.extension().and_then(|e| e.to_str()) == Some("lm");

        if is_binary {
            compiled.save_binary(&out_path)?;
        } else {
            let json = serde_json::to_string_pretty(&compiled)?;
            std::fs::write(&out_path, json)?;
        }
        let out_bytes = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
        println!("║ Exported Output:    {:<32} ║", out_path.display());
        println!(
            "║ Binary Output Size: {:>29.2} MB ║",
            out_bytes as f64 / (1024.0 * 1024.0)
        );
    }
    println!("╚══════════════════════════════════════════════════════╝");
    Ok(())
}

fn run_eval(model_path: Option<PathBuf>) -> anyhow::Result<()> {
    use std::time::Instant;

    let (lm, source_desc) = if let Some(ref p) = model_path {
        let loaded = lekhani_ai::LanguageModel::from_binary_file(p)?;
        (loaded, format!("Custom Binary ({})", p.display()))
    } else {
        let loaded = lekhani_ai::LanguageModel::new();
        (loaded, "Auto-Probed System / Static Baseline".to_string())
    };

    let predictor = lekhani_ai::NextWordPredictor::with_language_model(lm.clone());

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║         🔬 Lekhani Bengali AI Model Evaluation       ║");
    println!("╠══════════════════════════════════════════════════════╣");
    println!("║ Active Model:       {:<32} ║", source_desc);
    println!("║ Unigram Vocabulary: {:>32} ║", lm.unigram_count());
    println!("║ Bigram Transitions: {:>32} ║", lm.bigram_count());
    println!("║ Trigram Contexts:   {:>32} ║", lm.trigram_count());
    println!("╠══════════════════════════════════════════════════════╣");
    println!("║ 1. Speed Benchmark (100,000 candidate evaluations):  ║");

    let eval_start = Instant::now();
    let sample_pairs = [
        (Some("আমি"), Some("ভাত"), "খাচ্ছি"),
        (Some("নতুন"), Some("জামা"), "পরা"),
        (Some("আমি"), Some("বই"), "পড়া"),
        (None, Some("চা"), "খাব"),
        (None, Some("ভালো"), "আছি"),
    ];
    let iters = 20_000;
    for _ in 0..iters {
        for &(p2, p1, w) in &sample_pairs {
            let _ = lm.score_candidate(p2, p1, w);
        }
    }
    let eval_duration = eval_start.elapsed();
    let total_evals = iters * sample_pairs.len();
    let per_eval_ns = eval_duration.as_nanos() as f64 / total_evals as f64;
    println!("║    Total Time: {:>37.2?} ║", eval_duration);
    println!("║    Latency / Evaluation: {:>29.2} ns ║", per_eval_ns);
    println!(
        "║    Throughput: {:>32.0} evals/sec ║",
        total_evals as f64 / eval_duration.as_secs_f64()
    );

    println!("╠══════════════════════════════════════════════════════╣");
    println!("║ 2. Homophone & Semantic Disambiguation Tests:        ║");

    let test_cases = [
        ("আমি বই ...", Some("আমি"), Some("বই"), "পড়া", "পরা"),
        ("নতুন জামা ...", Some("নতুন"), Some("জামা"), "পরা", "পড়া"),
        ("চা ...", None, Some("চা"), "খাব", "যাব"),
        ("ভালো ...", None, Some("ভালো"), "আছি", "খাচ্ছি"),
        (
            "বাংলাদেশ একটি ...",
            Some("বাংলাদেশ"),
            Some("একটি"),
            "সুন্দর",
            "খারাপ",
        ),
    ];

    let mut passed = 0;
    for (ctx_desc, p2, p1, expected, wrong) in test_cases {
        let score_exp = lm.score_candidate(p2, p1, expected);
        let score_wrg = lm.score_candidate(p2, p1, wrong);
        let ok = score_exp > score_wrg;
        if ok {
            passed += 1;
        }
        let status = if ok { "✅ PASS" } else { "❌ FAIL" };
        println!(
            "║  {} {:<18} \"{}\" > \"{}\" ({:>5.2} vs {:>5.2}) ║",
            status, ctx_desc, expected, wrong, score_exp, score_wrg
        );
    }
    println!(
        "║  Disambiguation Accuracy: {:>23} / {} ║",
        passed,
        test_cases.len()
    );

    println!("╠══════════════════════════════════════════════════════╣");
    println!("║ 3. Next-Word Prediction Tests:                       ║");
    let contexts = [
        vec!["আমি", "ভাত"],
        vec!["বাংলাদেশ", "একটি"],
        vec!["শুভ"],
    ];
    for ctx in &contexts {
        let preds = predictor.predict_next(ctx, 3);
        let formatted = preds.join(", ");
        println!("║  \"{}\" ➔ [{}]", ctx.join(" "), formatted);
    }
    println!("╚══════════════════════════════════════════════════════╝");
    Ok(())
}

