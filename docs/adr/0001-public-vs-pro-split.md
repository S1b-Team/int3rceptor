# ADR 0001: Public vs Pro Split

## Status

Accepted

## Context

int3rceptor ships a capable open Intruder (Sniper, Battering, Pitchfork,
ClusterBomb), plus the API, UI, and live stats that operators rely on every
day. Separately, we want room for premium payload packs, attack strategies,
and commercial integrations without gutting the public engine or forcing a
fork of the whole product.

## Decision

Three rules govern the split:

1. **Public keeps the product.** Basic Intruder, the UI, the API, and
   WebSocket stats stay in this repository and keep working on a normal
   build. We do not empty the public Intruder to make room for Pro.

2. **Premium stays private.** Premium payloads, strategies, and detection
   rules, NOWARU / VOIDWALKER wiring, and the signing key live only in the
   int3rceptor-pro repository. Nothing premium is checked into the public
   tree.

3. **Next work fills Pro, then cleans licenses.** Follow-up is license
   cleanup and filling the Pro placeholders — not stripping features from
   public. The public Intruder remains the default engine.

## Extension mechanism

Intruder exposes a small trait seam in `interceptor-core`:

- `PayloadSource` — supplies payload lists for positions
- `AttackStrategy` — expands a template + config into concrete requests

The public defaults are `CommonPayloadSource` (same values as today's
`common_payloads()`) and `PublicAttackStrategy` (the four built-in modes).
`resolve_payload_source()` and `resolve_attack_strategy()` pick the active
impl. Under the Cargo feature `pro`, those resolvers call hooks in
`intruder::extension::pro_extension`, which int3rceptor-pro wires to premium
modules. Without that feature, behavior is unchanged.

## Consequences

- A normal `cargo build` / `cargo test` in this repo stays green and
  behaves as before.
- The Intruder seam under `pro` falls back to the public impls in this
  tree so the hooks type-check without premium content. The full
  `--features pro` crate build still expects the private `core-pro`
  path used by the scanner rules; that build belongs in the
  int3rceptor-pro workspace.
- API routes that serve built-in payloads go through the resolver so Pro
  can swap sources without touching the route layer.
