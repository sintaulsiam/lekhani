# 🗺️ Lekhani (লেখনী) - Future Development Roadmap

This document outlines the planned architectural improvements, intelligence features, and desktop ergonomics inspired by modern input methods (Rime, Gboard, Mozc, Avro Windows, and macOS).

---

## 📌 Phase 1: Phonetic Intelligence & Power-User Tools

### 1.1 🔀 Fuzzy Phonetic Matching Engine
- **Objective:** Forgive common phoneme spelling variations in Latin letters.
- **Rules:**
  - `s` ⇄ `sh` ⇄ `ss` (`শ` / `ষ` / `স`)
  - `z` ⇄ `j` (`জ` / `য`)
  - `i` ⇄ `ee` ⇄ `ii` (`ই` / `ঈ`)
  - `u` ⇄ `oo` ⇄ `uu` (`উ` / `ঊ`)
  - `n` ⇄ `N` (`ন` / `ণ`)
  - `r` ⇄ `rr` ⇄ `rrh` (`র` / `ড়` / `ঢ়`)
- **Target:** `crates/lekhani-core/src/phonetic/`

### 1.2 🧮 Inline Math Calculator & Unit Converter
- **Objective:** Evaluate inline mathematical formulas and conversions directly in the preedit candidate buffer.
- **Triggers:**
  - `=125*8` ➔ Suggests `১,০০০` and `1000`
  - `=sqrt(144)` ➔ Suggests `১২` and `12`
  - `#usd50` ➔ Suggests `৳৬,১০০`
- **Target:** `crates/lekhani-core/src/snippets.rs`

### 1.3 ✍️ Smart Typography & Punctuation
- Automatic pairing for smart curly quotes (`“` `”` / `‘` `’`)
- Double hyphen expansion `--` ➔ Em-dash `—`
- Automatic bullet conversion `* ` ➔ `• `

---

## 🎨 Phase 2: UI & Interactive Desktop Ergonomics

### 2.1 🎯 Live Key Press Highlighting in Layout Viewer
- Real-time visualization lighting up on-screen keys in `lekhani-gui` when physical keyboard keys are pressed.
- Dynamic color-coding for Shift and AltGr (Level 3 Shift) key states.
- **Target:** `crates/lekhani-gui/ui/layout_viewer.slint`

### 2.2 📚 Custom Word & Dictionary Manager in GUI
- Add/Edit/Delete custom user dictionary entries visually in the Slint GUI.
- Import and export dictionary entries via JSON/CSV.
- **Target:** `crates/lekhani-gui/`

### 2.3 🎨 Modern Desktop Theme Presets
- Built-in theme packs matching popular desktop color schemes:
  - **Catppuccin (Mocha/Latte)**
  - **Nord**
  - **Tokyo Night**
  - **KDE Breeze Dark / Light**
  - **GNOME Adwaita**
- **Target:** `data/fcitx5/` & `crates/lekhani-gui/ui/theme.slint`

---

## 🧠 Phase 3: Adaptive Learning & Cross-Device Sync

### 3.1 📈 Adaptive Frequency Learning
- Dynamically track user word frequencies.
- Automatically promote frequently typed words and newly learned names to rank #1.
- **Target:** `crates/lekhani-core/src/ngram.rs`

### 3.2 ☁️ User Data Backup & Sync CLI
- One-line command to export and import user settings, custom layouts, and learned dictionaries:
  ```bash
  lekhani sync --export ./my_lekhani_backup.json
  lekhani sync --import ./my_lekhani_backup.json
  ```
- **Target:** `crates/lekhani-cli/`

### 3.3 📊 Personal Typing Dashboard & Statistics
- Track total Bengali words typed, keystrokes saved via suggestions, and typing speed (WPM).
- **Target:** `crates/lekhani-gui/`

---

## 🤖 Phase 4: On-Device AI & Contextual Neural Engine

