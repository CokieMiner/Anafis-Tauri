# ANAFIS Project Plan (Tauri Edition)

This document outlines the comprehensive plan for the ANAFIS desktop application, adapted for its Tauri implementation. It consolidates design principles, requirements, architectural details, and an implementation roadmap.

## 1. Executive Summary

ANAFIS is envisioned as a **detachable-notebook** desktop application for scientific data analysis. Its core functionality revolves around a tabbed interface where major capabilities—Spreadsheet, Curve-Fitting, and Wolfram-like Solver—reside in their own closable and detachable tabs, spawned from a central **Home Menu**. A small, floating Uncertainty Calculator is also available. The Monte Carlo module remains in the codebase as a placeholder and is currently hidden from tab-creation UI. Each tab is designed for reusability, with GPU acceleration where beneficial, and communicates via a light **shared data-bus** implemented through Tauri's inter-process communication (IPC).

**Current Status**: Core infrastructure is complete with a fully functional spreadsheet application featuring advanced import/export capabilities, data visualization, and scientific computation tools. The application demonstrates production-ready code quality with zero linting errors and comprehensive type safety.

## 2. Code Guidelines

To ensure code quality and maintainability, the project adheres to the following guidelines and utilizes specific tooling for Rust and web technologies:

### Rust (Backend)
-   **`rustfmt`**: For consistent code formatting.
-   **`clippy`**: For linting and identifying common pitfalls.

### TypeScript/JavaScript (Frontend)
-   **`Biome`**: For linting and formatting (single-tool pipeline).
-   **`TypeScript`**: For static type checking, enhancing code reliability and scalability.

### General Principles
-   **Functional Programming**: Emphasis on pure functions and immutable data structures, particularly in the frontend (React components) and where applicable in Rust.
-   **Immutable State**: Application state is managed through appropriate patterns (e.g., React hooks, Zustand/Jotai/Redux for frontend; immutable data structures in Rust).
-   **Tauri Integration**: Prioritize minimal Rust-specific UI, leveraging Tauri to integrate a rich, web-based functional core.
-   **Library Reuse**: Maximize the use of existing, well-vetted Rust crates and web libraries.

### Code Quality Achievements ✅
-   **Biome**: 0 errors/warnings in checked files, standardized formatting and linting
-   **TypeScript**: 100% type coverage, strict null checks, no 'any' types
-   **Rust**: Clippy compliant, modern Rust idioms, optimized performance
-   **Build System**: Clean compilation, optimized bundles, zero runtime warnings

## 3. Repository Layout

The project structure is organized to separate frontend (web) and backend (Rust) concerns within the Tauri framework:

```
Anafis-Tauri/
├── LICENSE
├── README.md
├── AnaFis/                     # Main application directory
│   ├── data-library.html       # Data Library window HTML
│   ├── biome.json              # Biome lint/format configuration
│   ├── index.html              # Main application HTML
│   ├── latex-preview.html      # LaTeX preview window HTML
│   ├── package.json            # Node.js dependencies and scripts
│   ├── settings.html           # Settings window HTML
│   ├── tab.html                # Detached tab window HTML
│   ├── tsconfig.json           # TypeScript configuration
│   ├── tsconfig.node.json      # TypeScript config for build tools
│   ├── uncertainty-calculator.html # Uncertainty calculator window HTML
│   ├── vite.config.ts          # Vite build configuration
│   ├── public/                 # Static assets
│   ├── src/                    # Frontend (React/TypeScript) application code
│   │   ├── core/               # Core application logic
│   │   │   ├── contexts/       # React contexts for state management
│   │   │   ├── managers/       # State management and business logic
│   │   │   ├── types/          # Core TypeScript type definitions
│   │   │   └── utils/          # Core utility functions
│   │   ├── shared/             # Shared components and utilities
│   │   │   ├── components/     # Reusable UI components
│   │   │   ├── dataLibrary/    # Data Library specific components
│   │   │   ├── types/          # Shared type definitions
│   │   │   ├── uncertaintyCalculator/ # Uncertainty calculator components
│   │   │   └── utils/          # Shared utility functions
│   │   ├── tabs/               # Tab components
│   │   │   ├── fitting/        # Curve fitting tab (active development)
│   │   │   ├── home/           # Home tab
│   │   │   ├── montecarlo/     # Monte Carlo tab (placeholder, deferred in UI)
│   │   │   ├── solver/         # Equation solver tab (placeholder)
│   │   │   └── spreadsheet/    # Spreadsheet tab with sidebars
│   │   ├── types/              # Global type definitions
│   │   ├── windows/            # Window components
│   │   │   ├── DataLibraryWindow.tsx
│   │   │   ├── SettingsWindow.tsx
│   │   │   └── uncertaintyCalculator/
│   │   ├── App.tsx             # Main React application entry point
│   │   ├── icons.ts            # Icon definitions
│   │   ├── index.css           # Global styles
│   │   ├── main.tsx            # React entry point
│   │   └── shared-styles.css   # Shared component styles
│   └── src-tauri/              # Rust Backend (Tauri application core)
│       ├── build.rs            # Rust build script
│       ├── capabilities/       # Tauri security capabilities
│       ├── Cargo.lock          # Rust dependency lock file
│       ├── Cargo.toml          # Rust project manifest and dependencies
│       ├── gen/                # Generated code
│       ├── icons/              # Application icons
│       ├── python/             # Python integration (SymPy)
│       ├── src/                # Rust source code
│       │   ├── data_library/   # Data Library (SQLite + FTS5)
│       │   ├── error.rs        # Error handling types
│       │   ├── export/         # Export system (10 formats)
│       │   ├── import/         # Import system (multiple formats)
│       │   ├── lib.rs          # Library entry point and Tauri commands
│       │   ├── main.rs         # Main Rust application entry point
│       │   ├── scientific/     # Scientific computations
│       │   ├── uncertainty_calculator/ # Uncertainty calculation logic
│       │   ├── unit_conversion/ # Unit conversion system
│       │   ├── utils/          # Utility modules
│       │   └── windows/        # Window management
│       └── tauri.conf.json     # Tauri configuration file
├── Installer/                  # Installation and distribution files
│   ├── DISTRIBUTION_STRATEGY.md
│   ├── Linux/
│   │   ├── Arch/
│   │   └── Flatpak/
│   └── Windows/
│       └── INSTALLER_DESIGN.md
└── Plans/                      # Project planning documents
    ├── anafis_tauri_plan.md
    ├── FILE_ASSOCIATION.md
    ├── uncertanty_cell_plan.md
    └── sidebars/
        ├── 02_statistical_analysis_sidebar.md
        ├── 03_data_smoothing_sidebar.md
        ├── 04_outlier_detection_sidebar.md
        └── README.md
```

## 4. Library Map (Tauri Edition)

This table outlines the primary libraries and crates intended for use across different modules, justifying their selection:

| Module | Primary Lib (Rust) | Primary Lib (Frontend) | Why |
|---|---|---|---|
| **Shell/Notebook** | `tauri`, `tauri-plugin-window` | React, Material-UI | For building the desktop application shell, managing native windows, and creating a responsive UI with consistent Material Design principles. |
| **Tabs/Solver Tab** | `sympy` through PyO3, `nalgebra` | React, MathJax/KaTeX, ECharts, Material-UI | To provide symbolic mathematics capabilities, numerical computation for solving equations, live LaTeX rendering, interactive plotting of solutions, and consistent UI. **Uses ECharts** for reliable export and smaller bundle. |
| **GUI/Plotting** | `plotters` (via WebAssembly/Canvas) | **ECharts** (primary), D3.js (advanced), Material-UI | For generating high-quality, interactive data visualizations and plots within the webview, with consistent UI. **ECharts chosen** for: reliable PNG/SVG export, native timeline animation, 500KB size, and no WebKit issues. **Plotly removed** due to export failures and 3MB bundle size. |
| **Tabs/Monte Carlo Tab (Deferred)** | `ndarray`, `rand` (via WebAssembly) | React, Web Workers | Deferred from current UI scope; intended for efficient N-dimensional simulations and async processing when reintroduced. |
| **Core/Data** | `uom` (Units of Measurement) | TypeScript types | For robust handling of physical quantities with units, ensuring type safety and correctness across the application. |
| **Services/Curve Fitting** | `argmin`, `nalgebra` | React, ECharts | For implementing N-dimensional optimization algorithms for curve fitting and visualizing the fitting results. **Uses ECharts** for consistent plotting. |
| **Core/Symbolic** | `sympy` through **PyO3 0.27.1** | | For symbolic manipulation and representing expressions as Directed Acyclic Graphs (DAGs) for efficient updates. |
| **Compute** | `wgpu` (GPU - planned), `rayon` (CPU) | WebAssembly, Web Workers | For auto-dispatching computations to available hardware (GPU/CPU) and enabling parallel processing for performance-critical tasks. |
| **Persistence/State** | `tauri-plugin-store`, `serde`, `rusqlite` | Zustand (frontend state) | For saving and restoring application state (e.g., open tabs, user preferences), managing complex frontend state, and persistent data storage with SQLite. |
| **Export System** | `rust_xlsxwriter`, `csv`, `arrow`, `parquet`, `serde_json` | TypeScript types | For exporting data in 10 formats: CSV, TSV, TXT, JSON, XLSX, Parquet, HTML, Markdown, LaTeX, AnaFisSpread. All export logic in Rust. Uses Arrow/Parquet (v57.0.0) directly instead of Polars for smaller binary and faster compilation. |
| **Import System** | `arrow`, `parquet`, `encoding_rs`, `flate2` | TypeScript types | For importing data from CSV, TSV, TXT, Parquet, and AnaFisSpread formats. Custom CSV parser with encoding detection (UTF-8, Windows-1252). Direct Parquet reading with type conversion. |
| **Utils** | `log`, `env_logger`, `config` | `zod` (validation) | For structured logging, environment-aware configuration management, and data validation. |

## 5. GUI Sketches

### Solver Tab

```
┌──────────────────────── Solver ────────────────────────┐
│ ┌─Live Preview───────────────┐                         │
│ |∫₀¹ x² dx|                  │                         │
│ └────────────────────────────┘                         |
│ ┌─Inputs──────────────┐                                |
│ │ buttons for easy    │                                |
│ │ writing in the      │                                |
│ │ preview             │                                |
│ └─────────────────────┘                                |
│ ┌─Step Guide (expand)──────┐┌─Plot────────────────────┐│
│ │ 1. Apply power rule …    ||  if possible to         ||
│ │                          ||  visualize              ||
│ │                          ||  the solution           ││
│ └──────────────────────────┘└─────────────────────────┘│
│ [Copy LaTeX] [Copy PNG] [Export PDF]                   │
└────────────────────────────────────────────────────────┘
```

## 6. Design Principles

ANAFIS is built upon the following core design principles:

1.  **Library Reuse**: Prioritize leveraging existing, robust Rust crates and web libraries to accelerate development and ensure reliability.
2.  **Tauri Native**: Fully utilize Tauri's capabilities for seamless native integration, performance optimization, and cross-platform compatibility.
3.  **Functional Programming**: Advocate for pure functions and immutable data structures, especially in the frontend, to enhance predictability and testability.
4.  **Modular Tabs**: Each analysis tool is designed as an independent, self-contained, and closable tab, implemented as a reusable web component.
5.  **Data Bus Communication**: Facilitate inter-tab data sharing and communication via Tauri's IPC mechanisms and efficient web-based state management patterns.
6.  **Detachable Interface**: ⚠️ **Currently Removed** - Tab detaching functionality was temporarily removed for stability improvements. **Planned for re-implementation** in a future release with improved multi-window state synchronization.
7.  **Material Design**: Adhere to Material Design principles for a modern and consistent user interface using Material-UI.

