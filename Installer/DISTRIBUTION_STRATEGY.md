# AnaFis Distribution Strategy

## Scope

This document defines practical distribution paths for AnaFis now that runtime Python dependencies are removed.

## Goals

- Reliable install/update flow on Linux and Windows
- Predictable first-run behavior
- Trusted binaries for end users (especially Windows)

## Linux Strategy

### Primary
1. Flatpak
   - Path: `Installer/Linux/Flatpak/`
   - Distribution: Flathub + direct bundle

### Secondary
2. Native packages (AUR, .deb, .rpm)
   - Add only if maintenance capacity is available

### Notes
- No Python runtime packaging is required for AnaFis.
- Keep metadata and icons aligned with app version.

## Windows Strategy

### Level 0 (testing)
- Distribute raw `AnaFis.exe`
- Fastest path, but users may see SmartScreen/Defender warnings

### Level 1 (serious OSS distribution)
- Build in GitHub Actions
- Sign artifacts using SignPath Foundation (OSS)
- Publish signed assets in Releases

## Recommended Release Artifacts

### Linux
- `anafis.flatpak` (and/or Flathub publication)

## CI/CD Baseline (Windows)

1. Build release artifact
2. Submit artifact for signing (SignPath or cert-based step)
3. Verify signature in CI
4. Upload signed artifacts to Release

## Checklist Before Public Release

### Functional
- [ ] Clean build on CI
- [ ] App launches on clean Windows 10/11 VM
- [ ] App launches on target Linux distro(s)

### Trust/Signing
- [ ] Windows artifacts are signed
- [ ] Timestamp is present in signature
- [ ] `Get-AuthenticodeSignature` verification passes

### Packaging
- [ ] Version is consistent across app and release notes
- [ ] Checksums are published for downloadable artifacts

## Related Docs

- `Installer/Linux/Flatpak/README.md`
- `Installer/Windows/INSTALLER_DESIGN.md`
- `README.md`