### 4.1 🧠 Pure-Rust On-Device Neural Disambiguator (`lekhani-ai`)
- **Objective:** 100% offline, privacy-first contextual disambiguation using a quantized lightweight neural language model (leveraging `candle` or `tract`).
- **Capabilities:**
  - **Context-Aware Homophone Selection:** Resolves ambiguous words based on surrounding context in the sentence:
    - *বই পড়া* (Reading book) vs *শার্ট পরা* (Wearing shirt)
    - *কাজ করে* (Doing work) vs *কোর খাতা*
    - *মন ভালো* vs *বন বাদাড়*
  - **Next-Word Neural Prediction:** Predicts the subsequent 1–3 Bengali words based on grammatical context and conversational cadence (e.g., typing *আমি ভাত* predicts *খাচ্ছি*, *খাব*, *খেয়েছি*).
  - **Zero Telemetry / Total Privacy:** Entire inference runs locally on CPU with INT8/INT4 quantization (<15MB RAM footprint, <5ms latency).
- **Target:** `crates/lekhani-ai/`

### 4.2 📖 Autonomous Dictionary Expansion & Morphological Learner
- **Objective:** Automatically discover and learn new vocabulary, proper names, technical terms, and regional slang without requiring manual user configuration.
- **Workflow:**
  - Detects repeated unrecognized phonetic input patterns.
  - Performs Bengali morphological stem/suffix extraction (e.g., detecting *কুয়েটে* ➔ stems to *কুয়েট* + inflection `-এ`).
  - Persists new words to a local SQLite/KV store (`~/.local/share/lekhani/user_learned.db`).
  - Automatically indexes discovered roots into the flat binary-search dictionary trie at runtime.
- **Target:** `crates/lekhani-core/src/phonetic/` & `crates/lekhani-ai/`

### 4.3 🎙️ Offline Bengali Voice Typing Bridge
- **Objective:** Local speech-to-text dictation with no internet connection required.
- **Integration:** Lightweight local `whisper.cpp` / `sherpa-onnx` model fine-tuned for Bengali speech.
- **UX:** Global push-to-talk hotkey (e.g., `Super + H`) for real-time speech streaming directly into the active X11/Wayland input field.
- **Target:** `crates/lekhani-voice/`

---

## ⚡ Phase 5: High-Performance Architecture & Binary Serialization

### 5.1 🚀 Memory-Mapped Finite State Transducer (FST / Binary Trie)
- **Objective:** Eliminate JSON deserialization on startup, achieving instant cold-boot under 0.5ms and a minimal RAM footprint.
- **Approach:**
  - Compile the 160,000+ word dictionary into an immutable, memory-mapped binary FST format (`dictionary.fst`).
  - Zero-copy runtime lookups with deterministic sub-millisecond prefix searches.
- **Target:** `crates/lekhani-core/src/trie.rs`

### 5.2 🏎️ SIMD-Vectorized Preceding Context Evaluation
- **Objective:** Hardware-accelerated n-gram scoring for large sentence sequence decoding.
- **Capabilities:** Vectorized bigram and trigram transition lookups for multi-token beam search in `lekhani-ai`.
- **Target:** `crates/lekhani-ai/src/lm.rs`

---

## 🖥️ Phase 6: Desktop Ergonomics & Advanced User Experience

### 6.1 🔔 On-Screen Display (OSD) / Mode Switch Indicator
- **Objective:** Lightweight, non-intrusive floating toast/HUD near the cursor indicating the active language state (`বাংলা` / `ENG`) upon pressing <kbd>F12</kbd>.
- **Compositor Support:** Native Wayland (Layer Shell) and X11 transient window support.
- **Target:** `crates/lekhani-gui/` & `crates/lekhani-fcitx5/`

### 6.2 ⌨️ Configurable Global Hotkeys
- **Objective:** Customizable global mode-switch hotkeys configured via `~/.config/lekhani/config.toml` or the settings GUI.
- **Options:** Support for <kbd>F12</kbd>, <kbd>Ctrl+Space</kbd>, <kbd>Shift+Space</kbd>, and <kbd>Super+Space</kbd>.
- **Target:** `crates/lekhani-settings/` & `crates/lekhani-gui/`

