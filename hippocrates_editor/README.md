# Hippocrates Editor

A native macOS visual editor for Hippocrates plans, built with SwiftUI.

## Features

- **Plan Visualizer**: Syntax highlighting for Hippocrates files.
- **Execution Timeline**: Visualization of plan execution events over time.

## How to Run

1. Open this folder in Xcode (`File > Open...` and select `hippocrates_editor`).
2. Select the `HippocratesEditor` scheme.
3. Press Run (Cmd+R).

## Architecture

- **CodeVisualizerView**: Handles syntax highlighting using `AttributedString` and Regex.
- **ExecutionTimelineView**: Uses Swift Charts to display execution events.
- **ContentView**: Main split view layout.