## 7. Core Requirements

ANAFIS must fulfill the following core requirements:

### ✅ **Implemented**
-   **Multi-tab Desktop Application**: ✅ A persistent home menu with the ability to spawn multiple analysis tabs.
-   **Spreadsheet Tool**: ✅ Core functionality including formula evaluation and unit support.
-   **Data Library**: ✅ SQLite-based persistent storage with FTS5 search and statistics.
-   **Import/Export System**: ✅ 10+ format support (CSV, TSV, TXT, JSON, XLSX, Parquet, HTML, Markdown, LaTeX, AnaFisSpread).
-   **Data Visualization**: ✅ ECharts-based plotting with PNG/SVG export capabilities.
-   **Scientific Computing**: ✅ Uncertainty propagation and unit conversion with Rust backends.
-   **File Associations**: ✅ .anafispread files open directly in AnaFis.
-   **Code Quality**: ✅ Biome lint/format baseline active, full TypeScript safety, Clippy compliance.

### 📋 **Planned**
-   **Detachable Tabs**: Tabs must be able to become independent windows (temporarily removed for stability).
-   **N-dimensional Curve Fitting Tool**: Support for multiple algorithms and comprehensive visualization.
-   **Equation Solver**: A Wolfram-like solver with step-by-step solutions.
-   **Monte-Carlo Simulation (Deferred)**: Capabilities for running simulations and analyzing results (currently not exposed in tab creation UI).
-   **Floating Utility Tools**: Small, quick calculation tools (Uncertainty Calculator partially implemented).
-   **Advanced Sidebars**: Statistical analysis, data smoothing, outlier detection, validation.
-   **Inter-tab Communication**: Seamless data exchange between different analysis tabs.
-   **Persistent Application State**: Ability to save and restore the application's state.
-   **Internationalization Support**: Localization for different languages.
-   **GPU Acceleration & Performance Optimization**: Leveraging Rust and WebAssembly for high-performance computations.

## 8. Implementation Plan (Tauri Tasks)

This section outlines the phased implementation plan for the Tauri-based ANAFIS application:

### ✅ **COMPLETED TASKS**
-   [x] 1. Project Setup and Basic Tauri Application Initialization
-   [x] 2. Frontend Framework Integration (React/TypeScript) and initial UI scaffolding
-   [x] 3. Data Bus Communication System (Tauri IPC) establishment
-   [x] 4. Basic Tab Management (Single-window tabbed interface with drag-to-reorder)
-   [x] 5. Spreadsheet Tab Core Functionality (Frontend) development
-   [x] 6. Spreadsheet Advanced Features (Univer.js integration complete)
-   [x] 7. Data Library Infrastructure (SQLite + FTS5 search + statistics + export)
-   [x] 8. Quick Plot Sidebar (ECharts 2D plotting + PNG/SVG export + Data Library integration)
-   [x] 9. Code Quality & Type Safety (Biome baseline, TypeScript strict, Clippy compliant)
-   [x] 10. Export System Implementation (10 formats: CSV, TSV, TXT, JSON, XLSX, Parquet, HTML, Markdown, LaTeX, AnaFisSpread)
-   [x] 11. Export Logic Refactoring (Header handling simplified, explicit data structure markers)
-   [x] 12. Import System Implementation (CSV, TSV, TXT, Parquet, AnaFisSpread with encoding detection)
-   [x] 13. Import Sidebar UI (File import + Data Library import with search/filter)
-   [x] 14. File Association System (.anafispread files open in AnaFis on double-click)
-   [x] 15. Dependency Optimization (Removed Polars, direct Arrow/Parquet usage, PyO3 0.27.1)
-   [x] 16. Uncertainty Propagation Sidebar (Rust backend with formula analysis)
-   [x] 17. Unit Conversion Sidebar (Comprehensive unit database with dimensional analysis)
-   [x] 18. Frontend Linting/Formatting Baseline (Biome pipeline configured and enforced)
-   [x] 19. TypeScript Strict Mode (100% type coverage, no 'any' types, strict null checks)
-   [x] 20. Rust Backend Optimization (Clippy compliant, modern Rust idioms)
-   [x] 21. Statistical Approximations Fixes (Kurtosis formula corrected to use sample variance, skewness test expectations fixed, KS statistic calculation corrected, Burr Type XII PDF missing -ln(lambda) term fixed)

