#!/bin/bash
set -e

APP_NAME="HippocratesEditor"
APP_BUNDLE="${APP_NAME}.app"
BUILD_DIR="hippocrates_editor/.build/release"
OUTPUT_DIR="."

echo "Building HippocratesEditor (Release)..."
cd hippocrates_editor
swift build -c release
cd ..

echo "Creating App Bundle Structure..."
rm -rf "$OUTPUT_DIR/$APP_BUNDLE"
mkdir -p "$OUTPUT_DIR/$APP_BUNDLE/Contents/MacOS"
mkdir -p "$OUTPUT_DIR/$APP_BUNDLE/Contents/Resources"

echo "Copying Executable..."
cp "$BUILD_DIR/$APP_NAME" "$OUTPUT_DIR/$APP_BUNDLE/Contents/MacOS/"

echo "Creating Info.plist..."
cat > "$OUTPUT_DIR/$APP_BUNDLE/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>com.manfred.HippocratesEditor</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>14.0</string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "Done! App created at $OUTPUT_DIR/$APP_BUNDLE"
echo "You can run it with: open $OUTPUT_DIR/$APP_BUNDLE"
