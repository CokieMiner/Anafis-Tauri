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
├── GEMINI.md
└── Code/
    ├── .eslint.config.js
    ├── package.json
    ├── tsconfig.json
    ├── vite.config.ts
    ├── node_modules/
    ├── public/
    ├── src/                  # Frontend (React/TypeScript) application code
    │   ├── assets/           # Static assets (images, icons)
    │   ├── components/       # Reusable UI components
    │   ├── hooks/            # Custom React hooks
    │   ├── pages/            # Top-level page components for each tab/view
    │   ├── services/         # Frontend services for API calls, data manipulation
    │   ├── store/            # Frontend state management (e.g., Zustand store)
    │   └── App.tsx           # Main React application entry point
    └── src-tauri/            # Rust Backend (Tauri application core)
        ├── Cargo.toml        # Rust project manifest and dependencies
        ├── src/              # Rust source code
        │   ├── commands.rs   # Tauri commands (Rust functions exposed to JavaScript)
        │   ├── main.rs       # Main Rust application entry point
        │   └── models.rs     # Rust data structures and types
        ├── tauri.conf.json   # Tauri configuration file
        └── build.rs          # Rust build script
```

## 4. Library Map (Tauri Edition)

This table outlines the primary libraries and crates intended for use across different modules, justifying their selection:

| Module | Primary Lib (Rust) | Primary Lib (Frontend) | Why |
|---|---|---|---|
| **Shell/Notebook** | `tauri`, `tauri-plugin-window` | React, Material-UI | For building the desktop application shell, managing native windows, and creating a responsive UI for detachable tabs, adhering to Material Design principles. |
| **Tabs/Solver Tab** | `sympy` through PyO3, `nalgebra` | React, MathJax/KaTeX, Plotly.js, Material-UI | To provide symbolic mathematics capabilities, numerical computation for solving equations, live LaTeX rendering, interactive plotting of solutions, and consistent UI. |
| **GUI/Plotting** | `plotters` (via WebAssembly/Canvas) | Plotly.js, D3.js, ECharts, Material-UI | For generating high-quality, interactive data visualizations and plots within the webview, with consistent UI. |
| **Tabs/Monte Carlo Tab** | `ndarray`, `rand` (via WebAssembly) | React, Web Workers | For efficient N-dimensional array operations, random number generation for simulations, and offloading heavy computations to improve UI responsiveness. |
| **Core/Data** | `uom` (Units of Measurement) | TypeScript types | For robust handling of physical quantities with units, ensuring type safety and correctness across the application. |
| **Services/Curve Fitting** | `argmin`, `nalgebra` | React, Plotly.js | For implementing N-dimensional optimization algorithms for curve fitting and visualizing the fitting results. |
| **Core/Symbolic** | `sympy` through PyO3 | | For symbolic manipulation and representing expressions as Directed Acyclic Graphs (DAGs) for efficient updates. |
| **Compute** | `wgpu` (GPU), `rayon` (CPU) | WebAssembly, Web Workers | For auto-dispatching computations to available hardware (GPU/CPU) and enabling parallel processing for performance-critical tasks. |
| **Persistence/State** | `tauri-plugin-store`, `serde` | Zustand/Jotai/Redux (frontend state) | For saving and restoring application state (e.g., open tabs, user preferences) and managing complex frontend state. |
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
6.  **Detachable Interface**: Enable tabs to be torn off into separate, independent windows, providing a flexible and customizable user workspace.
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
-   [x] 4. Basic Tab Management and Detachable Windows implementation
-   [x] 5. Spreadsheet Tab Core Functionality (Frontend) development
-   [ ] 6. Spreadsheet Advanced Features and Testing
-   [ ] 7. Curve Fitting Tab Foundation (Frontend & Rust integration)
-   [ ] 8. Fitting Algorithms Implementation (Rust backend)
-   [ ] 9. Advanced Visualization (Web-based Plotting) integration
-   [ ] 10. Equation Solver Tab Implementation (Frontend & Rust integration)
-   [ ] 11. Monte Carlo Simulation Tab (Frontend & Rust/WebAssembly integration)
-   [ ] 12. Floating Tools Implementation
-   [ ] 13. Internationalization System setup
-   [ ] 14. Application Settings and Configuration management
-   [ ] 15. Update System Implementation
-   [ ] 16. State Persistence and File Management
-   [ ] 17. Comprehensive Testing Suite (Unit, Integration, E2E) development
-   [ ] 18. Distribution and Packaging (Tauri Bundler) setup
-   [ ] 19. Documentation and User Guide creation
-   [ ] 20. GPU Acceleration and Performance Optimization (Rust/WebAssembly) fine-tuning
-   [ ] 21. UI Polish and Accessibility improvements
-   [ ] 22. Final Integration and Release Preparation

## 9. Plan for Tabs (Tauri Edition)

This section details the implementation strategy for browser-style drag-and-drop tabs within the Tauri framework:

### 9.1. Core Drag-and-Drop Tab Functionality
-   Leverage web-based drag-and-drop APIs for reordering tabs within a single window.
-   Utilize Tauri's window management APIs to enable detaching tabs into new, independent windows.
-   Implement a custom React component for the tabs and tab bar to ensure full control over behavior and appearance.
-   Use Tauri's IPC to transfer essential tab information (e.g., ID, state) between different windows during drag-and-drop operations.

### 9.2. Handling Drops and Window Creation
-   Implement comprehensive drag and drop handlers in the frontend to manage the visual feedback and state changes during a drag operation.
-   Use `tauri::api::window::WindowBuilder` in the Rust backend to programmatically create new windows when a tab is detached.
-   Develop a robust system to manage tab state and ensure data consistency across multiple Tauri windows.

### 9.3. Advanced Features and Customization
-   Implement a persistent Home Tab that cannot be closed or detached, serving as the application's central hub.
-   Utilize `tauri-plugin-store` for application-wide state management, persisting user preferences and application settings.
-   Explore implementing cross-instance drag-and-drop using Tauri's IPC and potentially a custom protocol for advanced scenarios.
