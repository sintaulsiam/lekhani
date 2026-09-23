# AGENTS.md — Engineering Rules for Lekhani

Instructions and constraints for AI agents developing the Lekhani Bengali IME.

## 1. Non-Negotiable Directives
- **Sub-Microsecond Hot Path**: Keystroke processing and scoring must execute in **< 100 µs**. Never block the GUI or input event loop.
- **Strict Memory Budget**: Keep `fcitx5` daemon RSS **< 60 MB**. Spawning new windows must produce **0 MB** model growth.
- **Stability**: Zero unhandled panics (`unwrap()` / `expect()` on user input is forbidden). Gracefully fallback on missing/corrupted models.
- **Privacy**: 100% on-device. Zero network telemetry.
- **Fidelity**: Strict Avro phonetic muscle memory compatibility, accurate conjuncts (juktoborno), and contextual homophone ranking.

## 2. Memory & Architecture Rules
- **Shared Singletons (`Arc` / `OnceLock`)**: Desktop IMEs create a new session per window. Dictionaries, layout maps, and language models MUST be wrapped in `Arc` — window creation must be an $O(1)$ 8-byte pointer bump.
- **Zero Allocations on Keystroke**: No heap allocations during key press or candidate scoring (`hashbrown::Equivalent` with `&str` against interned `Arc<str>`).
- **Cap Collections**: Next-word continuations map capped to `MAX_CONTINUATIONS = 6`. Call `.shrink_to_fit()` on collections after loading.
- **Compact Shipped Model**: Keep `bengali_lm.bin` **< 6 MB**. Prune low-frequency long-tail N-grams.

## 3. Product & Code Hygiene
- **User vs Dev Separation**: Keep user CLI and GUI clean and jargon-free. Developer tools belong strictly under `lekhani dev <train|eval|benchmark>`.
- **Code Shield**: Do not mangle code tokens (`camelCase`, URLs, markdown backticks) when active.
- **Atomic Git Commits**:
  - Make small, focused, atomic git commits that represent a single logical change.
  - Write clear, imperative commit messages (e.g. `perf: intern language model strings to reduce memory`).
  - Never bundle unrelated refactors, features, or formatting changes together.
  - **Never commit raw training corpus files** (`*.txt`, `*.corpus`, `*.bz2`) to Git.

## 4. Mandatory Verification Checklist
Before concluding tasks:
1. `cargo run --release -p lekhani-cli -- dev eval` $\rightarrow$ Latency < 200 ns/eval, 5/5 homophones pass.
2. `cargo test --workspace --release` $\rightarrow$ 100% tests pass across all crates.
3. `ps aux | grep fcitx5` $\rightarrow$ Verify RSS remains flat and lightweight.
