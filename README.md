# Lekhani (লেখনী)

<p align="center">
  <img src="data/icons/128.png" alt="Lekhani Logo" />
</p>

<p align="center">
  <b>Modern, Ultra-Fast Pure Rust Bengali Input Method & Desktop Suite for Linux</b>
</p>

<p align="center">
  <a href="#features">Features</a> •
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
- **⚡ Frequency-Weighted Trie Candidate Engine**: Sub-millisecond candidate queries with 200+ conversational unigram weights.
- **🧠 Context-Aware Homophone Disambiguation**: Evaluates preceding words ($w_{t-1}$) to resolve homophones (*বই পড়া* vs *শার্ট পরা*, *বাংলা ভাষা* vs *ভেসে ভাসা*, *জীবনের লক্ষ্য* vs *এক লক্ষ*).
- **🔮 Zero-Preedit Next-Word Prediction**: Instant probable word suggestions upon committing (`আমি` ➔ `ভালো`, `তোমাকে`, `যাব`, `চাই`).
- **📖 Autonomous Morphological Learner**: Automatically extracts base stems and indexes new vocabulary from user typing.
- **🌐 Bilingual Code-Mixing & Tech Loanwords**: Dual candidate generation for English words (`meeting` ➔ `মিটিং` & `meeting`, `laptop`, `doctor`).
- **⌨️ Popular Fixed Keyboard Layouts**: Full built-in support for **Probhat**, **National (Jatiya)**, **Munir Optima**, **Borno**, and **Avro Easy**.
- **🪄 Advanced Typing Automations**:
  - Automatic Vowel Forming (অ + া ➔ আ)
  - Automatic Chandra Position Fixing (কঁ + া ➔ কাঁ)
  - Traditional Kar Joining (ligature blocking via ZWNJ)
  - Old-style Reph (র্) insertion algorithm
  - Number pad Bengali digits auto-mapping
- **🌐 Dual Desktop Framework Support**:
  - **Fcitx5 Native Addon** (`fcitx5-lekhani.so`) for **KDE Plasma 6** and modern **Wayland** compositors (Hyprland, Sway).
  - **IBus Daemon** (`ibus-lekhani` via `zbus`) for GNOME and standard desktops.
- **🎨 Sleek Slint Native Desktop UI** (`lekhani-gui`):
  - Floating, frameless, draggable TopBar with always-on-top mode.
  - Interactive Layout Viewer (Normal & AltGr views with key-press lighting up).
  - AutoCorrect Manager with live real-time search, insert/update, and deletion.
- **🔄 Bijoy (ANSI) ⇄ Unicode Converter**: Built-in lossless bidirectional converter for legacy SutonnyMJ/Bijoy documents.
- **😀 Semantic Emojis & Bengali Shortcodes**: Instant expansion for `:bhalobasha:` (❤️), `:cha:` (☕), `:pani:` (💧), `:daktar:` (🩺), `*taka*` (৳), `*dari*` (।).
- **🧮 Inline Math Calculator & Currency Converter**: Live formula calculation (`=125*8` ➔ `১,০০০`) and currency conversion (`#usd50` ➔ `৳৬,১০০`).
- **📝 Text Expander & Macros**: Dynamic date/time macros (`#tarikh`, `#shomoy`), cultural phrases (`!shubhechha`), and custom user shortcuts (`;email`).
- **☁️ Cross-Device Backup & Sync CLI**: One-line command to export/import configurations, dictionaries, and custom layouts (`lekhani sync`).
- **📊 Personal Typing Dashboard**: Live telemetry measuring words typed, keystrokes saved, and typing efficiency gains (`lekhani stats`).

---

## 📦 Installation

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
