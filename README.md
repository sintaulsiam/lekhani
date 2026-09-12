# Lekhani (লেখনী)

<p align="center">
  <img src="data/icons/128.png" alt="Lekhani Logo" />
</p>

<p align="center">
  <b>Modern, Ultra-Fast Pure Rust Bengali Input Method & Desktop Suite for Linux</b>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="docs/TYPING_GUIDE.md">Typing Guide</a> •
  <a href="ARCHITECTURE.md">Architecture</a> •
  <a href="#installation">Installation</a> •
  <a href="#building-from-source">Building</a> •
  <a href="ROADMAP.md">Roadmap</a> •
  <a href="#license">License</a>
</p>

---

## ✨ Features

- **🚀 100% Pure Rust Architecture**: Zero C/C++ memory leaks, zero FFI overhead, and compiler-enforced memory safety.
- **🎙️ Full Avro Phonetic Support**: Complete compatibility with official Avro Phonetic typing rules.
- **🗣️ Colloquial & Spoken Verbal Conjugations**: Native recognition and suffix peeling for spoken dialect continuous and past forms (`kortesi` ➔ `করছি`/`করতেছি`, `jaitasi` ➔ `যাচ্ছি`/`যাইতেছি`, `khaitesi` ➔ `খাচ্ছি`/`খাইতেছি`, `boltase` ➔ `বলছে`/`বলতেছে`).
- **🧩 4-Layer Affix & Sandhi Engine**: Systematic classifier, plural, case, and particle composition with automatic phonetic Sandhi joins (`ভবিষ্যৎ` + `ের` ➔ `ভবিষ্যতের`, `রং` + `এর` ➔ `রঙের`, `পা` + `ে` ➔ `পায়ে`).
- **🔊 Canonical Bengali Phonetic Soundex**: $O(1)$ phonetic sound laws clustering for effortless homophone disambiguation (`বিদেশি` ↔ `বিদেশী`, `শহীদ` ↔ `সহিদ`).
- **⚡ Zero-Allocation Compact PrefixTrie**: Cache-line friendly contiguous memory layout with 12-byte packed entries for instant sub-millisecond dictionary queries.
- **🧠 Context-Aware Homophone Disambiguation**: Evaluates preceding words ($w_{t-1}$) to resolve homophones (*বই পড়া* vs *শার্ট পরা*, *বাংলা ভাষা* vs *ভেসে ভাসা*, *জীবনের লক্ষ্য* vs *এক লক্ষ*).
- **🔮 Zero-Preedit Next-Word Prediction**: Instant AI trigram continuations upon committing words (`আমি` ➔ `ভালো`, `তোমাকে`, `যাব`, `চাই`).
- **📖 Autonomous Morphological Learner**: Automatically extracts base stems and indexes new vocabulary dynamically from user typing.
- **🌐 Bilingual Code-Mixing & Tech Loanwords**: Dual candidate generation for English words (`meeting` ➔ `মিটিং` & `meeting`, `laptop`, `doctor`).
- **⌨️ Popular Fixed Keyboard Layouts**: Full built-in support for **Probhat**, **National (Jatiya)**, **Munir Optima**, **Borno**, **Unijoy**, and **Avro Easy**.
- **🪄 Advanced Typing Automations**:
  - Automatic Vowel Forming (অ + া ➔ আ)
  - Automatic Chandra Position Fixing (কঁ + া ➔ কাঁ)
  - Traditional Kar Joining (ligature blocking via ZWNJ)
  - Smart Old-style Reph (র্) insertion algorithm
  - Number pad Bengali digits auto-mapping (০-৯)
- **🌐 Dual Desktop Framework Support**:
  - **Fcitx5 Native Addon** (`fcitx5-lekhani.so`) for **KDE Plasma 6** and modern **Wayland** compositors (Hyprland, Sway).
  - **IBus Daemon** (`ibus-lekhani` via `zbus`) with real-time D-Bus signal emitters for GNOME and standard desktops.
