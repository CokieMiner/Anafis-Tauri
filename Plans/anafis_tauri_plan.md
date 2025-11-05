# ANAFIS Project Plan (Tauri Edition)

This document outlines the comprehensive plan for the ANAFIS desktop application, adapted for its Tauri implementation. It consolidates design principles, requirements, architectural details, and an implementation roadmap.

## 1. Executive Summary

ANAFIS is envisioned as a **detachable-notebook** desktop application for scientific data analysis. Its core functionality revolves around a tabbed interface where major capabilities—Spreadsheet, Curve-Fitting, Wolfram-like Solver, and Monte-Carlo Simulator—each reside in their own closable and detachable tabs, spawned from a central **Home Menu**. A small, floating Uncertainty Calculator will also be available. Each tab is designed for reusability, with GPU acceleration where beneficial, and communicates via a light **shared data-bus** implemented through Tauri's inter-process communication (IPC).

## 2. Code Guidelines

To ensure code quality and maintainability, the project adheres to the following guidelines and utilizes specific tooling for Rust and web technologies:

### Rust (Backend)
-   **`rustfmt`**: For consistent code formatting.
-   **`clippy`**: For linting and identifying common pitfalls.

### TypeScript/JavaScript (Frontend)
-   **`ESLint`**: For linting and enforcing code style.
-   **`Prettier`**: For automated code formatting.
-   **`TypeScript`**: For static type checking, enhancing code reliability and scalability.

### General Principles
-   **Functional Programming**: Emphasis on pure functions and immutable data structures, particularly in the frontend (React components) and where applicable in Rust.
-   **Immutable State**: Application state is managed through appropriate patterns (e.g., React hooks, Zustand/Jotai/Redux for frontend; immutable data structures in Rust).
-   **Tauri Integration**: Prioritize minimal Rust-specific UI, leveraging Tauri to integrate a rich, web-based functional core.
-   **Library Reuse**: Maximize the use of existing, well-vetted Rust crates and web libraries.

## 3. Repository Layout

The project structure is organized to separate frontend (web) and backend (Rust) concerns within the Tauri framework:

```
Anafis-Tauri/
├── .gitignore
├── LICENSE
├── README.md
└── AnaFis/                   # Main application directory
    ├── eslint.config.js
    ├── package.json
    ├── tsconfig.json
    ├── tsconfig.node.json
    ├── vite.config.ts
    ├── node_modules/
    ├── public/
    ├── src/                  # Frontend (React/TypeScript) application code
    │   ├── components/       # Reusable UI components
    │   │   ├── spreadsheet/  # Spreadsheet-specific components
    │   │   │   ├── sidebar/  # Sidebars (Export, QuickPlot, Uncertainty, UnitConversion)
    │   │   │   └── univer/   # Univer.js integration
    │   │   ├── dataLibrary/  # Data Library components
    │   │   └── ...           # Tab management, toolbar, etc.
    │   ├── hooks/            # Custom React hooks
    │   ├── pages/            # Top-level page components for each tab
    │   │   ├── HomeTab.tsx
    │   │   ├── SpreadsheetTab.tsx
    │   │   ├── FittingTab.tsx (stub)
    │   │   ├── SolverTab.tsx (stub)
    │   │   └── MonteCarloTab.tsx (stub)
    │   ├── themes/           # Theme configuration
    │   ├── types/            # TypeScript type definitions
    │   ├── utils/            # Utility functions
    │   ├── DataLibraryWindow.tsx
    │   ├── LatexPreviewWindow.tsx
    │   ├── SettingsWindow.tsx
    │   ├── UncertaintyCalculatorWindow.tsx
    │   ├── App.tsx           # Main React application entry point
    │   └── main.tsx          # React entry point
    └── src-tauri/            # Rust Backend (Tauri application core)
        ├── Cargo.toml        # Rust project manifest and dependencies
        ├── tauri.conf.json   # Tauri configuration file
        ├── build.rs          # Rust build script
        └── src/              # Rust source code
            ├── main.rs       # Main Rust application entry point
            ├── lib.rs        # Library entry point
            ├── data_library/ # Data Library (SQLite + FTS5)
            ├── export/       # Export system (10 formats)
            ├── scientific/   # Scientific computations
            ├── uncertainty_calculator/
            ├── unit_conversion/
            ├── utils/        # Utility modules
            └── windows/      # Window management
```