### 📋 **CURRENT STATUS**
**Core Infrastructure**: ✅ COMPLETE
- Data Library with SQLite FTS5 search
- Import/Export system (10 formats)
- Spreadsheet with sidebars (Uncertainty, Unit Conversion, Quick Plot, Import, Export)
- Code quality: Biome + TypeScript strict checks in active use
- Build system: Clean compilation, optimized bundles
- Tab creation UI currently exposes Spreadsheet, Fitting, and Solver (Monte Carlo deferred from menu/toolbar)



**Plugin Architecture**: 🔄 IN PROGRESS
- Univer Plugin for Automatic Uncertainty Propagation (replaces deprecated cell-based approach)
- Correlated uncertainty support with covariance matrices
- Plugin-based extension architecture to work around Univer constraints

### 🔄 **PLANNED TASKS**
-   [ ] 22. Statistical Analysis Sidebar Implementation (Contextual interface with 5 analysis types)
-   [ ] 22. Statistical Analysis Backend (Rust functions for all analysis types)
-   [ ] 23. Weighted Statistics Implementation (χ² analysis, uncertainty propagation)
-   [ ] 24. Hypothesis Testing Implementation (t-tests, normality tests)
-   [ ] 25. Weighted Correlation Analysis (uncertainty-weighted correlation coefficients)
-   [ ] 26. Statistical Analysis UI Polish (Clean labels, logical option ordering)
-   [ ] 27. Statistical Analysis Testing (Contextual interface validation - 5/5 tests passing)
-   [ ] 28. Hypothesis Testing Validation (t-test calculations and result display - 5/5 tests passing)
-   [ ] 29. Weighted Statistics Testing (χ² analysis and uncertainty propagation - 5/5 tests passing)
-   [ ] 30. Statistical Tests Enhancement (ANOVA, Chi-square, non-parametric alternatives)
-   [ ] 31. Advanced Visualization Components (QQ plots, scatter plots, box plots, residual plots - moved to Graphs & Fitting tab)
-   [ ] 32. Shapiro-Wilk Test Robust Implementation (replace approximation with well-tested library)
-   [ ] 33. Weighted Correlation Significance Fix (improve approximation accuracy)
-   [ ] 34. F-test Implementation Fix (proper statistical library integration)
-   [ ] 35. **Univer Plugin for Automatic Uncertainty Propagation** (Plugin architecture to replace deprecated cell-based approach)
-   [ ] 36. **Correlated Uncertainty Support** (Covariance matrix integration for multivariate propagation)
-   [ ] 37. **Basic Plotting Component** (ECharts 2D scatter/line plots with error bars from data library)
-   [ ] 38. **Expression Parser for Custom Functions** (fasteval/meval for user-defined functions like 'a*exp(-x/b) + c')
-   [ ] 39. **Fitting Backend with LM Algorithm** (argmin Levenberg-Marquardt with uncertainty propagation)
-   [ ] 40. **Fitting UI Components** (Data selection, fit functions, initial guesses)
-   [ ] 41. **Plotting-Fitting Integration** (Overlay fit curves, residuals, parameter display)
-   [ ] 42. **Physics-Specific Fit Functions** (Exponential decay, damped oscillation, power laws)
-   [ ] 43. Curve Fitting Tab Foundation (Frontend & Rust integration)
-   [ ] 44. Fitting Algorithms Implementation (levenberg-marquardt, nalgebra)
-   [ ] 45. Advanced Visualization (3D plotting with ECharts-GL) integration
-   [ ] 46. Equation Solver Tab Implementation (Frontend & Rust integration)
-   [ ] 47. Monte Carlo Simulation Tab (Frontend & Rust/WebAssembly integration) **[Deferred from current UI scope]**
-   [ ] 48. Floating Tools Implementation (Uncertainty Calculator, LaTeX Preview)
-   [ ] 49. Data Smoothing Sidebar (moving average, Savitzky-Golay, Gaussian filters)
-   [ ] 50. Outlier Detection Sidebar (Z-score, IQR methods)
-   [ ] 51. Data Validation Sidebar (real-time validation rules)
-   [ ] 52. Metadata Manager Sidebar (experimental context tracking)
-   [ ] 53. Tab Detaching Re-implementation (Multi-window state synchronization)
-   [ ] 54. Internationalization System setup
-   [ ] 55. Application Settings and Configuration management
-   [ ] 56. Update System Implementation
-   [ ] 57. State Persistence and File Management
-   [ ] 58. Comprehensive Testing Suite (Unit, Integration, E2E) development
-   [ ] 59. Distribution and Packaging (Tauri Bundler) setup
-   [ ] 60. Documentation and User Guide creation
-   [ ] 61. GPU Acceleration and Performance Optimization (Rust/WebAssembly) fine-tuning
-   [ ] 62. UI Polish and Accessibility improvements
-   [ ] 63. Final Integration and Release Preparation