- **🎨 Sleek Slint Native Desktop UI** (`lekhani-gui`):
  - Floating, frameless, draggable TopBar with always-on-top mode.
  - **Mode-Switch OSD HUD Overlay**: Instant visual on-screen display confirming layout toggling (<kbd>F12</kbd>).
  - Interactive Layout Viewer (Normal, Shift, and AltGr views with key-press illumination).
  - AutoCorrect Manager with live real-time search, insert/update, and deletion.
- **🔄 Bijoy (ANSI) ⇄ Unicode Converter**: Built-in lossless bidirectional converter supporting 50+ rare Sanskrit conjuncts, pre-Kar multi-glyph cluster reordering, and split-vowel synthesis.
- **😀 Semantic Emojis & Bilingual Shortcodes**: 3,600+ emojis with English and Bengali tags (`:bhalobasha:`, `:cha:`, `:fire:`/`:আগুন:`).
- **🧮 Inline Math Calculator & Currency Converter**: Live formula calculation (`=25*4+10=` ➔ `১১০`) and currency conversion (`#usd50` ➔ `৳৫,৮৫০`).
- **📝 Text Expander & Macros**: Dynamic date/time macros (`#date`/`#tarikh`, `#time`/`#shomoy`, `#bongabdo`), cultural phrases (`!shubhechha`), and custom user shortcuts.
- **⚙️ Systemd User Service Integration**: Ready-to-use background service units (`ibus-lekhani.service`, `lekhani-gui.service`).
- **☁️ Cross-Device Backup & Sync CLI**: One-line command to export/import configurations, dictionaries, and custom layouts (`lekhani sync`).
- **📊 Personal Typing Dashboard**: Live telemetry measuring words typed, keystrokes saved, and typing efficiency gains (`lekhani stats`).

---

## 📦 Installation

### Quick Installer Script (All Linux Distributions)
```bash
./install.sh           # Interactive installer menu
./install.sh --fcitx5  # Install for KDE Plasma 6 / Wayland
./install.sh --ibus    # Install for GNOME / Ubuntu
./install.sh --all     # Install both Fcitx5 and IBus engines
```

### Uninstallation
```bash
./uninstall.sh         # Or: ./install.sh --uninstall
```

### Arch Linux / Manjaro
```bash
cd packaging/arch
makepkg -si
```

### Fedora
```bash
sudo dnf builddep packaging/fedora/lekhani.spec
rpmbuild -ba packaging/fedora/lekhani.spec
```

---

## 🛠️ Building from Source

Ensure you have a modern Rust toolchain installed:
```bash
# Clone the repository
git clone https://github.com/OpenBangla/lekhani.git
cd lekhani

# Run test suite
cargo test --workspace

# Build optimized release binaries
cargo build --workspace --release
```

Binaries will be available in `target/release/`:
- `lekhani-gui` (Slint desktop TopBar & Tools)
- `ibus-lekhani` (IBus DBus engine daemon)
- `fcitx5-lekhani` (Fcitx5 / KDE Plasma engine daemon)
- `lekhani` (CLI tool & layout converter)

---

## 🏗️ Workspace Structure

- **`crates/lekhani-core`**: Core phonetic & fixed layout typing engines, Trie dictionary, Suffix rules, Autocorrect, Bijoy converter, Snippets.
- **`crates/lekhani-settings`**: XDG configuration (`~/.config/lekhani/config.toml`), dynamic layout discovery, and auto-migration.
- **`crates/lekhani-ibus`**: Pure Rust IBus DBus service daemon.
- **`crates/lekhani-fcitx5`**: Pure Rust Fcitx5 engine daemon for KDE Plasma 6 & Wayland.
- **`crates/lekhani-gui`**: Native Slint UI (TopBar, Layout Viewer, AutoCorrect search/delete, Settings).
- **`crates/lekhani-cli`**: Command-line tool and Avro 5 layout converter.

---

## 📜 License

Licensed under the [GNU General Public License v3.0 or later](LICENSE).
