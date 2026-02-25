# AnaFis Windows Installer Design

## Purpose

Define a Windows installer and signing flow for AnaFis without runtime Python dependencies.

## Current Context

- AnaFis no longer requires Python/SymPy installation on user machines.
- Main concerns for Windows distribution are install UX and trust (signature / SmartScreen).

## Installer Scope

The installer should:
1. Install AnaFis binaries/resources
2. Create desktop/start-menu shortcuts (optional)
3. Register uninstall entry
4. Support silent install mode for automation

## Packaging Options

### Option A - MSI via Tauri bundle
- Simple baseline
- Good with code-signing pipelines

### Option B - NSIS EXE
- More control over UX and custom flows
- Good for single-file installer distribution

## Recommended Public Distribution

1. Build artifacts in CI
2. Sign artifacts
3. Publish signed installer + optional signed portable exe

## Signing Paths

### OSS path
- Use SignPath Foundation + GitHub Actions
- Keep signing policy and release branches/tags controlled

## Test Matrix

- Windows 10 clean VM
- Windows 11 clean VM
- Non-admin install path
- Silent install mode
- Uninstall and cleanup
- Signature verification (`Get-AuthenticodeSignature`)

## Distribution Channels

- GitHub Releases (primary)
- Winget (future)
- Chocolatey (future)

## Release Checklist

- [ ] Installer builds in CI
- [ ] Installer is signed
- [ ] Portable exe (if published) is signed
- [ ] Install/uninstall tested on clean VMs
- [ ] Checksums published
