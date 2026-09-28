#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════════════
# Lekhani Dictionary & Language Model Sync Script
# ══════════════════════════════════════════════════════════════════════════════
# Synchronizes trained binary models and dictionary assets between:
#   • Linux Desktop: /mnt/data/lekhani/data/dictionaries
#   • Android App:   /mnt/data/lekhani-android/data/dictionaries
#
# Usage:
#   ./scripts/sync_models.sh          # Sync from Linux -> Android (default)
#   ./scripts/sync_models.sh --check  # Check if models are in sync (no copy)
#   ./scripts/sync_models.sh --push   # Push from Android -> Linux
# ══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ANDROID_DIR="$(cd "$SCRIPT_DIR/.." && pwd)/data/dictionaries"
LINUX_DIR="/mnt/data/lekhani/data/dictionaries"

ACTION="${1:---pull}"

SYNC_FILES=(
    "bengali_lm.bin"
    "dictionary.bin"
    "dictionary.json"
    "autocorrect.json"
    "regex.json"
    "suffix.json"
)

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║          🔄 Lekhani Model & Dictionary Synchronizer             ║"
echo "╚══════════════════════════════════════════════════════════════════╝"

if [ ! -d "$LINUX_DIR" ]; then
    echo "❌ Linux dictionaries directory not found: $LINUX_DIR"
    exit 1
fi

if [ ! -d "$ANDROID_DIR" ]; then
    mkdir -p "$ANDROID_DIR"
fi

check_sync() {
    local all_matched=true
    printf "\n%-22s | %-12s | %-12s | %-8s\n" "File" "Linux MD5" "Android MD5" "Status"
    printf -- "-----------------------+--------------+--------------+----------\n"
    
    for file in "${SYNC_FILES[@]}"; do
        local linux_file="$LINUX_DIR/$file"
        local android_file="$ANDROID_DIR/$file"
        
        local l_md5="MISSING"
        local a_md5="MISSING"
        
        if [ -f "$linux_file" ]; then
            l_md5=$(md5sum "$linux_file" | awk '{print substr($1,1,10)}')
        fi
        if [ -f "$android_file" ]; then
            a_md5=$(md5sum "$android_file" | awk '{print substr($1,1,10)}')
        fi
        
        local status="MISMATCH"
        if [ "$l_md5" = "$a_md5" ] && [ "$l_md5" != "MISSING" ]; then
            status="MATCH ✅"
        else
            all_matched=false
        fi
        
        printf "%-22s | %-12s | %-12s | %-8s\n" "$file" "$l_md5" "$a_md5" "$status"
    done
    echo ""
    if [ "$all_matched" = true ]; then
        echo "✅ All models and dictionaries are in sync!"
    else
        echo "⚠️ Some files differ or are missing."
    fi
}

case "$ACTION" in
    --check|-c)
        check_sync
        ;;
    --pull|-p)
        echo "📥 Pulling models from Linux -> Android ($LINUX_DIR -> $ANDROID_DIR)..."
        for file in "${SYNC_FILES[@]}"; do
            if [ -f "$LINUX_DIR/$file" ]; then
                cp -v "$LINUX_DIR/$file" "$ANDROID_DIR/$file"
            else
                echo "⚠️ Skip $file (not present in Linux dir)"
            fi
        done
        echo ""
        check_sync
        ;;
    --push)
        echo "📤 Pushing models from Android -> Linux ($ANDROID_DIR -> $LINUX_DIR)..."
        for file in "${SYNC_FILES[@]}"; do
            if [ -f "$ANDROID_DIR/$file" ]; then
                cp -v "$ANDROID_DIR/$file" "$LINUX_DIR/$file"
            else
                echo "⚠️ Skip $file (not present in Android dir)"
            fi
        done
        echo ""
        check_sync
        ;;
    --help|-h)
        echo "Usage: $0 [--pull | --push | --check]"
        echo "  --pull   (default) Copy Linux models to Android"
        echo "  --push   Copy Android models to Linux"
        echo "  --check  Compare hashes without copying"
        exit 0
        ;;
    *)
        echo "Unknown option: $ACTION"
        exit 1
        ;;
esac
