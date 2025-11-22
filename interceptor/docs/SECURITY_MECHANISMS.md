# 🛡️ Security & Protection Mechanisms

Int3rceptor employs enterprise-grade security measures to protect its intellectual property and ensure license compliance.

## 1. Anti-Tamper & Obfuscation

### Binary Hardening

-   **Symbol Stripping**: All debug symbols are stripped from release binaries.
-   **Panic Abort**: Panic unwinding logic is removed to prevent leaking internal paths and structures.
-   **LTO (Link Time Optimization)**: Aggressive cross-crate inlining makes control flow analysis significantly harder.

### String Encryption

Sensitive strings (error messages, keys, internal logic) are encrypted at compile-time using XOR obfuscation (`obfstr`).

-   **Benefit**: Strings are not visible in `strings` or hex editors.
-   **Benefit**: Static analysis tools cannot easily find critical validation logic.

## 2. License System

### Architecture

-   **Offline Validation**: No "phone home" requirement.
-   **Digital Signatures**: Licenses are cryptographically signed to prevent forgery.
-   **Hardware Binding**: Enterprise licenses can be locked to specific hardware fingerprints.

### License Tiers

| Tier             | Max Connections | Max RPS   | Advanced Features |
| ---------------- | --------------- | --------- | ----------------- |
| **Free**         | 10              | 100       | ❌                |
| **Professional** | 100             | 1,000     | ✅                |
| **Enterprise**   | Unlimited       | Unlimited | ✅                |

### Validation Logic

1. **Format Check**: Base64 decoding and JSON parsing.
2. **Signature Verification**: Verifies the license was issued by S1BGr0uP.
3. **Expiration Check**: Validates the `expires_at` timestamp.
4. **Hardware Check**: (Optional) Verifies CPU/Machine ID matches the license.

## 3. Watermarking

### Build Fingerprinting

Every build includes a unique, immutable fingerprint:

-   **Git Hash**: Exact commit of the source code.
-   **Timestamp**: Precise build time (UTC).
-   **Embedded**: Hardcoded into the binary at compile time.

### Runtime Display

The application displays copyright and license information on every startup, creating an audit trail in logs.

## 4. Anti-Debugging (Planned)

Future updates will include:

-   `ptrace` detection (Linux)
-   Debugger attachment checks
-   Timing checks to detect single-stepping

---

**CONFIDENTIAL**: This document describes internal security mechanisms. Do not distribute.
