<#
PowerShell helper to create a Python virtual environment and bundle SymPy
for use with the PyO3-embedded SymPy helper in this project.

Usage (Windows PowerShell):
  ./pack_python.ps1 -OutDir .\python_bundle -VenvName build_venv -PythonPath C:\Python310\python.exe

Notes:
- You must use the same Python interpreter/ABI that you used when building the Rust
  binary (the Python version that pyo3 was compiled against). If you built with
  Rust + PyO3 on this machine, use that same python.exe.
- The script creates a venv, installs SymPy, and copies the minimal runtime
  (python.exe + Lib site-packages) into an output folder that can be shipped with
  the application.
- This is a convenience script. Adjust paths and included files to suit your
  installer packaging strategy.
#>
param(
    [string]$OutDir = ".\python_bundle",
    [string]$VenvName = "build_venv",
    [string]$PythonPath = "python"
)

Write-Host "Using Python executable: $PythonPath"
Write-Host "Creating virtualenv: $VenvName"
& $PythonPath -m venv $VenvName
if ($LASTEXITCODE -ne 0) { throw "Failed to create venv" }

$py = Join-Path $VenvName "Scripts\python.exe"
Write-Host "Upgrading pip and installing sympy in venv..."
& $py -m pip install --upgrade pip setuptools wheel
& $py -m pip install sympy
if ($LASTEXITCODE -ne 0) { throw "Failed to install sympy in venv" }

# Prepare output folder
if (Test-Path $OutDir) { Remove-Item -Recurse -Force $OutDir }
New-Item -ItemType Directory -Path $OutDir | Out-Null

# Copy python executable(s)
Copy-Item -Path $py -Destination (Join-Path $OutDir "python.exe") -Force

# Copy the Lib folder (site-packages)
$srcLib = Join-Path $VenvName "Lib"
$dstLib = Join-Path $OutDir "Lib"
Write-Host "Copying Lib -> $dstLib (this may take a while)"
Copy-Item -Path $srcLib -Destination $dstLib -Recurse -Force

# Optionally create a zip
$zipPath = "$OutDir.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath }
Write-Host "Creating zip archive $zipPath"
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory((Resolve-Path $OutDir).Path, (Resolve-Path $zipPath).Path)

Write-Host "Python bundle created at: $OutDir and $zipPath"
Write-Host "Remember: use the same Python ABI used to build the Rust binary with PyO3."