## 4. Library Map (Tauri Edition)

This table outlines the primary libraries and crates intended for use across different modules, justifying their selection:

| Module | Primary Lib (Rust) | Primary Lib (Frontend) | Why |
|---|---|---|---|
| **Shell/Notebook** | `tauri`, `tauri-plugin-window` | React, Material-UI | For building the desktop application shell, managing native windows, and creating a responsive UI with consistent Material Design principles. |
| **Tabs/Solver Tab** | `sympy` through PyO3, `nalgebra` | React, MathJax/KaTeX, ECharts, Material-UI | To provide symbolic mathematics capabilities, numerical computation for solving equations, live LaTeX rendering, interactive plotting of solutions, and consistent UI. **Uses ECharts** for reliable export and smaller bundle. |
| **GUI/Plotting** | `plotters` (via WebAssembly/Canvas) | **ECharts** (primary), D3.js (advanced), Material-UI | For generating high-quality, interactive data visualizations and plots within the webview, with consistent UI. **ECharts chosen** for: reliable PNG/SVG export, native timeline animation, 500KB size, and no WebKit issues. **Plotly removed** due to export failures and 3MB bundle size. |
| **Tabs/Monte Carlo Tab** | `ndarray`, `rand` (via WebAssembly) | React, Web Workers | For efficient N-dimensional array operations, random number generation for simulations, and offloading heavy computations to improve UI responsiveness. |
| **Core/Data** | `uom` (Units of Measurement) | TypeScript types | For robust handling of physical quantities with units, ensuring type safety and correctness across the application. |
| **Services/Curve Fitting** | `argmin`, `nalgebra` | React, ECharts | For implementing N-dimensional optimization algorithms for curve fitting and visualizing the fitting results. **Uses ECharts** for consistent plotting. |
| **Core/Symbolic** | `sympy` through PyO3 | | For symbolic manipulation and representing expressions as Directed Acyclic Graphs (DAGs) for efficient updates. |
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

-   **Multi-tab Desktop Application**: A persistent home menu with the ability to spawn multiple analysis tabs.
-   **Detachable Tabs**: Tabs must be able to become independent windows.
-   **Spreadsheet Tool**: Core functionality including formula evaluation and unit support.
-   **N-dimensional Curve Fitting Tool**: Support for multiple algorithms and comprehensive visualization.
-   **Equation Solver**: A Wolfram-like solver with step-by-step solutions.
-   **Monte-Carlo Simulation**: Capabilities for running simulations and analyzing results.
-   **Floating Utility Tools**: Small, quick calculation tools (e.g., Uncertainty Calculator).
-   **Inter-tab Communication**: Seamless data exchange between different analysis tabs.
-   **Persistent Application State**: Ability to save and restore the application's state.
-   **Internationalization Support**: Localization for different languages.
-   **GPU Acceleration & Performance Optimization**: Leveraging Rust and WebAssembly for high-performance computations.

## 8. Implementation Plan (Tauri Tasks)

This section outlines the phased implementation plan for the Tauri-based ANAFIS application:

