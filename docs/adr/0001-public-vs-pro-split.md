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
   rules, NOWARU / VOIDWALKER wiring, and signing-key material and access
   are governed by the private int3rceptor-pro side, covering CI/CD secret
   access and HSM/KMS integration; cryptographic material never touches the
   public tree. Nothing premium is checked into the public tree.

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

## Security & Governance

Two more rules keep the split safe to operate:

4. **Secret management.** Premium signing keys move from hex strings
   stored in a repository to an HSM or a managed KMS. The public
   repository only ever contains the public verifying key. Any key that
   was previously tracked in git history, even in the private repo, is
   treated as compromised long-term and is rotated as part of this split.

5. **CI/CD and seam integrity.** Community PRs must not be able to break
   the Pro extension points:
   - CI runs two pipelines: `default` (public) and `--features pro`, the
     latter on a private runner with access to the Pro workspace.
   - `core/src/intruder/extension.rs` and the feature-gate section of
     `core/Cargo.toml` are under strict CODEOWNERS review.
   - The Pro feature gate fails closed in production builds. Falling back
     to the public impls exists only for development ergonomics and local
     compilation; it is never a licensing path.

## Pro build requirements

Sensitive Pro trait impls must be marked `#[inline(never)]`, so that LTO
or `opt-level = "z"` cannot fuse Pro logic into functions that look public
in the compiled binary. This is a non-functional security requirement, not
a performance tweak. The rule is stated here in the public ADR, but it is
enforced in the int3rceptor-pro tree.

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
- Governance: community contributors stay productive without Pro source,
  and the public tree remains the source of truth for engine behavior.
- Security: moving secrets to a KMS reduces the blast radius of a
  compromise of int3rceptor-pro.
