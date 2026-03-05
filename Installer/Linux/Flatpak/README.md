# AnaFis Flatpak Build Instructions

This directory contains the Flatpak manifest used to build and distribute AnaFis on Linux.

## Prerequisites

```bash
# Debian/Ubuntu
sudo apt install flatpak flatpak-builder

# Fedora
sudo dnf install flatpak flatpak-builder

# Arch
sudo pacman -S flatpak flatpak-builder

# Add Flathub
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Install runtime + SDK
flatpak install flathub org.gnome.Platform//48
flatpak install flathub org.gnome.Sdk//48
```

## Build Steps

### 1) Build AnaFis release binary

```bash
cd ../../../AnaFis
bun install
bun run tauri build
```

### 2) Build Flatpak

```bash
cd ../Installer/Linux/Flatpak

# Local install for testing
flatpak-builder --user --install --force-clean build-dir com.CokieMiner.AnaFis.yml

# Build distributable bundle
flatpak-builder --repo=repo build-dir com.CokieMiner.AnaFis.yml
flatpak build-bundle repo anafis.flatpak com.CokieMiner.AnaFis
```

### 3) Test

```bash
flatpak run com.CokieMiner.AnaFis
```

## Included in Bundle

- AnaFis application binary
- Required runtimes and shared libraries
- Desktop entry and icon
- AppStream metadata

## Distribution

### Flathub
1. Fork `flathub/flathub`
2. Add app manifest and metadata
3. Submit PR

### Direct
Ship `anafis.flatpak` and install with:

```bash
flatpak install anafis.flatpak
```

## Troubleshooting

### Permission issues

```bash
flatpak run --filesystem=host com.CokieMiner.AnaFis
```

### Rebuild clean

```bash
rm -rf build-dir .flatpak-builder
flatpak-builder --user --install --force-clean build-dir com.CokieMiner.AnaFis.yml
```

## References

- https://docs.flatpak.org/
- https://docs.flathub.org/
