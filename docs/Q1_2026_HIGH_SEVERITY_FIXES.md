# Q1 2026: HIGH Severity Security Fixes

## Overview
Implementation plan for HIGH severity issues identified in Vectal security analysis.

**Timeline:** 2-3 weeks  
**Priority:** P2-P5 (after CRITICAL fixes)

---

## P2: API Authentication Hardening

### Current Issue
- Authentication is **OPTIONAL** - only validated if `INTERCEPTOR_API_TOKEN` env var is set
- Production deployments without token are completely unprotected
- No distinction between dev and production modes

### Fix: Mandatory Authentication with Dev Mode
**Estimated Time:** 1 day

#### Changes
1. **main.rs**
   - Check if running in production or dev mode
   - In production: API token is REQUIRED
   - In dev: Allow optional token via `INTERCEPTOR_DEV_MODE=1` env var

2. **auth.rs**
   - Update `require_auth` to enforce token when in production
   - Add detailed error logging for failed auth attempts

3. **Audit Trail**
   - Log all authentication failures
   - Track API usage by token

#### Code Changes Needed
```rust
// main.rs
let dev_mode = std::env::var("INTERCEPTOR_DEV_MODE")
    .map(|v| v == "1" || v.to_lowercase() == "true")
    .unwrap_or(false);

let api_token = if dev_mode {
    std::env::var("INTERCEPTOR_API_TOKEN").ok().map(Arc::new)
} else {
    // Production: token is required
    Some(Arc::new(
        std::env::var("INTERCEPTOR_API_TOKEN")
            .expect("INTERCEPTOR_API_TOKEN is required in production mode")
    ))
};
```

---

## P3: CSRF Protection Hardening

### Current Issues
1. **Optional by Default** - CSRF protection can be disabled
2. **IP-Based Client ID** - Uses `X-Forwarded-For` header which can be spoofed
3. **No Session Binding** - Tokens don't bind to user session

### Fix: Always-On CSRF with Session Binding
**Estimated Time:** 1-2 days

#### Changes
1. **csrf.rs**
   - Make CSRF protection mandatory in production
   - Replace IP-based client_id with session-based binding
   - Use HMAC-SHA256 with session secret

2. **Session Management**
   - Generate unique session ID on first request
   - Store session ID in secure cookie (HttpOnly, Secure, SameSite)
   - Bind CSRF tokens to specific sessions

3. **Token Validation**
   - Validate CSRF token + session binding
   - Prevent cross-session token reuse

#### Code Changes Needed
```rust
// csrf.rs - Use session instead of IP
pub fn validate_token(&self, session_id: &str, provided_token: &str) -> bool {
    // session_id comes from secure cookie, not IP header
    // This prevents spoofing via X-Forwarded-For
}

// Secure cookie headers
Set-Cookie: session_id=abc123; HttpOnly; Secure; SameSite=Strict
```

---

## P4: Memory Bounds (WebSocket & Intruder)

### Current Issues
1. **Unbounded WebSocket Frame Storage** - No limit on accumulated frames
2. **Unbounded Intruder Results** - Can exhaust memory with many payloads
3. **No Memory Monitoring** - No alerts when usage is high

### Fix: Add Memory Limits and Monitoring
**Estimated Time:** 1-2 days

#### Changes
1. **websocket.rs**
   - Add configurable max frames per session
   - Implement FIFO eviction when limit reached
   - Track memory usage per session

2. **intruder.rs**
   - Add max results limit (default: 10,000)
   - Track request count and memory
   - Return error when limit exceeded

3. **Memory Monitoring**
   - Add telemetry for memory usage
   - Send alerts if usage > 80% of limit

#### Configuration
```env
INTERCEPTOR_WS_MAX_FRAMES=1000        # Per WebSocket session
INTERCEPTOR_INTRUDER_MAX_RESULTS=10000 # Max payloads tested
INTERCEPTOR_MEMORY_LIMIT_MB=512        # Total process limit
INTERCEPTOR_MEMORY_WARN_THRESHOLD=80   # Alert at % usage
```

---

## P5: Plugin Security

### Current Issues
1. **Path Traversal** - Plugins can load from arbitrary paths
2. **No Resource Limits** - Plugins can consume unlimited CPU/memory
3. **No Sandboxing** - WASM plugins run with full module access

### Fix: Plugin Security Hardening
**Estimated Time:** 1 day

#### Changes
1. **Plugin Loading**
   - Whitelist allowed plugin directories
   - Reject paths with `..` or symbolic links
   - Validate plugin signatures (optional)

2. **Resource Limits**
   - Set memory limits per plugin (WASM)
   - Set CPU time limits (instructions)
   - Kill runaway plugins

3. **Capability-Based Security**
   - Plugins only get required capabilities
   - No filesystem access by default
   - No network access unless explicitly granted

#### Code Changes Needed
```rust
// plugin/loader.rs
const ALLOWED_PLUGIN_DIRS: &[&str] = &[
    "plugins/",
    "plugins/official/",
];

fn validate_plugin_path(path: &Path) -> Result<()> {
    // Reject .. and symlinks
    // Verify directory is whitelisted
}
```

---

## Implementation Order

1. **Week 1**
   - API Authentication Hardening (P2)
   - CSRF Protection Hardening (P3)

2. **Week 2**
   - Memory Bounds (P4)
   - Plugin Security (P5)

3. **Week 3**
   - Testing and integration
   - Documentation updates

---

## Testing Checklist

- [ ] API auth fails when token missing in production
- [ ] API auth passes when token present
- [ ] CSRF tokens bind to sessions
- [ ] CSRF tokens expire correctly
- [ ] WebSocket respects frame limits
- [ ] Intruder respects result limits
- [ ] Memory monitoring alerts work
- [ ] Plugin path traversal rejected
- [ ] Plugin resource limits enforced

---

## Documentation Updates

- [ ] Update API documentation with auth requirements
- [ ] Add environment variable reference
- [ ] Create session management guide
- [ ] Document plugin security model
- [ ] Add memory tuning guide

---

## Rollout Plan

**Phase 1: Internal Testing**
- Enable in dev environment
- Run full test suite

**Phase 2: Beta Release**
- Optional in production (default disabled)
- Allow opt-in for security-conscious users

**Phase 3: Full Production Release**
- Mandatory in production
- Dev mode available with `INTERCEPTOR_DEV_MODE=1`

