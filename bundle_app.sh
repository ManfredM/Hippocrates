#!/bin/bash
set -e

APP_NAME="HippocratesEditor"
APP_BUNDLE="${APP_NAME}.app"
OUTPUT_DIR="."
echo "Building HippocratesEditor (Release)..."
cd hippocrates_editor
# Ensure project is generated
xcodegen
# Build
xcodebuild -scheme HippocratesEditor -configuration Release -derivedDataPath .build -arch arm64 build
cd ..

echo "Copying App Bundle..."
# xcodebuild creates the full .app bundle
rm -rf "$OUTPUT_DIR/$APP_BUNDLE"
cp -r "hippocrates_editor/.build/Build/Products/Release/$APP_BUNDLE" "$OUTPUT_DIR/"

echo "Done! App created at $OUTPUT_DIR/$APP_BUNDLE"
echo "You can run it with: open $OUTPUT_DIR/$APP_BUNDLE"
