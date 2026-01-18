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
EDITOR_LIBS_DIR="$PROJECT_ROOT/hippocrates_editor/lib"

echo "Building Rust engine..."
cd "$RUST_PROJECT_DIR"

# Build for the target architecture (x86_64 or arm64 depending on Xcode build, 
# but for now we let cargo decide based on the current machine or force universal if needed)
# Simply running cargo build --release handles the current arch.
cargo build --release

# Ensure destination exists
mkdir -p "$EDITOR_LIBS_DIR"

# Copy library and header
cp "$RUST_PROJECT_DIR/target/release/libhippocrates_engine.a" "$EDITOR_LIBS_DIR/"
cp "$RUST_PROJECT_DIR/include/hippocrates_engine.h" "$EDITOR_LIBS_DIR/"

echo "Rust engine built and copied to $EDITOR_LIBS_DIR"
