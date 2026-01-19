#!/bin/bash
set -e

# Ensure Cargo is in PATH (Xcode shell environment is restricted)
export PATH="$HOME/.cargo/bin:/usr/local/bin:/opt/homebrew/bin:$PATH"

if ! command -v cargo &> /dev/null; then
    echo "Error: cargo could not be found. Please ensure Rust is installed."
    echo "PATH is: $PATH"
    exit 1
fi

# Directory where this script resides
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."

# Rust Project Path
RUST_PROJECT_DIR="$PROJECT_ROOT/hippocrates_engine"

# Editor Libs Path (where Xcode expects to find the library)
# Editor Libs Path (where Xcode expects to find the library)
EDITOR_LIBS_DIR="$PROJECT_ROOT/hippocrates_editor/lib"
# Header Include Path (where Xcode expects headers)
EDITOR_INCLUDE_DIR="$PROJECT_ROOT/hippocrates_editor/Sources/CHippocratesEngine/include"

echo "Building Rust engine..."
cd "$RUST_PROJECT_DIR"

# Determine Build Profile based on Xcode CONFIGURATION
if [ "$CONFIGURATION" = "Debug" ]; then
    echo "Building for Debug..."
    cargo build
    TARGET_DIR="debug"
else
    echo "Building for Release..."
    cargo build --release
    TARGET_DIR="release"
fi

# Ensure destinations exist
mkdir -p "$EDITOR_LIBS_DIR"
mkdir -p "$EDITOR_INCLUDE_DIR"

# Copy library
cp "$RUST_PROJECT_DIR/target/$TARGET_DIR/libhippocrates_engine.a" "$EDITOR_LIBS_DIR/"

# Copy header to both locations to be safe (lib for search paths, include for target reference)
cp "$RUST_PROJECT_DIR/include/hippocrates_engine.h" "$EDITOR_LIBS_DIR/"
cp "$RUST_PROJECT_DIR/include/hippocrates_engine.h" "$EDITOR_INCLUDE_DIR/"

echo "Rust engine built ($TARGET_DIR) and copied to $EDITOR_LIBS_DIR and $EDITOR_INCLUDE_DIR"
