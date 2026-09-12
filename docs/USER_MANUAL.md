# 📖 Lekhani User Manual (লেখনী ব্যবহার নির্দেশিকা)

> **Lekhani (লেখনী)** is a modern, high-performance, pure-Rust Bengali Input Method Engine (IME) for Linux desktops. It brings 100% faithful Avro Phonetic typing, intelligent AI-powered word prediction, fixed layouts (Unijoy/Bijoy), and rich productivity power tools to KDE Plasma 6, GNOME, Wayland, and X11.

---

## 📑 Table of Contents

1. [Quick Start & Setup](#1-quick-start--setup)
   - [Switching Input Methods & Hotkeys](#switching-input-methods--hotkeys)
   - [Desktop Indicator & Layout Selection](#desktop-indicator--layout-selection)
2. [Typing with Avro Phonetic](#2-typing-with-avro-phonetic)
   - [Vowels & Vowel Marks (স্বরবর্ণ ও কার)](#vowels--vowel-marks-স্বরবর্ণ-ও-কার)
   - [Consonants (ব্যঞ্জনবর্ণ)](#consonants-ব্যঞ্জনবর্ণ)
   - [Special Characters, Punctuation & Modifiers](#special-characters-punctuation--modifiers)
3. [Mastering Complex Ligatures (যুক্তবর্ণ নির্দেশিকা)](#3-mastering-complex-ligatures-যুক্তবর্ণ-নির্দেশিকা)
   - [Reph (রেফ: র্)](#reph-রেফ-র্)
   - [Ja-phala (য-ফলা: ্য)](#ja-phala-য-ফলা-্য)
   - [Khanda-Ta (খণ্ড-ত: ৎ)](#khanda-ta-খণ্ড-ত-ৎ)
   - [Chandrabindu (চন্দ্রবিন্দু: ঁ)](#chandrabindu-চন্দ্রবিন্দু-ঁ)
   - [Ri-kar (ঋ-কার: ৃ)](#ri-kar-ঋ-কার-ৃ)
   - [Phala Conjuncts (ব-ফলা, ম-ফলা, র-ফলা)](#phala-conjuncts-ব-ফলা-ম-ফলা-র-ফলা)
   - [Essential Juktoborno Cheatsheet](#essential-juktoborno-cheatsheet)
4. [Dual-Engine Experience: Strict vs. Smart Casual Typing](#4-dual-engine-experience-strict-vs-smart-casual-typing)
5. [Candidate Window Navigation & Hotkeys](#5-candidate-window-navigation--hotkeys)
6. [Built-in Productivity Power Tools](#6-built-in-productivity-power-tools)
   - [Live Inline Math Calculator](#live-inline-math-calculator)
   - [Currency Converter](#currency-converter)
   - [Dynamic Macros & Date/Time](#dynamic-macros--datetime)
   - [Bengali & Shortcode Emojis](#bengali--shortcode-emojis)
7. [Fixed Layouts: Unijoy & Bijoy Mode](#7-fixed-layouts-unijoy--bijoy-mode)
8. [Troubleshooting & FAQs](#8-troubleshooting--faqs)

---

## 1. Quick Start & Setup

### Switching Input Methods & Hotkeys

- **Activate Lekhani**: Press your desktop input switcher hotkey (usually <kbd>Ctrl</kbd> + <kbd>Space</kbd> or <kbd>Super</kbd> + <kbd>Space</kbd>).
- **Toggle Bengali / English Mode**: Press <kbd>F12</kbd> anytime to instantly toggle between Bengali transliteration and direct English pass-through typing.
- **Commit Selected Word**: Press <kbd>Space</kbd> or <kbd>Enter</kbd>.
- **Select Specific Candidate**: Press number keys <kbd>1</kbd>–<kbd>9</kbd> or use <kbd>↓</kbd>/<kbd>Tab</kbd> then <kbd>Space</kbd>.

---

## 2. Typing with Avro Phonetic

Avro Phonetic lets you type Bengali naturally using standard English QWERTY keys.

### Vowels & Vowel Marks (স্বরবর্ণ ও কার)

| Bengali | Independent Key | Kar Form | Kar Keystroke | Example Word |
| :---: | :---: | :---: | :---: | :--- |
| **অ** | `o` / `a` | ক | `k` / `ko` | `onil` ➔ অনিল |
| **আ** | `a` / `aa` / `A` | কা | `ka` / `kaa` / `kA` | `aam` ➔ আম, `kaj` ➔ কাজ |
| **ই** | `i` | কি | `ki` | `iti` ➔ ইতি, `din` ➔ দিন |
| **ঈ** | `I` / `ee` | কী | `kI` / `kee` | `Igol` ➔ ঈগল, `nodI` ➔ নদী |
| **উ** | `u` | কু | `ku` | `uthan` ➔ উঠান, `ful` ➔ ফুল |
| **ঊ** | `U` / `oo` | কূ | `kU` / `koo` | `Usha` ➔ ঊষা, `dUr` ➔ দূর |
| **ঋ** | `rri` | কৃ | `krri` | `rrin` ➔ ঋণ, `krishi` ➔ কৃষি |
| **এ** | `e` | কে | `ke` | `ek` ➔ এক, `desh` ➔ দেশ |
| **ঐ** | `OI` / `oi` | কৈ | `kOI` / `koi` | `Oikyo` ➔ ঐক্য, `toiri` ➔ তৈরি |
| **ও** | `O` / `o` | কো | `kO` / `ko` | `olpo` ➔ অল্প, `golpo` ➔ গল্প |
| **ঔ** | `OU` / `ou` | কৌ | `kOU` / `kou` | `OUshodh` ➔ ঔষধ, `nouka` ➔ নৌকা |

---

### Consonants (ব্যঞ্জনবর্ণ)

| Varga | Letter 1 | Letter 2 | Letter 3 | Letter 4 | Nasal |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **ক-বর্গ** | **ক** (`k`) | **খ** (`kh`) | **গ** (`g`) | **ঘ** (`gh`) | **ঙ** (`Ng`) |
| **চ-বর্গ** | **চ** (`c`) | **ছ** (`ch`) | **জ** (`j`) | **ঝ** (`jh`) | **ঞ** (`NG`) |
| **ট-বর্গ (Murdhanya)** | **ট** (`T`) | **ঠ** (`Th`) | **ড** (`D`) | **ঢ** (`Dh`) | **ণ** (`N`) |
| **ত-বর্গ (Dantya)** | **ত** (`t`) | **থ** (`th`) | **দ** (`d`) | **ধ** (`dh`) | **ন** (`n`) |
| **প-বর্গ** | **প** (`p`) | **ফ** (`f`/`ph`) | **ব** (`b`) | **ভ** (`bh`/`v`) | **ম** (`m`) |

#### Sibilants, Liquids & Specials
- **য** (Antastha Ja): `z` or `y` (e.g. `zodi` ➔ যদি, `zog` ➔ যোগ)
- **র** (Ro): `r` (e.g. `rong` ➔ রং)
- **ল** (Lo): `l` (e.g. `lal` ➔ লাল)
- **শ** (Talobyo Sha): `sh` or `S` (e.g. `shanti` ➔ শান্তি)
- **ষ** (Murdhonyo Sha): `Sh` (Shift+s+h) (e.g. `bhaSha` ➔ ভাষা, `koShTo` ➔ কষ্ট)
- **স** (Donto Sa): `s` (e.g. `sundor` ➔ সুন্দর)
- **হ** (Ho): `h` (e.g. `hasi` ➔ হাসি)
- **ড়** (Dabindo Ra): `R` (Shift+r) (e.g. `boRo` ➔ বড়, `gaRi` ➔ গাড়ি)
- **ঢ়** (Dhabindo Rha): `Rh` (Shift+r+h) (e.g. `aaRhaai` ➔ আড়াই)
- **য়** (Antastha A): `y` or `Y` (e.g. `bhay` ➔ ভয়, `doya` ➔ দয়া)

---

### Special Characters, Punctuation & Modifiers

| Character | Name | Keystroke | Example Input ➔ Output |
| :---: | :--- | :--- | :--- |
| **ৎ** | Khanda Ta | `t``` (`t` + backtick) | `ut```shob` ➔ উৎসব, `hot```hat` ➔ হঠাৎ |
| **ং** | Anusvara | `ng` | `baangla` ➔ বাংলা, `rong` ➔ রং |
| **ঃ** | Visarga | `:` (Colon) | `du:kho` ➔ দুঃখ, `prato:` ➔ প্রাতঃ |
| **ঁ** | Chandrabindu | `^` (Caret / Shift+6) | `ca^d` ➔ চাঁদ, `ha^s` ➔ হাঁস, `ba^sh` ➔ বাঁশ |
| **।** | Dari (Bengali Period) | `.` or `\|` | `ami banglay gai.` ➔ আমি বাংলায় গাই। |
| **্** | Hasanta / Virama | `,,` (Double comma) | Force explicit consonant stop |

---

## 3. Mastering Complex Ligatures (যুক্তবর্ণ নির্দেশিকা)

### Reph (রেফ: র্)
Type **`rr` before the consonant** where the reph sits:
- `borrtoman` ➔ **বর্তমান**
- `korrmo` ➔ **কর্ম**
- `durrghoTona` ➔ **দুর্ঘটনা**
- `shoorrzo` ➔ **সূর্য**

### Ja-phala (য-ফলা: ্য)
Type **Capital `Z` (Shift+z)** after the consonant:
- `bidZaloy` ➔ **বিদ্যালয়**
- `totthZ` / `tothZ` ➔ **তথ্য**
- `bZakhyan` ➔ **ব্যাখ্যান**

### Khanda-Ta (খণ্ড-ত: ৎ)
Type **`t` followed by a backtick (`` ` ``)**:
- `ut```shob` ➔ **উৎসব**
- `bidZut``` ➔ **বিদ্যুৎ**
- `attotZag` ➔ **আত্মত্যাগ**

### Chandrabindu (চন্দ্রবিন্দু: ঁ)
Type **`^` immediately after the vowel or kar**:
- `ca^d` ➔ **চাঁদ**
- `ha^s` ➔ **হাঁস**
- `pa^c` ➔ **পাঁচ**
- `pZa^c` ➔ **প্যাঁচ**

### Ri-kar (ঋ-কার: ৃ)
Type **`rri` or `rre`**:
- `krritrim` ➔ **কৃত্রিম**
- `matrribhUmi` ➔ **মাতৃভূমি**
- `brriddhi` ➔ **বৃদ্ধি**

### Phala Conjuncts (ব-ফলা, ম-ফলা, র-ফলা)
- **ব-ফলা (`w`)**: `swadhin` ➔ **স্বাধীন**, `dwidha` ➔ **দ্বিধা**, `bishwash` ➔ **বিশ্বাস**
- **ম-ফলা (`m`)**: `smriti` ➔ **স্মৃতি**, `padmo` ➔ **পদ্ম**, `brohmo` ➔ **ব্রহ্ম**
- **র-ফলা (`r`)**: `gram` ➔ **গ্রাম**, `probhat` ➔ **প্রভাত**, `shrom` ➔ **শ্রম**

---

### Essential Juktoborno Cheatsheet

| Ligature | Letters | Standard Keystroke | Example Word |
| :---: | :---: | :--- | :--- |
| **জ্ঞ** | জ + ঞ | `jng` or `gZ` | `gZan` / `jngan` ➔ **জ্ঞান** |
| **ক্ষ** | ক + ষ | `kkh` or `kSh` | `shikkha` ➔ **শিক্ষা**, `kkhoma` ➔ **ক্ষমা** |
| **ষ্ণ** | ষ + ণ | `ShN` | `krriShNo` ➔ **কৃষ্ণ**, `uShNo` ➔ **উষ্ণ** |
| **ঙ্ক** | ঙ + ক | `Nk` | `oNko` ➔ **অঙ্ক**, `aatoNk` ➔ **আতঙ্ক** |
| **ঙ্গ** | ঙ + গ | `Ng` | `boNgo` ➔ **বঙ্গ**, `shoNge` ➔ **সঙ্গে** |
| **ঞ্চ** | ঞ + চ | `Nc` | `oNcol` ➔ **অঞ্চল**, `kaNcon` ➔ **কাঞ্চন** |
| **ঞ্জ** | ঞ + জ | `Nj` | `roNjon` ➔ **রঞ্জন**, `guNjon` ➔ **গুঞ্জন** |
| **হ্ণ** | হ + ণ | `hN` | `oporaahN` ➔ **অপরাহ্ণ** |
| **হ্ন** | হ + ন | `hn` | `cihno` ➔ **চিহ্ন**, `moddhaahn` ➔ **মধ্যাহ্ন** |
| **হ্ম** | হ + ম | `hm` | `brohmonbaria` ➔ **ব্রাহ্মণবাড়িয়া** |
| **হ্ব** | হ + ব | `hw` / `hb` | `ahban` ➔ **আহ্বান** |
| **স্ট / ষ্ট** | স+ট / ষ+ট | `sT` / `ShT` | `posTor` ➔ **পোস্টার**, `driShTi` ➔ **দৃষ্টি** |

---

## 4. Dual-Engine Experience: Strict vs. Smart Casual Typing

Lekhani provides a zero-compromise dual engine:

1. **Exact Precision Typing**: If you type exact Avro keys (`borrtoman`, `ut```shob`, `ca^d`), the transliteration engine instantly outputs exact Bangla Academy spellings.
2. **Forgiving Casual Typing**: If you type quickly without capital letters or complex modifiers, Lekhani's AI and sound-law suggestion engine automatically ranks standard spellings at top priority:

| Desired Word | Casual Input | Standard Avro Key | Lekhani Candidate #1 |
| :--- | :--- | :--- | :---: |
| **দূরে** | `dure` / `dUre` | `dUr-e` | **দূরে** |
| **বর্তমান** | `bortoman` | `borrtoman` | **বর্তমান** |
| **স্বাস্থ্য** | `shastho` / `sastho` | `swasthZo` | **স্বাস্থ্য** |
| **মৃত্যু** | `mrittyu` / `mrittu` | `mrritZu` | **মৃত্যু** |
| **ভালোবাসা** | `bhalobasha` / `valobasha` | `bhalobasha` | **ভালোবাসা** |
| **বিজ্ঞান** | `biggan` / `bignan` | `bijngan` | **বিজ্ঞান** |
| **উৎসব** | `utshob` | `ut```shob` | **উৎসব** |
| **চাঁদ** | `chad` | `ca^d` | **চাঁদ** |

---

## 5. Candidate Window Navigation & Hotkeys

```
┌────────────────────────────────────────────────────────┐
│  dure                                                  │
├────────────────────────────────────────────────────────┤
│  1. দূরে  │ 2. দুরে  │ 3. ডুরে  │ 4. দূড়ে  │ 5. দূর্       │
└────────────────────────────────────────────────────────┘
```

- <kbd>Space</kbd> : Accepts the highlighted suggestion (default: candidate #1).
- <kbd>1</kbd> .. <kbd>9</kbd> : Instantly picks that numbered candidate.
- <kbd>Tab</kbd> or <kbd>↓</kbd> : Moves selection forward.
- <kbd>Shift</kbd> + <kbd>Tab</kbd> or <kbd>↑</kbd> : Moves selection backward.
- <kbd>Backspace</kbd> : Edits the active pre-edit text.
- <kbd>Esc</kbd> : Dismisses the candidate bar and commits raw English.

---

## 6. Built-in Productivity Power Tools

### Live Inline Math Calculator
Type any calculation starting or ending with `=`, and Lekhani calculates it in Bengali numerals:
- `=15*24` ➔ **৩৬০**
- `500+250=` ➔ **৭৫০**
- `=1200/4` ➔ **৩০০**

### Currency Converter
Convert international currencies to Bangladeshi Taka (৳) on the fly:
- `#usd50` ➔ **৳৬,১০০**
- `#eur100` ➔ **৳১৩,২০০**
- `#gbp20` ➔ **৳৩,১০০**

### Dynamic Macros & Date/Time
Expand dynamic dates, times, and formal phrases instantly:
- `#tarikh` ➔ **১২ সেপ্টেম্বর ২০২৬** (Today's Bengali Date)
- `#shomoy` ➔ **০৯:৫৫ অপরাহ্ন** (Current Time)
- `!shubhechha` ➔ **আপনাকে আন্তরিক শুভেচ্ছা ও অভিনন্দন।**
- `!dhyanobad` ➔ **আপনাকে অসংখ্য ধন্যবাদ।**

### Bengali & Shortcode Emojis
Type emotional keywords or shortcodes surrounded by `:` to insert emojis and symbols:
- `:bhalobasha:` ➔ ❤️
- `:cha:` ➔ ☕
- `:daktar:` ➔ 🩺
- `:shurjo:` ➔ ☀️
- `*taka*` ➔ ৳
- `*dari*` ➔ ।

---

## 7. Fixed Layouts: Unijoy & Bijoy Mode

Prefer traditional key-by-key fixed keyboard layouts? Lekhani supports full Unicode fixed mappings:

- **Unijoy Layout**: Select `Unijoy` from the Lekhani GUI tray menu or settings.
- **Bijoy (Unicode) Layout**: Select `National` or `Bijoy` from layout options.
- **Bijoy ⇄ Unicode Document Converter**: Open the Lekhani GUI (`lekhani-gui`) or run `lekhani-cli convert` to convert legacy ANSI / SutonnyMJ documents into standard Unicode.

---

## 8. Troubleshooting & FAQs

#### Q1: How do I open Lekhani Settings & Dictionary Manager?
Run `lekhani-gui` in your application launcher or terminal. Here you can add custom autocorrect rules, manage user vocabulary, and switch themes.

#### Q2: Candidate popup doesn't appear in Wayland apps?
Lekhani connects natively via Fcitx5 and IBus protocols, which fully support Wayland protocols (`zwp_input_method_v2`). Ensure your session variables are configured:
```bash
export GTK_IM_MODULE=fcitx
export QT_IM_MODULE=fcitx
export XMODIFIERS=@im=fcitx
```

#### Q3: How do I restart the typing daemon after updates?
- For Fcitx5: `fcitx5 -r -d`
- For IBus: `ibus restart`

---

*Lekhani: Crafted with ❤️ in Pure Rust for Bengali Typists everywhere.*
