## Sentry Overview (Int3rceptor)

Goals
- Minimal, privacy-first telemetry by default.
- Operator-controlled tiers: 0 (errors only), 1 (errors + safe crumbs), 2 (adds low-rate tracing ≤0.05).
- Never capture PII or traffic contents.

Projects
- `int3rceptor-core` (Rust) — panics/internal errors.
- `int3rceptor-ui` (Tauri WebView) — JS errors only.
- Backend/API shares the core initializer.

Never captured
- HTTP payloads, headers, cookies, auth tokens, full URLs/query strings.
- Session Replay, Metrics (disabled).
- Tracing off unless Tier 2 is explicitly enabled; capped at 0.05.

Common env vars
- DSN: `SENTRY_DSN` (core/api), `VITE_SENTRY_DSN` (ui)
- Environment: `SENTRY_ENVIRONMENT`, `VITE_SENTRY_ENVIRONMENT`
- Release: `SENTRY_RELEASE`, `VITE_SENTRY_RELEASE` (fallbacks to version+sha)
- Tier: `SENTRY_TELEMETRY_TIER`, `VITE_TELEMETRY_TIER` (0/1/2)
- Smoke tests: `SENTRY_SMOKE_TEST=1` (core); UI uses a temporary commented line.

Tags set automatically
- `component` (core/ui/api)
- `tier`
- `os`, `arch`
- Optional module-specific tags via `capture_anyhow` (core).

Noise control
- Filters common network errors (timeout/reset/refused/unreachable/broken pipe).
- Drops XHR/fetch breadcrumbs; Tier 0 drops all breadcrumbs (UI).
- Redacts request URL/headers/cookies/body before send (UI and defensive core hook).

Build & releases
- UI builds with sourcemaps; `@sentry/vite-plugin` uploads when `SENTRY_AUTH_TOKEN`, `SENTRY_ORG`, `SENTRY_PROJECT=int3rceptor-ui` are set.
- Releases use `int3rceptor-<component>@<version>[+<sha>]`.

Smoke test quick refs
- Core: set `SENTRY_SMOKE_TEST=1` and run the backend; look for “Int3rceptor core smoke test”.
- UI: temporarily add `Sentry.captureException(new Error("Sentry UI smoke test"))` after init, reload once, then remove.

DSN hygiene
- Use the `int3rceptor-core` DSN for Rust services and `int3rceptor-ui` DSN for the WebView.
- DSNs live only in env files (`.env.local`, CI secrets). Never commit them.
