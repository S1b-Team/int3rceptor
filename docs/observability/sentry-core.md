## Sentry (Core / Rust)

Scope
- Captures: panics and internal errors.
- Drops: request/response payloads, headers, cookies, URLs, noisy network errors (timeouts/reset/refused/unreachable/broken pipe).
- PII: `send_default_pii = false`.
- Tracing: off by default (`traces_sample_rate = 0.0`). Tier 2 can raise it but is capped at `0.05`.

Environment
- `SENTRY_DSN` (required to send)
- `SENTRY_ENVIRONMENT` (default: `development`)
- `SENTRY_RELEASE` (optional; otherwise `int3rceptor-core@<crate_version>[+<GIT_SHA>]`)
- `SENTRY_TELEMETRY_TIER` = `0|1|2` (default `0`; Tier 2 enables low-rate tracing)
- `SENTRY_TRACES_SAMPLE_RATE` (optional; capped at `0.05`, only used in Tier 2)
- `SENTRY_SMOKE_TEST=1` to emit a one-time `"Int3rceptor core smoke test"` on startup.

Behavior
- Panic hook enabled via Sentry `panic` feature; backtraces included (`backtrace` feature).
- Guard is kept alive in `api/src/main.rs` (`let _sentry_guard = ...`), so Sentry stays active for the process lifetime.
- `before_send` strips request fields and sensitive tags/extras.
- Noisy network errors are filtered only when the exception chain looks network-ish to avoid hiding real bugs.

Helper API
- `interceptor_core::telemetry::sentry::capture_anyhow(err, module, tags)`:
  - Skips noisy network IO errors.
  - Adds safe tags (`component=core`, `module`, `tier`, `os`, `arch`, plus caller-supplied safe tags).
  - Does not attach payloads or URLs.

Smoke Test
```
export SENTRY_DSN=...
export SENTRY_ENVIRONMENT=development
export SENTRY_SMOKE_TEST=1
cargo run -p api
unset SENTRY_SMOKE_TEST
```
You should see “Int3rceptor core smoke test” in Sentry; no traffic data is sent.
