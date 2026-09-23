#!/usr/bin/env bash
# Lekhani Large Bengali AI Language Model Automated Training Script
# Usage:
#   ./scripts/train_large_model.sh --quick        # Fast build: 150 articles (~5 MB text, ~20 sec)
#   ./scripts/train_large_model.sh --large        # High capacity: 600 articles (~20 MB text, ~1 min)
#   ./scripts/train_large_model.sh --full-dump    # Complete Bengali Wikipedia dump (~1.5 GB text)
#   ./scripts/train_large_model.sh --dir <PATH>   # Ingest custom folder of text files
#   ./scripts/train_large_model.sh --install      # Train and install directly to system

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$ROOT_DIR"

MODE="${1:---large}"

case "$MODE" in
    --quick)
        echo "=== Running Quick Bengali AI Training (150 Articles) ==="
        python3 "$SCRIPT_DIR/train_large_model.py" --articles 150 "${@:2}"
        ;;
    --large)
        echo "=== Running High-Capacity Bengali AI Training (600 Articles) ==="
        python3 "$SCRIPT_DIR/train_large_model.py" --articles 600 --threads 12 "${@:2}"
        ;;
    --full-dump)
        echo "=== Running Full Bengali Wikipedia Dump Training (~1.5 GB Text) ==="
        python3 "$SCRIPT_DIR/train_large_model.py" --full-dump "${@:2}"
        ;;
    --dir|-d)
        echo "=== Training on Local Directory: $2 ==="
        python3 "$SCRIPT_DIR/train_large_model.py" --dir "$2" --skip-fetch "${@:3}"
        ;;
    --skip-fetch)
        echo "=== Retraining on Existing Corpus ==="
        python3 "$SCRIPT_DIR/train_large_model.py" --skip-fetch "${@:2}"
        ;;
    --install)
        echo "=== Installing Trained Bengali Language Model to System ==="
        MODEL_PATH="$ROOT_DIR/data/dictionaries/bengali_lm.bin"
        if [ ! -f "$MODEL_PATH" ]; then
            echo "[!] Error: Model $MODEL_PATH not found. Train first." >&2
            exit 1
        fi
        sudo install -d /usr/share/lekhani/data
        sudo install -Dm644 "$MODEL_PATH" /usr/share/lekhani/data/bengali_lm.bin
        echo "[✓] Language model installed to /usr/share/lekhani/data/bengali_lm.bin successfully!"
        ;;
    --help|-h)
        echo "Lekhani Automated Large Bengali AI Training Script"
        echo ""
        echo "Options:"
        echo "  --quick            Quick training on 150 Wikipedia articles (~20s)"
        echo "  --large            High-capacity training on 600 Wikipedia articles (~1m)"
        echo "  --full-dump        Download and train on full 540MB Wikipedia dump (~1.5GB text)"
        echo "  --dir <PATH>       Ingest and train on local folder of .txt files"
        echo "  --skip-fetch       Re-train on existing data/bengali_training_corpus.txt"
        echo "  --install          Install trained model to /usr/share/lekhani/data/"
        echo "  --help             Display this help message"
        exit 0
        ;;
    *)
        python3 "$SCRIPT_DIR/train_large_model.py" "$@"
        ;;
esac
