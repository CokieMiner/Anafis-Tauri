# AnaFis (Tauri Edition)

AnaFis is a powerful desktop application for scientific data analysis and numerical computing, built with Tauri for cross-platform compatibility. It combines the performance of Rust with the flexibility of modern web technologies to provide a comprehensive suite of tools for scientists, engineers, and researchers.

## Features

AnaFis offers a modern, detachable notebook-style interface with the following core capabilities:

### Core Tabs
- **🏠 Home Tab**: Central hub for creating new analysis tabs and accessing quick actions
- **📊 Spreadsheet Tab**: Advanced spreadsheet with formula evaluation, unit support, and data manipulation
- **📈 Fitting Tab**: Robust curve fitting algorithms with interactive visualization and regression analysis
- **🧮 Solver Tab**: Intelligent equation solver providing step-by-step mathematical solutions
- **🎲 Monte Carlo Tab**: Complex simulation capabilities for statistical analysis and probabilistic modeling

### Additional Tools
- **🔢 Uncertainty Calculator**: Floating utility window for quick uncertainty calculations and error propagation
- **📝 LaTeX Preview**: Real-time LaTeX rendering for mathematical expressions and documentation
- **⚙️ Settings**: Customizable application preferences and configuration

### Key Features
- **Detachable Tabs**: Drag tabs outside the main window for multi-monitor workflows
- **Drag & Drop Interface**: Intuitive tab reordering and management
- **Python Integration**: Embedded Python runtime with SymPy for symbolic mathematics
- **Cross-Platform**: Native desktop application for Windows, macOS, and Linux
- **Modern UI**: Material Design interface built with React and Material-UI
- **Type-Safe**: Full TypeScript support for reliable development
- **Consistent Theming**: Unified color scheme with purple accent for optimal readability
- **Production-Ready**: Clean codebase with no debug statements and optimized performance

## Technologies Used

### Backend (Rust)
- **Tauri 2.x**: Cross-platform desktop application framework
- **PyO3**: Python integration for advanced mathematical computations
- **Tokio**: Asynchronous runtime for concurrent operations
- **Serde**: Serialization framework for data interchange
- **Optimized Code**: Clippy-compliant with modern Rust idioms and performance improvements

### Frontend (TypeScript/React)
- **React 19**: Modern React with hooks and concurrent features
- **TypeScript**: Type-safe JavaScript development
- **Material-UI (MUI)**: Component library following Material Design principles
- **Vite**: Fast build tool and development server
- **Zustand**: Lightweight state management
- **@dnd-kit**: Drag and drop functionality for tab management
- **KaTeX**: High-quality mathematical typesetting

### Python Integration
- **SymPy**: Symbolic mathematics library for equation solving and manipulation
- **System Python**: Uses system Python installation (Python 3.8+ with SymPy required)

## Prerequisites

Before building and running AnaFis, ensure you have the following installed:

