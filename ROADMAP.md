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

## 📋 Status Matrix

| Feature | Target Component | Status |
| :--- | :--- | :--- |
| **Fuzzy Phonetic Rules** | `lekhani-core` | ✅ Implemented |
| **Inline Math Calculator** | `lekhani-core` | ✅ Implemented |
| **Smart Typography & Bengali Dari (`..` ➔ `।`)** | `lekhani-core` | ✅ Implemented |
| **Interactive Key Highlighter** | `lekhani-gui` (Slint) | ✅ Implemented |
| **Dictionary Manager GUI & CLI** | `lekhani-gui` / `lekhani-cli` | ✅ Implemented |
| **Theme Presets (Catppuccin/Nord)** | `lekhani-gui` / `fcitx5` | ✅ Implemented |
| **Adaptive Frequency Engine & User Stats** | `lekhani-core` | ✅ Implemented |
| **Sync & Backup CLI (`sync --export/import`)** | `lekhani-cli` / `lekhani-settings` | ✅ Implemented |
| **Personal Typing Dashboard (`lekhani stats`)** | `lekhani-cli` | ✅ Implemented |
| **Neural Context Disambiguation** | `lekhani-ai` (Candle/Tract) | 🗓️ Planned |
| **Autonomous Dictionary Expansion** | `lekhani-core` / `lekhani-ai` | 🗓️ Planned |
| **Neural Next-Word Prediction** | `lekhani-ai` | 🗓️ Planned |
| **Offline Voice Typing** | `lekhani-voice` | 🗓️ Planned |

