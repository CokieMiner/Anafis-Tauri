# AnaFis Windows Installer Application Design

**Purpose**: Simple installer app that handles Python dependency installation and AnaFis setup on Windows.

---

## Overview

A lightweight Tauri application that:
1. Checks for Python 3.13 installation
2. Installs Python + SymPy if missing
3. Installs AnaFis main application
4. Creates desktop shortcuts
5. Handles uninstallation

---

## Architecture

### Technology Stack
- **Framework**: Tauri (same as main app for consistency)
- **Frontend**: Simple HTML/CSS/JavaScript (no React needed)
- **Backend**: Rust for system operations
- **Python Installer**: Embed Python installer or use WinGet

### File Structure
```
anafis-installer/
├── src/
│   ├── index.html          # Main installer UI
│   ├── styles.css          # Installer styling
│   ├── installer.js        # Frontend logic
│   └── assets/
│       ├── logo.png
│       └── progress.gif
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── installer.rs    # Installation logic
│   │   ├── python.rs       # Python detection & installation
│   │   └── registry.rs     # Windows Registry operations
│   ├── Cargo.toml
│   └── tauri.conf.json
├── bundled/
│   ├── python-installer.exe    # Python 3.11+ installer
│   └── AnaFis.exe              # Main application
└── README.md
```

---

## User Flow

### Installation Steps

1. **Welcome Screen**
   - AnaFis logo and description
   - License agreement
   - Install location selection (default: `C:\Program Files\AnaFis`)
   - [Next] button

2. **Dependency Check**
   - Check for Python 3.8+ in PATH
   - If found: Show Python version ✓
   - If not found: Show warning and prepare to install

3. **Python Installation** (if needed)
   - Download or use bundled Python installer
   - Run Python installer with flags:
     ```
     python-3.13.x-amd64.exe /quiet InstallAllUsers=1 PrependPath=1
     ```
   - Wait for completion
   - Verify Python is accessible

4. **SymPy Installation**
   - Show progress: "Installing SymPy..."
   - Run: `python -m pip install sympy`
   - Verify import: `python -c "import sympy"`

5. **AnaFis Installation**
   - Copy AnaFis.exe to install directory
   - Create desktop shortcut
   - Create Start Menu entry
   - Register in Windows Add/Remove Programs

6. **Completion**
   - Show success message
   - Option to launch AnaFis
   - [Finish] button

---

## Implementation Details

### Rust Backend Commands

#### 1. Python Detection
```rust
#[tauri::command]
async fn check_python() -> Result<PythonInfo, String> {
    // Check PATH for python.exe
    // Get version: python --version
    // Check SymPy: python -c "import sympy; print(sympy.__version__)"
}

struct PythonInfo {
    installed: bool,
    version: Option<String>,
    path: Option<String>,
    sympy_installed: bool,
    sympy_version: Option<String>,
}
```

#### 2. Python Installation
```rust
#[tauri::command]
async fn install_python(install_path: String) -> Result<(), String> {
    // Extract bundled python-installer.exe
    // Run installer with silent flags
    // Wait for completion
    // Verify installation
}
```

#### 3. SymPy Installation
```rust
#[tauri::command]
async fn install_sympy() -> Result<(), String> {
    // Run: python -m pip install sympy
    // Stream output to frontend for progress
    // Verify installation
}
```

#### 4. AnaFis Installation
```rust
#[tauri::command]
async fn install_anafis(install_path: String) -> Result<(), String> {
    // Create installation directory
    // Copy AnaFis.exe and resources
    // Create shortcuts (desktop + start menu)
    // Register in Windows Registry
}
```

#### 5. Registry Operations
```rust
fn register_uninstaller(install_path: &str, version: &str) -> Result<(), String> {
    // Add entry to: HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AnaFis
    // Required keys:
    //   - DisplayName: "AnaFis"
    //   - DisplayVersion: version
    //   - Publisher: "CokieMiner"
    //   - InstallLocation: install_path
    //   - UninstallString: path to uninstaller
    //   - DisplayIcon: path to icon
}
```

---

## UI Design

### Simple, Modern Interface

