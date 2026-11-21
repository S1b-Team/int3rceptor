# INT3RCEPTOR Status and Roadmap

This is the single living status document. Reports under `docs/archive/` are historical and may claim work that was never finished.

**Target v1 (3.0.0):** desktop app (Linux, macOS, Windows) with the current core, plus a Pro tier unlocked by an Ed25519 license. EDICT, MYRMIDON, NOWARU and VOIDWALKER are out of v1.

**Milestones:** Alpha = F0+F1+F2 · Beta = F3+F4 · RC = F5 · GA = F6

| Phase | Goal | Status |
|-------|------|--------|
| F0 | CI at the repo root, honest docs, one version, public-core sync | In progress |
| F1 | Backend gaps: dashboard metrics, secure-by-default API, HAR import, scope excludes | Pending |
| F2 | `desktop/` replaces `ui/` (stores, single API client, router) | Pending |
| F3 | Design system and screen parity with `docs/mockups` | Pending |
| F4 | Tauri: proxy sidecar, CA onboarding, CSP, updater, signing, releases | Pending |
| F5 | Pro tier: license enforcement in API and UI, activation, private build | Pending |
| F6 | Honest pricing and site, checkout, EULA, user docs, beta, GA | Pending |

## Known gaps today

- `desktop/` does not build (157 TypeScript errors, one invalid attribute in `IntruderView.vue`).
- `desktop/` calls `/api/stats`, which does not exist. Use `/api/dashboard/metrics` (added in F1).
- API auth is off unless `INTERCEPTOR_API_TOKEN` is set, and CORS is permissive.
- No HAR import. Loading a project drops scope excludes.
- Tauri does not start the proxy. No updater, signing or CSP.
- The license is not enforced, and the public build embeds a placeholder key.

## Version

Core, API, CLI and the desktop app use `3.0.0`. Bump them together. `examples/simple-plugin` is a sample and keeps its own version.
