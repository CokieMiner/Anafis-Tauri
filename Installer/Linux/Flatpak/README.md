# AnaFis Flatpak Build Instructions

This directory contains the Flatpak manifest for building and distributing AnaFis on Linux.

## Prerequisites

```bash
# Install Flatpak and flatpak-builder
sudo apt install flatpak flatpak-builder  # Debian/Ubuntu
sudo dnf install flatpak flatpak-builder  # Fedora
sudo pacman -S flatpak flatpak-builder    # Arch

# Add Flathub repository
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Install required runtime and SDK
flatpak install flathub org.freedesktop.Platform//23.08
flatpak install flathub org.freedesktop.Sdk//23.08
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//23.08
```

## Building AnaFis Flatpak

### 1. Build the AnaFis binary first

```bash
cd ../../AnaFis
npm install
npm run tauri build
```

This creates the binary at `AnaFis/src-tauri/target/release/AnaFis`

### 2. Build the Flatpak

```bash
cd Installer/Linux/Flatpak

# Build and install locally
flatpak-builder --user --install --force-clean build-dir com.CokieMiner.AnaFis.yml

# Or build a bundle for distribution
flatpak-builder --repo=repo build-dir com.CokieMiner.AnaFis.yml
flatpak build-bundle repo anafis.flatpak com.CokieMiner.AnaFis
```

### 3. Test the Flatpak

```bash
# Run the locally installed version
flatpak run com.CokieMiner.AnaFis

# Or install from bundle
flatpak install anafis.flatpak
flatpak run com.CokieMiner.AnaFis
```

## What's Included

The Flatpak bundle includes:
- ✅ AnaFis application binary
- ✅ Python 3 runtime
- ✅ SymPy library (for symbolic mathematics)
- ✅ All required system libraries
- ✅ Application icon
- ✅ Desktop file for application menu
- ✅ AppStream metadata

## Distribution

### Publishing to Flathub

1. Fork the [Flathub repository](https://github.com/flathub/flathub)
2. Create a new branch with the app ID
3. Add the manifest and required files
4. Submit a pull request
5. Wait for review and approval

See [Flathub submission guide](https://docs.flathub.org/docs/for-app-authors/submission) for details.

### Direct Distribution

Distribute the `anafis.flatpak` bundle file via:
- GitHub Releases
- Project website
- Direct download

Users can install with:
```bash
flatpak install anafis.flatpak
```

## Permissions

The Flatpak has the following permissions:
- **Wayland/X11**: Window display
- **OpenGL**: Hardware acceleration
- **Network**: Updates and external resources
- **Home directory**: Import/export data files
- **D-Bus**: System integration
- **Audio**: Notifications (optional)

## Troubleshooting

### Python/SymPy not found
The Flatpak bundles Python and SymPy, so this should not happen. If it does:
```bash
flatpak run --command=python3 com.CokieMiner.AnaFis -c "import sympy; print(sympy.__version__)"
```

### Permission issues
Add more permissions temporarily for testing:
```bash
flatpak run --filesystem=host com.CokieMiner.AnaFis
```

### Build errors
Clean and rebuild:
```bash
rm -rf build-dir .flatpak-builder
flatpak-builder --user --install --force-clean build-dir com.CokieMiner.AnaFis.yml
```

## Files

- `com.CokieMiner.AnaFis.yml` - Main Flatpak manifest
- `com.CokieMiner.AnaFis.desktop` - Desktop entry file
- `com.CokieMiner.AnaFis.metainfo.xml` - AppStream metadata
- `README.md` - This file

## References

- [Flatpak Documentation](https://docs.flatpak.org/)
- [Flatpak Builder](https://docs.flatpak.org/en/latest/flatpak-builder.html)
- [Flathub Guidelines](https://docs.flathub.org/docs/for-app-authors/requirements)
