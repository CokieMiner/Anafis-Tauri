# AGENTS.md

## Project layout

- Tauri v2 app: React 19 + TypeScript 6 frontend, Rust (edition 2024) backend.
- **All commands run from `AnaFis/`**, not the repo root.
- Frontend: `AnaFis/src/`  |  Backend: `AnaFis/src-tauri/`
- Package manager: **Bun** (lockfile is `bun.lock`; `package-lock.json` is gitignored).
- Path alias: `@/` → `./src/` (Vite + TypeScript).

## Commands

| Command | What |
|---|---|
| `bun run dev` | Vite dev server (port 1420) |
| `bun run tauri:dev` | Full Tauri dev (Vite + Rust) |
| `bun run build` | Type-check then Vite build |
| `bun run tauri:build` | Production Tauri build → `../Installer/BuildOutputs` |
| `bun run lint` | Biome check (lint + format) |
| `bun run lint:fix` | Biome auto-fix |
| `bun run type-check` | `tsc --noEmit` |

**Build order matters**: `lint` → `type-check` → `build` before committing frontend changes.
Rust changes: `cargo clippy --all-targets --all-features` then `cargo test` from `AnaFis/src-tauri/`.

## Testing

- **No frontend test framework.** There are no `test` scripts in `package.json`.
- Rust tests: `cargo test` (key file: `src-tauri/src/scientific/curve_fitting/tests.rs`).
- No CI, no pre-commit hooks.

## Linting & style

- **Biome, not ESLint/Prettier.** Biome config at `AnaFis/biome.json`.
- Biome only lints TypeScript (HTML, CSS, Rust, and `dist/` are excluded).
- Formatter: 2-space indent, single quotes, semicolons always, ES5 trailing commas, 80-char width.
- Linter rules: `useImportType`, `noNonNullAssertion`, `noExplicitAny` are errors.

### Rust clippy is extremely strict

`Cargo.toml` denies: `unwrap_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `print_stdout`, `print_stderr`, `exit`, `dbg_macro`, `mem_forget`, `suboptimal_flops`, `imprecise_flops`, and many more. Use `.expect("reason")` for panics — never bare `unwrap()`. All `#[allow(...)]` must have a documented reason.

The Rust lib crate is named `anafis_lib` (not `anafis`) to avoid a Windows name collision with the binary.

## Architecture notes

### Multi-window architecture
The app spawns 5 separate webview windows, each with its own HTML entry point and React root:
- `index.html` — main window (tabs: Home, Spreadsheet, Fitting, Solver)
- `settings.html`, `data-library.html`, `uncertainty-calculator.html`, `latex-preview.html`

The main window starts **hidden** (`visible: false` in `tauri.conf.json`) and only becomes visible when the frontend emits `anafis://ready` via Tauri IPC, with a 2.5s timeout fallback (`src/shared/utils/windowReady.ts`). Always keep this flow intact when modifying startup.

### Key dependencies
- **State management**: Zustand (stores in `src/core/managers/`)
- **Spreadsheet engine**: Univer.js 0.21.1 (integration layer in `src/tabs/spreadsheet/univer/`)
- **Charts**: Plotly 3.5.1 (via `react-plotly.js`)
- **Math rendering**: KaTeX (`react-katex`)
- **Drag-and-drop tabs**: `@dnd-kit`
- **UI**: MUI v9 + Emotion
- **CAS engine**: `symb_anafis` (Rust crate for symbolic math)

### File format
`.anafispread` is a custom binary format: 6-byte magic header (`ANAFIS\x01\x00`) + u32 version + gzip-compressed JSON payload. Implementation in `src-tauri/src/export/anafispread.rs` and `src-tauri/src/import/anafispread.rs`.

### Vite chunking
The Vite config manually splits vendor chunks by package group (vendor-react, vendor-mui, vendor-plotly, vendor-univer-core, vendor-univer-sheets, etc.). When adding/removing dependencies that affect bundle size, update `vite.config.ts` accordingly. Minification drops `console.log`/`console.info` in production.

## Operating system notes
- Primary build targets: `deb`, `rpm` (Linux). Flatpak via `build_flatpak.sh`.
- Windows installer design document exists (`Installer/Windows/INSTALLER_DESIGN.md`) but not yet implemented.