## 9. Plan for Tabs (Tauri Edition)

This section details the implementation strategy for browser-style tabs within the Tauri framework:

### 9.1. Core Tab Functionality
-   Utilize web-based drag-and-drop APIs for reordering tabs within a single window.
-   Use @dnd-kit for smooth drag-and-drop interactions with React.
-   Implement a custom React component for the tabs and tab bar to ensure full control over behavior and appearance.
-   Use Zustand for tab state management (active tab, tab order, tab content).

### 9.2. Tab Detaching (Removed for Stability)
⚠️ **Status**: Temporarily removed for stability improvements. Planned for re-implementation.

**Current Implementation**:
-   Single-window tabbed interface with drag-to-reorder functionality
-   Home Tab remains permanently open as application hub
-   Other tabs (Spreadsheet, Fitting, Solver) can be opened/closed dynamically
-   Tab state persisted using `tauri-plugin-store`
-   Optimized tab rendering to prevent unnecessary re-renders

**Removal Reasons**:
-   State synchronization complexity across multiple windows
-   Window lifecycle management (closing detached windows)
-   Data consistency when same spreadsheet open in multiple windows
-   Performance impact of IPC communication overhead

**Future Re-implementation Plan**:
-   Implement robust state synchronization mechanism
-   Add window registry to track all open windows
-   Use event-driven architecture for cross-window updates
-   Implement conflict resolution for concurrent edits
-   Add user preferences for detachment behavior

### 9.3. Current Tab System
-   Single-window tabbed interface with drag-to-reorder functionality
-   Home Tab remains permanently open as application hub
-   Other tabs (Spreadsheet, Fitting, Solver) can be opened/closed dynamically
-   Tab state persisted using `tauri-plugin-store`
-   Optimized tab rendering to prevent unnecessary re-renders

### 9.4. Advanced Features
-   Implement a persistent Home Tab that cannot be closed, serving as the application's central hub.
-   Utilize `tauri-plugin-store` for application-wide state management, persisting user preferences and application settings.
-   Tab lazy-loading: Only render active tab content to improve performance.
-   Tab state caching: Preserve tab state when switching between tabs.