-   [x] 1. Project Setup and Basic Tauri Application Initialization
-   [x] 2. Frontend Framework Integration (React/TypeScript) and initial UI scaffolding
-   [x] 3. Data Bus Communication System (Tauri IPC) establishment
-   [x] 4. Basic Tab Management (Detachable Windows temporarily removed, planned for re-implementation)
-   [x] 5. Spreadsheet Tab Core Functionality (Frontend) development
-   [x] 6. Spreadsheet Advanced Features (Univer.js integration complete)
-   [x] 7. Data Library Infrastructure (SQLite + FTS5 search + statistics + export)
-   [x] 8. Quick Plot Sidebar (ECharts 2D plotting + PNG/SVG export + Data Library integration)
-   [x] 9. Code Quality & Type Safety (ESLint, TypeScript, Clippy - all errors fixed)
-   [x] 10. Export System Implementation (10 formats: CSV, TSV, TXT, JSON, XLSX, Parquet, HTML, Markdown, LaTeX, AnaFisSpread)
-   [x] 11. Export Logic Refactoring (Header handling simplified, explicit data structure markers)
-   [x] 12. Import System Implementation (CSV, TSV, TXT, Parquet, AnaFisSpread with encoding detection)
-   [x] 13. Import Sidebar UI (File import + Data Library import with search/filter)
-   [x] 14. File Association System (.anafispread files open in AnaFis on double-click)
-   [x] 15. Dependency Optimization (Removed Polars, direct Arrow/Parquet usage, PyO3 0.22.0)
-   [ ] 16. Curve Fitting Tab Foundation (Frontend & Rust integration)
-   [ ] 17. Fitting Algorithms Implementation (Rust backend)
-   [ ] 18. Advanced Visualization (3D plotting with ECharts-GL) integration
-   [ ] 19. Equation Solver Tab Implementation (Frontend & Rust integration)
-   [ ] 20. Monte Carlo Simulation Tab (Frontend & Rust/WebAssembly integration)
-   [ ] 21. Floating Tools Implementation
-   [ ] 22. Statistical Analysis Sidebar (statrs crate + descriptive statistics)
-   [ ] 23. Data Smoothing Sidebar (moving average, Savitzky-Golay, Gaussian filters)
-   [ ] 24. Outlier Detection Sidebar (Z-score, IQR methods)
-   [ ] 25. Data Validation Sidebar (real-time validation rules)
-   [ ] 26. Metadata Manager Sidebar (experimental context tracking)
-   [ ] 27. Tab Detaching Re-implementation (Multi-window state synchronization)
-   [ ] 28. Internationalization System setup
-   [ ] 29. Application Settings and Configuration management
-   [ ] 30. Update System Implementation
-   [ ] 31. State Persistence and File Management
-   [ ] 32. Comprehensive Testing Suite (Unit, Integration, E2E) development
-   [ ] 33. Distribution and Packaging (Tauri Bundler) setup
-   [ ] 34. Documentation and User Guide creation
-   [ ] 35. GPU Acceleration and Performance Optimization (Rust/WebAssembly) fine-tuning
-   [ ] 36. UI Polish and Accessibility improvements
-   [ ] 37. Final Integration and Release Preparation

## 9. Plan for Tabs (Tauri Edition)

This section details the implementation strategy for browser-style tabs within the Tauri framework:

### 9.1. Core Tab Functionality
-   Utilize web-based drag-and-drop APIs for reordering tabs within a single window.
-   Use @dnd-kit for smooth drag-and-drop interactions with React.
-   Implement a custom React component for the tabs and tab bar to ensure full control over behavior and appearance.
-   Use Zustand for tab state management (active tab, tab order, tab content).

### 9.2. Tab Detaching (Planned Feature)
⚠️ **Status**: Temporarily removed for stability improvements. Planned for re-implementation.

**Original Vision**:
-   Enable tabs to be detached into new, independent Tauri windows.
-   Use Tauri's window management APIs (`tauri::api::window::WindowBuilder`).
-   Implement cross-window state synchronization via Tauri IPC.
-   Maintain data consistency across multiple windows.

**Challenges Identified**:
-   State synchronization complexity across multiple windows
-   Window lifecycle management (closing detached windows)
-   Data consistency when same spreadsheet open in multiple windows
-   Performance impact of IPC communication overhead

**Future Implementation Plan**:
-   Implement robust state synchronization mechanism
-   Add window registry to track all open windows
-   Use event-driven architecture for cross-window updates
-   Implement conflict resolution for concurrent edits
-   Add user preferences for detachment behavior

### 9.3. Current Tab System
-   Single-window tabbed interface with drag-to-reorder functionality
-   Home Tab remains permanently open as application hub
-   Other tabs (Spreadsheet, Fitting, Solver, Monte Carlo) can be opened/closed dynamically
-   Tab state persisted using `tauri-plugin-store`
-   Optimized tab rendering to prevent unnecessary re-renders

### 9.4. Advanced Features
-   Implement a persistent Home Tab that cannot be closed, serving as the application's central hub.
-   Utilize `tauri-plugin-store` for application-wide state management, persisting user preferences and application settings.
-   Tab lazy-loading: Only render active tab content to improve performance.
-   Tab state caching: Preserve tab state when switching between tabs.