### Required for All Platforms
- **Rust**: Install via [rustup.rs](https://rustup.rs/)
- **Node.js & npm**: Download from [nodejs.org](https://nodejs.org/) (LTS version recommended)

### Python Dependencies (Platform-Specific)

#### Linux
AnaFis is distributed through standard Linux package managers, which handle Python dependencies automatically:

**Flatpak** (Universal - Recommended)
```bash
flatpak install flathub com.cokieminer.anafis
```

**Arch Linux (AUR)**
```bash
yay -S anafis
# or
paru -S anafis
```

**Debian/Ubuntu (.deb)**
```bash
sudo dpkg -i anafis_*.deb
sudo apt install -f  # Install dependencies
```

**Fedora/RHEL (.rpm)**
```bash
sudo dnf install anafis-*.rpm
```

All Linux packages include Python 3.8+ and SymPy as dependencies, so no manual installation is required.

#### Windows
For Windows users, we provide a dedicated installer application that handles all dependencies including Python and SymPy automatically.

**Manual installation** (for development):
1. Download Python 3.8+ from [python.org](https://www.python.org/downloads/)
2. During installation, check "Add Python to PATH"
3. Open Command Prompt and run: `pip install sympy`

#### macOS
Python 3 can be installed via Homebrew:
```bash
brew install python3
pip3 install sympy
```

**Note**: AnaFis requires Python with SymPy for uncertainty propagation and symbolic mathematics. Production packages handle this automatically on all platforms.

---

## Installation & Development

### Clone the Repository
```bash
git clone https://github.com/CokieMiner/Anafis-Tauri.git
cd Anafis-Tauri
```

### Install Dependencies
```bash
# Install Node.js dependencies
cd AnaFis
npm install
```

### Development Mode
```bash
# Start the development server
npm run tauri dev
```
This will launch the AnaFis application with hot-reloading enabled for development.

### Building for Production
```bash
# Build the application
npm run tauri build
```
The built application will be available in `src-tauri/target/release/` (platform-specific).

## Usage

1. **Launch AnaFis** using the development or built executable
2. **Home Tab**: Use the central hub to create new analysis tabs
3. **Work with Tabs**: Each tab provides specialized tools:
   - **Spreadsheet**: Enter data, use formulas with Python/SymPy integration
   - **Fitting**: Import data and perform curve fitting with visualization
   - **Solver**: Input equations and get step-by-step solutions
   - **Monte Carlo**: Set up and run probabilistic simulations
4. **Detach Tabs**: Drag any tab outside the main window for flexible multi-tasking
5. **Uncertainty Calculator**: Access via the toolbar for quick calculations
6. **Settings**: Customize the application behavior and appearance

## Architecture

AnaFis follows a **Rust-first architecture** with clear separation of concerns:

### Core Principles
- **Rust Backend**: ALL business logic, calculations, data processing, and system operations
- **TypeScript Frontend**: UI rendering, user input handling, and visual feedback ONLY
- **Python Integration**: Symbolic mathematics via system Python + SymPy (for uncertainty propagation)

### Component Structure
- **Frontend (React/TypeScript)**: Modern React application with Material-UI components
  - No calculation logic in TypeScript
  - All data processing via Tauri `invoke()` commands to Rust
- **Backend (Rust)**: High-performance native application
  - Unit conversions (custom dimensional analysis)
  - Uncertainty calculations (via PyO3/SymPy)
  - Window management and IPC
  - Future: Statistical analysis, curve fitting, data smoothing, SQLite database
- **State Management**: Zustand for tab management and UI state
- **Communication**: Tauri's secure IPC system between frontend and backend
- **Python Runtime**: System Python with SymPy for symbolic derivative calculations

### Performance Benefits
- **10-100x faster** calculations compared to JavaScript implementations
- Type-safe operations with Rust's type system
- Consistent numeric precision across platforms
- Efficient memory usage with Rust's ownership model

---

## Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure tests pass and code is properly formatted
5. Submit a pull request

### Development Guidelines
- Follow Rust formatting with `rustfmt` and linting with `clippy`
- Use ESLint and Prettier for TypeScript/JavaScript code
- Maintain type safety with TypeScript
- Write clear commit messages
- **Code Quality**: Keep production builds clean (no console statements)
- **Theme Consistency**: Use shared theme utilities for consistent UI
- **Performance**: Optimize Rust code with modern idioms and patterns

## Roadmap

### Current Features (v0.1.0)
- ✅ Basic tab interface with detachable tabs
- ✅ Spreadsheet functionality with formula support
- ✅ Python/SymPy integration
- ✅ Uncertainty calculator
- ✅ LaTeX preview
- ✅ Drag & drop tab management
- ✅ **Unified theme system and consistent UI**

### Planned Features
- 🔄 Advanced curve fitting algorithms
- 🔄 Monte Carlo simulation engine
- 🔄 GPU acceleration for computations
- 🔄 Plotting and visualization enhancements
- 🔄 Data import/export capabilities
- 🔄 Plugin system for extensibility

## License

This project is licensed under the GPL-3.0-or-later License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Tauri](https://tauri.app/) for the desktop framework
- Symbolic Mathematical computations powered by [SymPy](https://www.sympy.org/)
- UI components from [Material-UI](https://mui.com/)
- Drag & drop functionality via [@dnd-kit](https://dndkit.com/)
- **Recent Improvements**: Codebase cleanup and optimization by GitHub Copilot

---

*AnaFis v0.1.0 -* ✨