### 6.3 📊 Interactive GUI Typing Analytics & WPM Heatmap
- **Objective:** Visual analytics dashboard in `lekhani-gui` presenting real-time typing speed (WPM), historical volume, and keystroke savings graphs.
- **Target:** `crates/lekhani-gui/ui/stats.slint`

---

## 🔤 Phase 7: Layout Expansion & Document Processing

### 7.1 ⌨️ Unijoy Fixed Keyboard Layout
- **Objective:** Built-in standard Unijoy layout support alongside Probhat, Jatiya (National), Munir Optima, Borno, and Avro Easy.
- **Target:** `data/layouts/unijoy.json` & `crates/lekhani-core/src/fixed/`

### 7.2 🔄 Batch File Converter CLI
- **Objective:** Fast CLI batch conversion of legacy SutonnyMJ/Bijoy documents to standard Unicode for entire directories and files (`.txt`, `.docx`, `.srt`).
- **Target:** `crates/lekhani-cli/`

### 7.3 🗣️ Colloquial & Regional Dialect Adaptation
- **Objective:** Contextual suggestions for spoken Bengali and regional verbal forms (e.g., `kortesi` ➔ `করছি`/`করতেছি`, `khaitesi` ➔ `খাচ্ছি`/`খাইতেছি`).
- **Target:** `crates/lekhani-core/src/phonetic/fuzzy.rs`

---

## 📋 Status Matrix

| Feature | Target Component | Status |
| :--- | :--- | :--- |
| **Fuzzy Phonetic Rules & Ergonomics** | `lekhani-core` | ✅ Implemented |
| **Embedded Core Suffix Fallback** | `lekhani-core` | ✅ Implemented |
| **Inline Math Calculator** | `lekhani-core` | ✅ Implemented |
| **Smart Typography & Bengali Dari (`..` ➔ `।`)** | `lekhani-core` | ✅ Implemented |
| **Interactive Key Highlighter** | `lekhani-gui` (Slint) | ✅ Implemented |
| **Dictionary Manager GUI & CLI** | `lekhani-gui` / `lekhani-cli` | ✅ Implemented |
| **Theme Presets (Catppuccin/Nord)** | `lekhani-gui` / `fcitx5` | ✅ Implemented |
| **Adaptive Frequency Engine & User Stats** | `lekhani-core` | ✅ Implemented |
| **Sync & Backup CLI (`sync --export/import`)** | `lekhani-cli` / `lekhani-settings` | ✅ Implemented |
| **Personal Typing Dashboard (`lekhani stats`)** | `lekhani-cli` | ✅ Implemented |
| **Autonomous Dictionary Expansion & Morphological Learner** | `lekhani-core` | ✅ Implemented |
| **Statistical & Neural Context Disambiguation** | `lekhani-ai` | ✅ Implemented |
| **Zero-Preedit Next-Word Prediction** | `lekhani-ai` | ✅ Implemented |
| **Global Beam Search Sequence Decoder** | `lekhani-ai` | ✅ Implemented |
| **Memory-Mapped Binary FST Dictionary** | `lekhani-core` | 🗓️ Planned |
| **SIMD-Vectorized Context Evaluation** | `lekhani-ai` | 🗓️ Planned |
| **OSD / Floating Mode Switch Indicator** | `lekhani-gui` / `fcitx5` | 🗓️ Planned |
| **Configurable Global Hotkeys** | `lekhani-settings` / `lekhani-gui` | 🗓️ Planned |
| **Interactive GUI Analytics Dashboard** | `lekhani-gui` | 🗓️ Planned |
| **Unijoy Keyboard Layout** | `lekhani-core` / `data` | 🗓️ Planned |
| **Batch Document Converter CLI** | `lekhani-cli` | 🗓️ Planned |
| **Colloquial & Regional Dialect Adaptation** | `lekhani-core` | 🗓️ Planned |
| **Offline Voice Typing Bridge** | `lekhani-voice` | 🗓️ Planned |


