# AnaFis (Tauri Edition)

AnaFis is a cross-platform desktop app for scientific data analysis.
It uses a Rust backend (computations and I/O) with a React/TypeScript UI (interaction and rendering).

## Project Status

- Main app architecture is stable and production-ready.
- Spreadsheet, fitting, data library, import/export, and utility windows are active.
- Build pipeline uses Bun + Vite + Tauri.
- Python runtime dependencies were removed from the application runtime.

## Main Features

- Multi-tab desktop workflow (home, spreadsheet, fitting, solver placeholders)
- Data Library (SQLite + search)
- Import/export in multiple formats
- Interactive charts and fitting workflow
- Dedicated utility windows (settings, uncertainty calculator, LaTeX preview)

## Tech Stack

### Backend
- Rust
- Tauri 2
- Serde, Tokio, Rusqlite, Arrow/Parquet, Nalgebra

### Frontend
- React 19 + TypeScript
- MUI
- Vite
- Zustand

## Prerequisites

- Rust toolchain (`rustup`)
- Bun (recommended) or Node.js/npm
- Platform build dependencies for Tauri

## Development

```bash
cd AnaFis
bun install
bun run tauri dev
```

## Production Build

```bash
cd AnaFis
bun run tauri build
```

Output is generated under `AnaFis/src-tauri/target/` (and/or configured bundle output directories).

## Architecture Principles

- Rust handles business logic, scientific calculations, data processing, and file operations.
- TypeScript/React handles UI state and user interactions.
- IPC between frontend and backend is explicit and typed.

## Contributing

1. Fork and create a feature branch.
2. Keep lint/build green.
3. Submit a pull request with clear scope and test notes.

## License

GPL-3.0-or-later. See `LICENSE`.
