#!/bin/bash

# Script to clean all build artifacts and directories
# This script is called from build.rs when architecture mismatch is detected

set -e

NIM_CODEX_DIR="${1:-vendor/nim-codex}"

echo "🧹 Starting build artifacts cleanup..."
echo "📁 Target directory: $NIM_CODEX_DIR"

# Clean Nim cache to prevent architecture conflicts
if [[ -n "$HOME" ]]; then
    NIM_CACHE_PATH="$HOME/.cache/nim/libcodex_d"
    if [[ -d "$NIM_CACHE_PATH" ]]; then
        echo "🗑️  Removing Nim cache: $NIM_CACHE_PATH"
        rm -rf "$NIM_CACHE_PATH"
    fi
fi

# Clean build directories
BUILD_DIRECTORIES=(
    "vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc/build"
    "vendor/nim-nat-traversal/vendor/libnatpmp-upstream/build"
    "vendor/nim-circom-compat/vendor/circom-compat-ffi/target"
    "vendor/nim-leveldbstatic/build"
    "build"
    "nimcache/release"
    "nimcache/debug"
)

for dir in "${BUILD_DIRECTORIES[@]}"; do
    FULL_PATH="$NIM_CODEX_DIR/$dir"
    if [[ -d "$FULL_PATH" ]]; then
        echo "🗑️  Removing build directory: $dir"
        rm -rf "$FULL_PATH"
    fi
done

# Clean any leftover .o files in specific directories
OBJECT_FILE_DIRS=(
    "vendor/nim-nat-traversal/vendor/libnatpmp-upstream"
    "vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc"
)

for dir in "${OBJECT_FILE_DIRS[@]}"; do
    DIR_PATH="$NIM_CODEX_DIR/$dir"
    if [[ -d "$DIR_PATH" ]]; then
        echo "🧹 Cleaning object files in: $dir"
        find "$DIR_PATH" -name "*.o" -type f -delete 2>/dev/null || true
    fi
done

# Restore .gitkeep file in nim-leveldbstatic build directory
cd "$NIM_CODEX_DIR/vendor/nim-leveldbstatic" && git restore build/.gitkeep 2>/dev/null || true

echo "✅ Build artifacts cleanup completed"