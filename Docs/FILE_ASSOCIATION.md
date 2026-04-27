# File Association for .anafispread Files (IMPLEMENTED)

*This feature is 100% complete. This document serves as a brief technical reference for the implemented architecture.*

## Core Mechanisms
1. **Magic Bytes**: Every exported file starts with `ANAFIS\x01\x00` and a `u32` little-endian version number. This prevents the OS from treating it as a generic gzip file.
2. **OS Registration**: `tauri.conf.json` registers the `.anafispread` MIME type (`application/x-anafis-spreadsheet`).
3. **Rust Handling**: On launch via double-click, `src-tauri/src/lib.rs` reads the file path from CLI arguments and emits an `open-file` event to the frontend.
4. **Export/Import Logic**: 
   - `export/anafispread.rs` writes the magic bytes + Gzip JSON. 
   - `import/anafispread.rs` verifies the signature before processing to avoid corruption.
5. **Frontend Reception**: `App.tsx` global effect listens for the `open-file` Tauri event and invokes the import commands to load the spreadsheet.

## Troubleshooting & Testing
- File associations **only work in production distributions**, not in development mode.
- If it opens an archive manager instead of AnaFis, the magic bytes might be missing (check via Hex editor) or the OS requires manual "Open With" registration once. 
- You must build (`bun run tauri build`) and install the `.deb`/`.rpm`/`.exe` for the OS file managers to bind the file type correctly.
