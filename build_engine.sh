#!/bin/bash
set -e

# Directories
PROJECT_ROOT=$(pwd)
ENGINE_DIR="$PROJECT_ROOT/hippocrates_engine"
EDITOR_DIR="$PROJECT_ROOT/hippocrates_editor/Sources/HippocratesEditor/HippocratesEngine"

# Build Rust Engine
echo "Building Hippocrates Engine (Release)..."
cd "$ENGINE_DIR"
cargo build --release

# Create destination directory in Editor
mkdir -p "$EDITOR_DIR"

# Copy Artifacts
echo "Copying artifacts to Editor..."
cp "$ENGINE_DIR/target/release/libhippocrates_engine.a" "$EDITOR_DIR/"
cp "$ENGINE_DIR/include/hippocrates_engine.h" "$EDITOR_DIR/"

echo "Done!"
