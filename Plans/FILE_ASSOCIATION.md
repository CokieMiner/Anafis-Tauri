# File Association for .anafispread Files

This document explains how AnaFis handles `.anafispread` file associations, allowing users to double-click these files to open them in the application.

## File Format Structure

The `.anafispread` format includes a unique file signature (magic number) to identify it as an AnaFis file and prevent it from being treated as a generic compressed archive:

```
Bytes 0-7:    Magic number "ANAFIS\x01\x00" (identifies file type)
Bytes 8-11:   Format version (u32, little-endian, currently 1)
Bytes 12+:    Gzip-compressed JSON data (workbook snapshot)
```

### Why the Magic Number?

The magic number serves several purposes:
1. **File Type Identification**: The OS can quickly identify the file type without relying solely on the extension
2. **Prevents Misidentification**: Stops the OS from treating it as a generic `.gz` or `.zip` file
3. **Corruption Detection**: Helps detect corrupted or tampered files early
4. **Version Management**: Allows for future format changes while maintaining backward compatibility

## How the Complete System Works:

1. **User exports spreadsheet** → File created with magic number "ANAFIS\x01\x00" + version + compressed data
2. **User double-clicks file** → OS recognizes `.anafispread` and launches AnaFis
3. **Backend receives file path** → Emits `open-file` event to frontend
4. **Frontend imports file** → Calls `import_anafis_spread_direct`
5. **Import verifies signature** → Checks magic number and version
6. **Data loaded** → Spreadsheet tab opens with file contents

The magic number ensures the OS and file managers treat `.anafispread` files as AnaFis documents, not generic compressed archives!

### 1. **File Association Registration** (`tauri.conf.json`)

The file association is registered in the Tauri bundle configuration:

```json
{
  "bundle": {
    "fileAssociations": [
      {
        "ext": ["anafispread"],
        "name": "AnaFis Spreadsheet",
        "description": "AnaFis Spreadsheet File",
        "role": "Editor",
        "mimeType": "application/x-anafis-spreadsheet"
      }
    ]
  }
}
```

**What this does:**
- Registers `.anafispread` as a file type associated with AnaFis
- Sets the MIME type to prevent OS from treating it as a compressed archive
- On Linux: Creates `.desktop` file entries with the association
- On Windows: Registers the file type in the Windows Registry
- On macOS: Updates the application's Info.plist

### 2. **Backend File Open Handler** (`src-tauri/src/lib.rs`)

When the app is launched with a file argument (e.g., from double-clicking), the backend:

```rust
// Check for file association open
let args: Vec<String> = std::env::args().collect();
if args.len() > 1 {
    let file_path = &args[1];
    if file_path.ends_with(".anafispread") {
        // Emit event to frontend to handle the file
        app_handle.emit("open-file", file_path);
    }
}
```

**Flow:**
1. App launches with file path as command-line argument
2. Backend detects the `.anafispread` file
3. Emits `open-file` event to the frontend with the file path

### 3. **Export with File Signature** (`src-tauri/src/export/anafispread.rs`)

When exporting, the file is created with:

```rust
// Write magic number to identify file type
file.write_all(b"ANAFIS\x01\x00")?;

// Write format version (u32, little-endian)
file.write_all(&1u32.to_le_bytes())?;

// Write compressed JSON data
let encoder = GzEncoder::new(file, Compression::default());
serde_json::to_writer(encoder, &export_data)?;
```

This ensures every `.anafispread` file has:
- A unique signature that identifies it as an AnaFis file
- Version information for future compatibility
- Properly compressed data

### 4. **Import with Signature Verification** (`src-tauri/src/import/anafispread.rs`)

When importing, the file signature is verified:

```rust
// Read and verify magic number
let mut magic_buf = [0u8; 8];
file.read_exact(&mut magic_buf)?;

if &magic_buf != b"ANAFIS\x01\x00" {
    return Err("Invalid file format: Not an AnaFis Spreadsheet file");
}

// Read and verify format version
let mut version_buf = [0u8; 4];
file.read_exact(&mut version_buf)?;
let version = u32::from_le_bytes(version_buf);

// Then decompress and read the data...
```

This ensures:
- Only valid AnaFis files are accepted
- Corrupted files are detected immediately
- Version compatibility is checked before processing

### 5. **Frontend File Open Handler** (`src/App.tsx`)

The main App component listens for the `open-file` event:

```typescript
useEffect(() => {
  const unlisten = await listen<string>('open-file', async (event) => {
    const filePath = event.payload;
    
    // Import the file using the import command
    const result = await invoke('import_anafis_spread_direct', { filePath });
    
    if (result.success && result.data?.workbook) {
      // Create or switch to spreadsheet tab
      // Load the workbook data
      window.dispatchEvent(new CustomEvent('load-workbook-data', { 
        detail: result.data.workbook 
      }));
    }
  });
  
  return () => unlisten();
}, []);
```

