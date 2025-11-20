# Tasks for Interceptor

## Phase 1 — Core Proxy Engine

-   Setup Rust workspace and crates compile
-   Implement HTTP proxy (Hyper 1.x) without TLS:
    -   Listen on configurable address
    -   Handle CONNECT tunneling (tcp copy_bidirectional)
    -   Forward regular HTTP requests
    -   Capture method, URL, headers, body
-   In-memory capture store with eviction
-   CLI to start proxy and API concurrently
-   Basic API endpoints: list, get, clear
-   WebSocket “request” events

## Phase 2 — TLS MITM and Editor (Complete)

-   [x] Certificate Authority manager (rcgen, cache, export)
-   [x] Dynamic certificate via SNI (Rustls ResolvesServerCert)
-   [x] MITM TLS traffic, capture request/response pairs
-   [x] Repeater endpoint and UI tab

## Phase 3 — Persistence and Search (Complete)

-   [x] SQLite storage with indexing
-   [x] Filter/search by method, host, status, extension
-   [x] Export HAR/JSON/CSV

## Phase 4 — Polish (Complete)

-   [x] Docker images and compose
-   [x] CI: build/test on PR
-   [x] Docs for CA installation per OS
-   [x] Authentication + request limiting + persistence hardening

## Phase 5 — Automation & Rules (In Progress)

-   [x] Rule Engine Core (Match & Replace)
-   [x] API Endpoints for Rules (CRUD)
-   [x] UI Interface for Rules Management
-   [x] Syntax Highlighting for Request/Response Body
-   [x] Scope Management (Include/Exclude Patterns)
-   [x] Intruder/Fuzzer (Sniper, Battering Ram, Pitchfork, Cluster Bomb)
-   [x] Intruder UI (Professional Interface)
-   [x] **Regex Matchers** (Advanced pattern matching in rules)
-   [x] **WebSocket Interception** (Capture WS traffic)
-   [ ] Scripting Support (Lua/Wasm)

## Phase 6 — Future Enhancements

-   [ ] WebSocket Interception
-   [ ] HTTP/2 Support
-   [ ] Collaborative Features (Team Mode)
-   [ ] Plugin System
