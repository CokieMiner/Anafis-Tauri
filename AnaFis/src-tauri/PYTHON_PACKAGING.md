Runtime packaging notes for embedded SymPy (used by `uncertainty.rs`)

Overview
- `src-tauri/src/uncertainty.rs` embeds `python_formulas.py` and calls SymPy via PyO3.
- At runtime the app must be able to attach to a Python interpreter and import SymPy.

Options to ship Python+SymPy
1) Ship a small embedded Python bundle (recommended for Windows):
   - Use the provided `pack_python.ps1` script to create a venv with SymPy and bundle
     the `python.exe` + `Lib` folder into `python_bundle`.
   - Ship `python_bundle` alongside the application and set the app to use that
     python binary (or point the embedded interpreter to it).

2) Require system Python with SymPy installed:
   - Document the required Python version and `pip install sympy` in your installer.

3) Use a subprocess approach (alternative):
   - Instead of embedding Python with PyO3, run a small Python subprocess that
     exposes the required functions (JSON-RPC or simple stdin/stdout protocol).
   - This decouples the Rust binary from Python ABI issues but adds IPC overhead.

Important notes
- PyO3 and in-process Python require the Python ABI used at build time to match
  the runtime Python. On Windows, prefer using the embeddable distribution or a
  virtualenv created with the same python.exe you used when building.
- The `pack_python.ps1` script helps create a portable bundle, but you must ensure
  any native extensions in site-packages are compatible with target OS/arch.

Troubleshooting
- If the app fails with "Failed to attach to Python interpreter", ensure a
  compatible Python is available in PATH or that you have bundled one and are
  pointing the runtime to it.
- If the Python call fails with an exception, the Rust code attempts to include
  a traceback in the error string when possible.

Contact
- For help integrating packaging into CI/CD or building installer artifacts,
  I can add scripts for 1) producing a zipped python bundle during CI, 2) an
  installer step that places the bundle alongside the app, and 3) runtime logic
  to locate and use the bundled Python interpreter.