```
┌─────────────────────────────────────────────────┐
│                                                 │
│          [AnaFis Logo]                          │
│                                                 │
│     Advanced Numerical Analysis and Fitting     │
│          Interface System - Installer           │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │ Installation Progress                     │ │
│  │                                           │ │
│  │ ✓ Checking Python installation...        │ │
│  │ ▶ Installing Python 3.11...              │ │
│  │   [■■■■■■■■░░░░░░░░░░] 45%              │ │
│  │                                           │ │
│  │ ⏸ Installing SymPy...                     │ │
│  │ ⏸ Installing AnaFis...                    │ │
│  │                                           │ │
│  └───────────────────────────────────────────┘ │
│                                                 │
│  Install Location:                              │
│  [C:\Program Files\AnaFis    ] [Browse...]     │
│                                                 │
│            [Cancel]              [Next]         │
└─────────────────────────────────────────────────┘
```

---

## Bundling Strategy

### Option 1: Embed Python Installer
**Pros**: Offline installation, controlled version
**Cons**: Larger installer (~30MB)

```toml
# In tauri.conf.json
{
  "bundle": {
    "resources": [
      "bundled/python-3.11.9-amd64.exe",
      "bundled/AnaFis.exe"
    ]
  }
}
```

### Option 2: Download Python Installer
**Pros**: Smaller installer (~5MB)
**Cons**: Requires internet connection

```rust
async fn download_python_installer() -> Result<PathBuf, String> {
    // Download from python.org
    // Show progress to user
    // Verify checksum
}
```

**Recommendation**: Option 1 (Embed) - Better user experience

---

## Uninstaller

The installer should also create an uninstaller:

```rust
// uninstall.rs
fn uninstall_anafis(install_path: &str) -> Result<(), String> {
    // Remove installation directory
    // Remove desktop shortcut
    // Remove Start Menu entry
    // Remove registry entry
    // Ask if user wants to keep Python (it may be used by other apps)
}
```

---

## Dependencies (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
winreg = "0.52"  # Windows Registry access
reqwest = { version = "0.11", features = ["stream"] }  # For downloading Python
zip = "0.6"  # For extracting bundled files
```

---

## Build Instructions

```bash
# Development
npm run tauri dev

# Build installer
npm run tauri build

# Output: anafis-installer_0.1.0_x64_en-US.msi
```

---

## Alternative: NSIS Script

If a full Tauri app is overkill, consider using NSIS (Nullsoft Scriptable Install System):

```nsis
; anafis-installer.nsi
!include "MUI2.nsh"

Name "AnaFis"
OutFile "anafis-installer.exe"
InstallDir "$PROGRAMFILES\AnaFis"

Section "Python Check"
  ; Check for Python
  ExecWait 'python --version' $0
  ${If} $0 != 0
    ; Install Python
    File "python-installer.exe"
    ExecWait '"python-installer.exe" /quiet InstallAllUsers=1 PrependPath=1'
  ${EndIf}
  
  ; Install SymPy
  ExecWait 'python -m pip install sympy'
SectionEnd

Section "Install AnaFis"
  File "AnaFis.exe"
  CreateShortcut "$DESKTOP\AnaFis.lnk" "$INSTDIR\AnaFis.exe"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd
```

**Recommendation**: Use NSIS for simplicity, unless you need:
- Custom UI with real-time progress
- Complex error handling
- Interactive dependency resolution

---

## Testing Checklist

- [ ] Fresh Windows 10/11 without Python
- [ ] Windows with Python 3.7 (upgrade needed)
- [ ] Windows with Python 3.11+ already installed
- [ ] Windows with SymPy already installed
- [ ] Installation to custom directory
- [ ] Installation with limited user permissions
- [ ] Uninstallation removes all files
- [ ] Shortcuts work correctly
- [ ] AnaFis launches after installation

---

## Future Enhancements

1. **Auto-update**: Check for AnaFis updates
2. **Repair**: Reinstall corrupted files
3. **Multiple Python versions**: Detect and use existing Python installations
4. **Silent install**: Command-line installation for enterprise deployment
   ```
   anafis-installer.exe /S /D=C:\Custom\Path
   ```

---

## Distribution

Once built, distribute via:
1. **GitHub Releases**: Primary distribution
2. **Website**: Direct download
3. **Chocolatey**: `choco install anafis` (future)
4. **Winget**: `winget install CokieMiner.AnaFis` (future)

---

**Next Steps**:
1. Choose between Tauri installer app or NSIS script
2. Test Python installation flow on clean Windows VM
3. Bundle Python 3.11 installer
4. Create installer icon and branding
5. Write installation/uninstallation tests
