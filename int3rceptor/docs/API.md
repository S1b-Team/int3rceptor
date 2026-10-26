# API Reference

Int3rceptor exposes a REST API (and one WebSocket) for the UI and local automation.

**Default base URL**: `http://127.0.0.1:3000`  
(CLI defaults: API `127.0.0.1:3000`, proxy listen `127.0.0.1:8080`.)

Paths below are absolute from that host. Source of truth: `int3rceptor/api/src/routes.rs` and `lib.rs`.

## Authentication and hardening

Local access is the normal case. When configured, the API stack can apply:

- CSRF token at `GET /api/csrf-token` (and CSRF middleware when enabled)
- Optional IP filter middleware
- Optional auth / rate-limit / body-size middleware (see `api` crate)

Remote exposure without those controls is not the intended deployment.

## Live updates

| Method | Path | Notes |
|--------|------|--------|
| GET (WS) | `/ws` | UI live channel |

## Traffic / capture

| Method | Path | Notes |
|--------|------|--------|
| GET | `/api/requests` | List captured requests |
| DELETE | `/api/requests` | Clear capture |
| GET | `/api/requests/:id` | One request |
| POST | `/api/requests/:id/repeat` | Replay a stored request |
| GET | `/api/requests/export` | Export capture |

## Repeater

| Method | Path |
|--------|------|
| POST | `/api/repeater/send` |

## Proxy control

| Method | Path |
|--------|------|
| GET | `/api/proxy/status` |
| POST | `/api/proxy/start` |
| POST | `/api/proxy/stop` |

## Interactive intercept

| Method | Path |
|--------|------|
| GET | `/api/intercept` |
| PUT | `/api/intercept` |
| POST | `/api/intercept/:id/forward` |
| POST | `/api/intercept/:id/drop` |
| POST | `/api/intercept/:id/edit` |

## Rules and scope

| Method | Path |
|--------|------|
| GET | `/api/rules` |
| POST | `/api/rules` |
| DELETE | `/api/rules` |
| GET | `/api/scope` |
| PUT | `/api/scope` |

## Settings, CA, plugins, license

| Method | Path |
|--------|------|
| GET | `/api/settings` |
| PUT | `/api/settings` |
| GET | `/api/ca-cert` |
| GET | `/api/plugins` |
| POST | `/api/plugins/upload` |
| POST | `/api/plugins/:name/toggle` |
| GET | `/api/license` |

## Intruder

| Method | Path |
|--------|------|
| POST | `/api/intruder/generate` |
| GET | `/api/intruder/results` |
| DELETE | `/api/intruder/results` |
| POST | `/api/intruder/start` |
| POST | `/api/intruder/stop` |
| GET | `/api/intruder/status` |
| GET | `/api/intruder/payloads` |

## Scanner

| Method | Path |
|--------|------|
| GET | `/api/scanner/config` |
| PUT | `/api/scanner/config` |
| GET | `/api/scanner/rules` |
| POST | `/api/scanner/start` |
| POST | `/api/scanner/stop` |
| GET | `/api/scanner/findings` |
| DELETE | `/api/scanner/findings` |
| GET | `/api/scanner/stats` |

## Encoding and comparer

| Method | Path |
|--------|------|
| POST | `/api/encoding/transform` |
| POST | `/api/comparer/diff` |

## WebSocket capture (HTTP API)

| Method | Path |
|--------|------|
| GET | `/api/websocket/connections` |
| GET | `/api/websocket/frames/:connection_id` |
| GET | `/api/websocket/stats` |
| DELETE | `/api/websocket/clear` |

## Project

| Method | Path |
|--------|------|
| POST | `/api/project/save` |
| POST | `/api/project/load` |
| GET | `/api/project/info` |
| PUT | `/api/project/info` |
| POST | `/api/project/new` |

## Metrics, health, dashboard

| Method | Path |
|--------|------|
| GET | `/api/metrics` |
| POST | `/api/metrics/reset` |
| GET | `/api/health` |
| GET | `/api/dashboard/activity` |
| DELETE | `/api/dashboard/activity/clear` |

## Security management

| Method | Path |
|--------|------|
| GET | `/api/security/ip-filter` |
| PUT | `/api/security/ip-filter` |
| POST | `/api/security/ip-filter/allow` |
| POST | `/api/security/ip-filter/block` |
| GET | `/api/security/audit-log/info` |
| POST | `/api/security/audit-log/rotate` |

## Note on older docs

Earlier drafts referred to `/traffic` and unprefixed `/rules` / `/scope`. Those paths are not registered; use the `/api/...` table above.
