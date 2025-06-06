# CodeOrbit Cross-Platform Packaging Guide

This guide describes how to build standalone CodeOrbit binaries for multiple operating systems.

## Prerequisites
- **Node.js 20** and `npm`
- Rust toolchain if you plan to build the Zed extension
- `pkg` (installed automatically via `npm install`)

## Building Executables

1. Install dependencies once:
   ```bash
   npm install
   ```

2. Run the packaging script:
   ```bash
   ./script/package-codeorbit.sh
   ```

This compiles the TypeScript sources and creates platform binaries in the `release/` directory:

- `CodeOrbit-win.exe` – Windows 64-bit
- `CodeOrbit-mac` – macOS binary (can be wrapped into a `.dmg` using tools like `create-dmg`)
- `CodeOrbit-linux` – Linux executable (use `appimagetool` to convert to `.AppImage` or package as `.deb`)
- `CodeOrbit-harmony` – Linux ARM64 build suitable for HarmonyOS via OpenHarmony

### Creating Installers

For macOS and Linux you can optionally create installer formats:

- **macOS `.dmg`**
  ```bash
  create-dmg release/CodeOrbit-mac release/
  ```

- **Linux `.AppImage`**
  ```bash
  appimagetool release/CodeOrbit-linux release/CodeOrbit.AppImage
  ```

- **Linux `.deb`** – package using `dpkg-deb` or tools like `electron-builder`.

These steps require the respective tools to be installed on your system.

## Publishing Releases

After packaging, upload the files from `release/` to your preferred distribution channel (GitHub Releases, website, etc.).
