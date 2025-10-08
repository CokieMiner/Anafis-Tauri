# AnaFis Distribution Strategy

**Status**: Ready for implementation  
**Target Platforms**: Linux (multiple formats), Windows (custom installer)

---

## Linux Distribution (Primary Platform)

### ✅ Distribution Methods

1. **Flatpak** (Universal - Recommended)
   - Location: `Installer/Linux/Flatpak/`
   - Status: Manifest ready
   - Distribution: Flathub + direct download
   - Installation: `flatpak install flathub com.cokieminer.anafis`
   - **Includes**: Python 3 + SymPy bundled automatically

2. **AUR (Arch Linux)**
   - Location: `Installer/Linux/AUR/` (to be created)
   - Status: Planned
   - Distribution: AUR repository
   - Installation: `yay -S anafis` or `paru -S anafis`
   - **Dependencies**: python, python-sympy (declared in PKGBUILD)

3. **.deb (Debian/Ubuntu)**
   - Location: `Installer/Linux/Debian/` (to be created)
   - Status: Planned
   - Distribution: GitHub Releases + PPA
   - Installation: `sudo dpkg -i anafis_*.deb`
   - **Dependencies**: python3 (>= 3.8), python3-sympy (in debian/control)

4. **.rpm (Fedora/RHEL)**
   - Location: `Installer/Linux/RPM/` (to be created)
   - Status: Planned
   - Distribution: GitHub Releases + COPR
   - Installation: `sudo dnf install anafis-*.rpm`
   - **Dependencies**: python3 >= 3.8, python3-sympy (in .spec file)

### Key Points
- ✅ **No manual Python installation needed** - Package managers handle it
- ✅ **Dependencies declared** in package metadata
- ✅ **Automatic updates** via package manager
- ✅ **Standard Linux packaging** follows best practices

---

## Windows Distribution

### Custom Installer Application

**Location**: `Installer/Windows/INSTALLER_DESIGN.md`

**Approach**: Lightweight installer app (Tauri or NSIS) that:
1. Checks for Python 3.8+
2. Installs Python + SymPy if missing
3. Installs AnaFis binary
4. Creates shortcuts
5. Registers in Add/Remove Programs

**Distribution**:
- GitHub Releases (primary)
- Project website
- Future: Chocolatey, Winget

**Why custom installer?**:
- Windows doesn't have universal package manager
- Python not pre-installed on Windows
- Users expect simple .exe installer
- Can bundle Python or download on-demand

---

## macOS Distribution (Future)

**Planned Approach**:
- .dmg application bundle
- Homebrew cask: `brew install --cask anafis`
- Dependencies: Homebrew handles Python + SymPy

---

## Current Implementation Status

### ✅ Ready
- [x] Flatpak manifest with Python dependencies
- [x] Desktop file for Linux application menu
- [x] AppStream metadata for app stores
- [x] Windows installer design document
- [x] README updated with installation instructions

### 🔄 In Progress
- [ ] Test Flatpak build process
- [ ] Create AUR PKGBUILD
- [ ] Create .deb package with debian/control
- [ ] Create .rpm package with .spec file

### 📋 Todo
- [ ] Windows installer implementation (Tauri or NSIS)
- [ ] Bundle Python 3.11 installer for Windows
- [ ] Test all package formats on clean VMs
- [ ] Submit to Flathub for distribution
- [ ] Create COPR repository for Fedora
- [ ] Create PPA for Ubuntu

---

## Build Requirements

### Linux Packages
```bash
# Debian/Ubuntu
sudo apt install flatpak-builder debhelper

# Fedora
sudo dnf install flatpak-builder rpm-build

# Arch
sudo pacman -S flatpak-builder
```

### Windows Installer
- Tauri development environment
- OR NSIS compiler (simpler option)
- Python 3.11 installer (bundled)

---

## Distribution Checklist

Before releasing v0.1.0:

**Linux**:
- [ ] Test Flatpak on Fedora
- [ ] Test Flatpak on Ubuntu
- [ ] Build .deb package
- [ ] Build .rpm package
- [ ] Create AUR package
- [ ] Submit Flatpak to Flathub

**Windows**:
- [ ] Implement installer app
- [ ] Test on Windows 10 (clean VM)
- [ ] Test on Windows 11 (clean VM)
- [ ] Test with existing Python installation
- [ ] Test without Python (fresh install)

**Documentation**:
- [x] Update README with installation methods
- [x] Document Python requirements
- [ ] Create installation troubleshooting guide
- [ ] Add screenshots for AppStream metadata

---

## Files Created

### Flatpak
- `Installer/Linux/Flatpak/com.CokieMiner.AnaFis.yml` - Flatpak manifest
- `Installer/Linux/Flatpak/com.CokieMiner.AnaFis.desktop` - Desktop entry
- `Installer/Linux/Flatpak/com.CokieMiner.AnaFis.metainfo.xml` - AppStream metadata
- `Installer/Linux/Flatpak/README.md` - Build instructions

### Windows
- `Installer/Windows/INSTALLER_DESIGN.md` - Complete installer design

### Documentation
- `IMPLEMENTATION_ANALYSIS.md` - Code analysis and architecture review
- `README.md` - Updated with distribution methods

---

## Next Steps

1. **Test Flatpak build** on your Linux system
2. **Choose Windows installer approach**: Tauri app vs NSIS script
3. **Create AUR package** for Arch Linux
4. **Start implementing Data Library Window** (core feature for v0.1.0)

---

**Priority**: Linux packages first (your primary platform), Windows installer later.
