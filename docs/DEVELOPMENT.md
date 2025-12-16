# 💻 Development Guide

Welcome to the Interceptor development team! This guide will help you set up your environment and contribute code.

## Prerequisites

-   **Rust**: Latest stable version (`rustup update`).
-   **Node.js**: Version 18 or higher.
-   **Git**: For version control.

## Setup

1. **Clone the Repository**

    ```bash
    git clone https://github.com/S1b-Team/int3rceptor.git
    cd interceptor
    ```

2. **Backend Setup**

    ```bash
    # Install sqlx-cli for database migrations
    cargo install sqlx-cli

    # Create database
    mkdir -p data
    sqlx database create
    sqlx migrate run
    ```

3. **Frontend Setup**
    ```bash
    cd ui
    npm install
    ```

## Running Locally

### Option 1: Run Separately (Recommended for Dev)

**Terminal 1 (Backend):**

```bash
cargo run
```

**Terminal 2 (Frontend):**

```bash
cd ui
npm run dev
```

Access the UI at `http://localhost:5173`.

### Option 2: Run All-in-One

```bash
# Build frontend first
cd ui && npm run build && cd ..

# Run backend (serves frontend static files)
cargo run --release
```

Access the UI at `http://localhost:3000`.

## Testing

-   **Backend**: `cargo test`
-   **Frontend**: `npm run type-check` (Unit tests coming soon)

## Contribution Workflow

1. Create a new branch: `git checkout -b feature/my-feature`
2. Make changes.
3. Run tests to ensure nothing broke.
4. Commit with Conventional Commits: `git commit -m "feat: add new button"`
5. Push and open a Pull Request.
