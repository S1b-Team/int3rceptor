# System Architecture

Int3rceptor splits a Rust proxy/API backend from Vue frontends. Premium modules live in the private `int3rceptor-pro` workspace (see ADR 0001).

## High-level flow

```mermaid
graph TD
    Client[Client / browser under test] -->|HTTP(S) via proxy| Proxy[interceptor-core proxy]
    Proxy -->|MITM TLS| CA[Certificate manager]
    Proxy -->|capture / rules / scope| Core[Core engines]
    Core -->|SQLite| Store[(rusqlite storage)]
    UI[Vue UI / Tauri desktop] -->|REST /api/*| API[interceptor-api Axum]
    UI -->|WebSocket /ws| API
    API --> Core
    Proxy -->|forward| Upstream[Upstream servers]
```

## Crates (public tree)

| Crate | Role |
|-------|------|
| `interceptor-core` | Proxy, TLS/CA, capture, rules, scope, Intruder, scanner, WebSocket proxy, plugins (WASM), metrics |
| `interceptor-api` | Axum REST + `/ws`, CSRF/IP-filter hooks, UI-facing routes |
| `interceptor-cli` | Process entrypoint; default API `127.0.0.1:3000`, proxy `127.0.0.1:8080`; build fingerprint watermark |

Workspace members are under `int3rceptor/{core,api,cli}` (root `Cargo.toml` points there).

## Backend stack

- **HTTP server**: Axum on Hyper
- **Proxy / TLS**: Hyper + rustls / tokio-rustls, dynamic certs via `rcgen`
- **State**: shared `AppState` behind `Arc` (parking_lot / DashMap where needed)
- **Persistence**: SQLite through **rusqlite** (bundled), not SQLx
- **Plugins**: Wasmtime-loaded WASM plugins
- **Observability**: `tracing`; optional Sentry in core

## Frontends

There are two Vue 3 + Vite surfaces today:

- **`int3rceptor/desktop/`** — Tauri desktop shell (Views: Traffic, Intruder, Repeater, Scanner, WebSocket, Dashboard, etc.)
- **`int3rceptor/ui/`** — lighter web UI (Tabs)

Both talk to the same API/WebSocket. Treat `desktop/` as the fuller product UI unless a task says otherwise.

## Key subsystems (core)

- **Certificate Authority** — dynamic MITM certificates
- **Rule engine** — match/replace style policies
- **Scope manager** — include/exclude filters
- **Intruder** — public attack modes; Pro extends via Cargo feature `pro` (ADR 0001)
- **Scanner** — rule-driven findings
- **WebSocket proxy / capture** — frames and connection stats exposed on `/api/websocket/*`
- **Interactive intercept queue** — forward / drop / edit
- **License manager** — tier reporting on `/api/license` (paid validation is a Pro concern)

## Public vs Pro

Public keeps a working Intruder, API, and UI. Premium payload packs, strategies, NOWARU/VOIDWALKER wiring, and signing material stay in `int3rceptor-pro`. See `docs/adr/0001-public-vs-pro-split.md`.
