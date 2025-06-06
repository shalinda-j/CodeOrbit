#!/usr/bin/env bash

set -euo pipefail

# Ensure TypeScript is compiled
npm run build

OUTPUT_DIR="release"
mkdir -p "$OUTPUT_DIR"

# Build platform binaries using pkg
npx pkg dist/main.js --targets node20-win-x64    --output "$OUTPUT_DIR/CodeOrbit-win.exe"
npx pkg dist/main.js --targets node20-macos-x64  --output "$OUTPUT_DIR/CodeOrbit-mac"
npx pkg dist/main.js --targets node20-linux-x64  --output "$OUTPUT_DIR/CodeOrbit-linux"
# HarmonyOS (OpenHarmony) uses arm64 Linux target
npx pkg dist/main.js --targets node20-linux-arm64 --output "$OUTPUT_DIR/CodeOrbit-harmony"

echo "Binaries are located in $OUTPUT_DIR"