**Flow:**
1. Listens for `open-file` event from backend
2. Calls `import_anafis_spread_direct` to read the file
3. Creates a spreadsheet tab if needed
4. Dispatches `load-workbook-data` event to load the data into Univer

## Platform-Specific Behavior

### Linux (DEB/RPM packages)
- The `.desktop` file includes a `MimeType` entry: `application/x-anafis-spreadsheet`
- File manager associates `.anafispread` files with AnaFis
- Double-clicking opens the app with the file path as an argument

### Windows
- Registry entries created under `HKEY_CLASSES_ROOT\.anafispread`
- File appears with AnaFis icon and description
- Double-clicking launches: `anafis.exe "path\to\file.anafispread"`

### macOS
- Info.plist updated with `CFBundleDocumentTypes` entry
- Finder associates `.anafispread` with AnaFis
- LaunchServices handles file opening

## Testing File Associations

### During Development
File associations only work in production builds, not in development mode.

### After Building
1. Build the application:
   ```bash
   npm run tauri build
   ```

2. Install the package:
   - **Linux DEB**: `sudo dpkg -i target/release/bundle/deb/*.deb`
   - **Linux RPM**: `sudo rpm -i target/release/bundle/rpm/*.rpm`
   - **Windows**: Run the installer `.exe`

3. Export a `.anafispread` file from within the app

4. Double-click the exported file to test the association

## Future Enhancements

- [ ] Handle multiple file opens (when user selects multiple files)
- [ ] Add "Open Recent" menu with file history
- [ ] Support drag-and-drop of `.anafispread` files onto the app window
- [ ] Add file icon for `.anafispread` files in the OS file manager

## Troubleshooting

**Problem:** Double-clicking opens archive manager instead of AnaFis
- **Cause:** The magic number and MIME type should prevent this, but some systems may need manual association
- **Solution:**
  - Linux: Right-click file → Properties → Open With → Choose AnaFis
  - Windows: Right-click → Open with → Choose another app → AnaFis
  - The magic number "ANAFIS\x01\x00" at the start of the file helps prevent this issue

**Problem:** "Invalid file format" error when opening
- **Cause:** File may be corrupted or created by an older version
- **Solution:** 
  - Check if the file starts with the magic bytes (use hex editor: should start with `41 4E 41 46 49 53 01 00`)
  - Try re-exporting from the original source
  - If it's from an older version without magic number, it won't be compatible

**Problem:** App opens but doesn't load the file
- **Solution:** Check the console logs for errors in the `open-file` event handler
- Ensure `import_anafis_spread_direct` command is working correctly

**Problem:** File association not registering
- **Solution:** Reinstall the application or manually update system file associations

## Technical Implementation Summary

### Changes Made to Support File Associations:

1. **Tauri Configuration** (`tauri.conf.json`)
   - Added `fileAssociations` with `.anafispread` extension
   - Defined MIME type: `application/x-anafis-spreadsheet`
   - Role: "Editor"

2. **Export Format** (`export/anafispread.rs`)
   - Added magic number: `ANAFIS\x01\x00` (8 bytes)
   - Added version field: u32 little-endian (4 bytes, currently version 1)
   - Maintains gzip compression for data
   - Uses standard Rust file I/O with `Write` trait

3. **Import Format** (`import/anafispread.rs`)
   - Verifies magic number on import
   - Checks version compatibility (currently supports version 1)
   - Provides clear error messages for invalid files
   - 100MB file size limit for safety
   - Uses `Read` trait for binary reading

4. **App Launch Handler** (`lib.rs`)
   - Detects file path from command-line args
   - Filters for `.anafispread` extension
   - Emits `open-file` event to frontend
   - Uses `tauri::Emitter` trait

5. **Frontend Handler** (`App.tsx`)
   - Listens for `open-file` events using `@tauri-apps/api/event`
   - Imports file using `invoke('import_anafis_spread_direct')`
   - Creates or switches to spreadsheet tab
   - Loads workbook data with `load-workbook-data` custom event

### Dependencies Used:
```toml
flate2 = "1.1.5"          # Gzip compression/decompression
tauri = { features = ["unstable"] }  # Event emitter
```

This complete implementation ensures `.anafispread` files are properly recognized as AnaFis documents by the operating system and can be opened with a double-click!

## Related Documentation

- See `IMPORT_SYSTEM.md` for details on all import formats
- See `EXPORT_SYSTEM.md` (if exists) for details on all export formats
- See Plans/sidebars/ for sidebar implementation details
