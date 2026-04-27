#!/bin/bash
set -e

# 1. Colors for status
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}==> Phase 1: Building Tauri Binary (Release)...${NC}"
cd AnaFis
bun run tauri build

echo -e "${GREEN}==> Phase 2: Building Flatpak...${NC}"
cd ../Installer/Linux/Flatpak/

# Remove previous build artifacts to ensure a clean slate
rm -rf build-dir repo

# Build the flatpak
flatpak-builder --user --install --force-clean build-dir com.CokieMiner.AnaFis.yml

echo -e "${GREEN}==> Success! AnaFis has been installed as a Flatpak.${NC}"
echo -e "${GREEN}==> You can now run it from your app menu or with: flatpak run com.CokieMiner.AnaFis${NC}"
