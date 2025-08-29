# ANAFIS (Tauri Edition)

ANAFIS is a powerful desktop application designed for scientific data analysis, built with the performance and security of Rust and the flexibility of modern web technologies via Tauri.

## Features

ANAFIS aims to provide a comprehensive suite of tools for scientists, engineers, and researchers, including:

-   **Multi-Tab Interface**: A detachable notebook-style interface where each major capability is its own closable tab.
-   **Spreadsheet Tool**: Advanced spreadsheet functionalities with formula evaluation and unit support.
-   **Curve Fitting**: Robust curve fitting algorithms with interactive visualization.
-   **Wolfram-like Solver**: An intelligent equation solver providing step-by-step solutions.
-   **Monte-Carlo Simulation**: Capabilities for running complex simulations and analyzing results.
-   **Uncertainty Calculator**: A floating utility for quick uncertainty calculations.
-   **GPU Acceleration**: Leveraging Rust and WebAssembly for high-performance computations.

## Technologies Used

-   **Tauri**: For building cross-platform desktop applications using web technologies.
-   **Rust**: For the high-performance backend logic, system interactions, and computation.
-   **React**: For building the dynamic and interactive user interface.
-   **TypeScript**: For type-safe and scalable frontend development.
-   **Material-UI**: For a modern and consistent user interface adhering to Material Design principles.
-   **Web Technologies**: HTML, CSS, and JavaScript for flexible UI design.

## If you want to Contribute

### Prerequisites

Before you begin, ensure you have the following installed:

-   **Rust**: Follow the instructions on [rustup.rs](https://rustup.rs/).
-   **Node.js & npm (or Yarn)**: Download from [nodejs.org](https://nodejs.org/).

### Steps

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/your-repo/anafis-tauri.git # Replace with actual repo URL
    cd anafis-tauri/Code
    ```

2.  **Install frontend dependencies**:
    ```bash
    npm install
    ```
    or
    ```bash
    yarn install
    ```

3.  **Run the application in development mode**:
    ```bash
    npm run tauri dev
    ```
    This will open the ANAFIS application window and enable hot-reloading for development.

4.  **Build the application**:
    ```bash
    npm run tauri build
    ```
    This will compile the application and create an executable in the `src-tauri/target/release` directory (or platform-specific equivalent).

## Usage

Once the application is running, you can interact with its various tabs and tools. The intuitive interface allows you to switch between the Spreadsheet, Curve Fitting, Solver, and Monte-Carlo simulation environments. Detachable tabs provide a flexible workspace for multi-tasking.


## License

This project is licensed under the [LICENSE](LICENSE) file.