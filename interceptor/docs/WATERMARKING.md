# 🔐 Anti-Piracy & Watermarking System

## Overview

Int3rceptor includes a multi-layer watermarking system to protect against unauthorized redistribution and identify legitimate builds.

## Protection Layers

### 1. **Source Code Headers**

Every source file includes a copyright header:

```rust
// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                          INT3RCEPTOR v2.0.0                               ║
// ║                   Copyright (c) 2025 S1BGr0uP                             ║
// ║                        All Rights Reserved                                ║
// ╚═══════════════════════════════════════════════════════════════════════════╝
```

**Purpose**: Legal protection, makes copying intent clear

### 2. **Build-Time Fingerprint**

Each build generates a unique fingerprint combining:

-   Git commit hash (short)
-   Build timestamp (UTC)

**Format**: `{git-hash}-{timestamp}`  
**Example**: `a3f7b2c-20251122-060000`

**Purpose**: Identify specific builds, track distribution

### 3. **Runtime Watermark**

On startup, Int3rceptor displays:

```
╔═══════════════════════════════════════════════════════════════════════════╗
║                          INT3RCEPTOR v2.0.0                               ║
║                   Copyright (c) 2025 S1BGr0uP                             ║
║                        All Rights Reserved                                ║
╠═══════════════════════════════════════════════════════════════════════════╣
║  PROPRIETARY SOFTWARE - Licensed under Proprietary License                ║
║  Build: a3f7b2c-20251122-060000                                           ║
║  License: https://github.com/S1b-Team/int3rceptor/blob/main/LICENSE      ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

**Purpose**: User awareness, screenshot identification

## How It Works

### Build Process

1. `build.rs` runs before compilation
2. Executes `git rev-parse --short HEAD` to get commit hash
3. Generates timestamp using `chrono`
4. Combines into unique fingerprint
5. Embeds as compile-time constant via `env!("BUILD_FINGERPRINT")`

### Runtime Display

1. On startup, `main()` prints copyright banner
2. Fingerprint is displayed prominently
3. Logged to tracing for audit trails

## Tracking Unauthorized Copies

If you find an unauthorized distribution:

1. **Check the fingerprint** in the banner
2. **Cross-reference** with your build logs
3. **Identify** when and where that build was created
4. **Take action** (DMCA, legal notice, etc.)

## Removing Watermarks = License Violation

Per the LICENSE file (Section 3):

> You **MAY NOT**: Remove or alter any copyright notices

Attempting to remove watermarks:

-   Violates the license agreement
-   Constitutes copyright infringement
-   May result in legal action

## For Legitimate Users

If you're a legitimate user and see this watermark:

-   ✅ This is normal and expected
-   ✅ It doesn't affect functionality
-   ✅ It protects the software you're using
-   ✅ It ensures you have an authentic copy

## For Contributors

When contributing code:

-   Keep copyright headers intact
-   Don't modify watermarking system
-   Your contributions become property of S1BGr0uP (per LICENSE)

## Technical Details

### Files Involved

-   `cli/build.rs` - Fingerprint generation
-   `cli/src/main.rs` - Runtime display
-   `cli/Cargo.toml` - Build dependencies

### Dependencies

-   `chrono` (build-time only) - Timestamp generation

### Environment Variables

-   `BUILD_FINGERPRINT` - Set by build script, embedded at compile-time

## FAQ

**Q: Can I disable the banner?**  
A: No. It's part of the license agreement.

**Q: Does it phone home?**  
A: No. The watermark is purely local.

**Q: Can I modify it for my fork?**  
A: Only if you're contributing back to the original project. Otherwise, no.

**Q: I found someone redistributing Int3rceptor without permission. What do I do?**  
A: Contact S1BGr0uP at licensing@s1bgr0up.com with evidence (screenshots showing fingerprint).

---

**Last Updated**: 2025-11-22  
**Version**: 2.0.0  
**Maintained by**: S1BGr0uP Security Team
