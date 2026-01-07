## Sentry (UI / Tauri WebView)

What we capture
- JavaScript errors only. No Session Replay, no Metrics.
- Tiered telemetry:
  - Tier 0 (default): errors only, no breadcrumbs, no tracing.
  - Tier 1: errors + safe breadcrumbs (XHR/fetch dropped).
  - Tier 2: Tier 1 + tracing at low rate (capped at `0.05`).

What we never capture
- Request URLs, headers, cookies, bodies/payloads.
- PII (`sendDefaultPii=false`).
- Network breadcrumbs (XHR/fetch) are dropped; Tier 0 drops all breadcrumbs.

Env vars (UI build/runtime)
- `VITE_SENTRY_DSN`
- `VITE_SENTRY_ENVIRONMENT` (fallback: Vite `MODE`)
- `VITE_SENTRY_RELEASE` (fallback: `VITE_APP_VERSION` or `dev`)
- `VITE_APP_VERSION`
- `VITE_TELEMETRY_TIER` = `0|1|2` (default `0`)
- `VITE_SENTRY_TRACES_SAMPLE_RATE` (only used when Tier 2; capped at `0.05`)
- Sourcemap upload (build/CI): `SENTRY_AUTH_TOKEN`, `SENTRY_ORG`, `SENTRY_PROJECT=int3rceptor-ui`

Implementation notes
- Init in `ui/src/telemetry/sentry.ts`; wired in `src/main.ts`.
- Redaction in `beforeSend`: URL -> `[REDACTED_URL]`, headers/cookies replaced, data removed, sensitive extras stripped.
- Breadcrumb filter: drops XHR/fetch; Tier 0 drops all breadcrumbs.
- Tags set: `component=ui`, `tier`, `os`.

Sourcemaps
- `vite.config.ts` enables sourcemaps and conditionally loads `@sentry/vite-plugin` when `SENTRY_AUTH_TOKEN`, `SENTRY_ORG`, `SENTRY_PROJECT` are present.
- Release used for upload: `VITE_SENTRY_RELEASE` or `VITE_APP_VERSION`.

Smoke test (manual, dev only)
```
export VITE_SENTRY_DSN=...
export VITE_TELEMETRY_TIER=0
npm run dev
# temporarily uncomment or add Sentry.captureException(new Error("Sentry UI smoke test"));
# reload once, then remove the line.
```
