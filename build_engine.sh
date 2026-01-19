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

# Also copy to lib/ as Xcode project seems to reference it there
EDITOR_LIB_DIR="$PROJECT_ROOT/hippocrates_editor/lib"
mkdir -p "$EDITOR_LIB_DIR"
cp "$ENGINE_DIR/target/release/libhippocrates_engine.a" "$EDITOR_LIB_DIR/"
cp "$ENGINE_DIR/include/hippocrates_engine.h" "$EDITOR_LIB_DIR/"

# Also copy to CHippocratesEngine/include/ as Xcode project seems to reference it there
EDITOR_C_INCLUDE_DIR="$PROJECT_ROOT/hippocrates_editor/Sources/CHippocratesEngine/include"
mkdir -p "$EDITOR_C_INCLUDE_DIR"
cp "$ENGINE_DIR/include/hippocrates_engine.h" "$EDITOR_C_INCLUDE_DIR/"

echo "Done!"
