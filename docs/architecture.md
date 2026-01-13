# S1b Ecosystem Architecture

Cross-cutting design decisions and integration contracts for the S1b security platform.

## Module Integration Overview

```
                                    ┌─────────────────┐
                                    │     EDICT       │
                                    │  (Trust Anchor) │
                                    └────────┬────────┘
                                             │
                    ┌────────────────────────┼────────────────────────┐
                    │ TLS Certs              │ Control Certs          │
                    ▼                        ▼                        ▼
           ┌─────────────────┐      ┌─────────────────┐      ┌─────────────────┐
           │   INT3RCEPTOR   │      │    MYRMIDON     │      │     NOWARU      │
           │  (Interception) │◄────►│ (Orchestration) │◄────►│   (Analytics)   │
           └────────┬────────┘      └─────────────────┘      └─────────────────┘
                    │
                    │ Payloads
                    ▼
           ┌─────────────────┐
           │   VOIDWALKER    │
           │   (Payloads)    │
           └─────────────────┘
```

## Integration Contracts

### 1. EDICT → All Modules (Trust Distribution)

EDICT serves as the Private PKI and trust anchor for the entire ecosystem.

| Consumer | Certificate Type | Purpose |
|----------|-----------------|---------|
| INT3RCEPTOR | Interception CA | Dynamic MITM certificate signing |
| MYRMIDON | Control Channel | Authenticated orchestration APIs |
| NOWARU | Ingest Channel | Secure telemetry ingestion |
| VOIDWALKER | Signing Certs | Payload provenance verification |

**Contract:**
```rust
// Certificate request from any module
pub struct CertificateRequest {
    pub subject: String,
    pub key_usage: KeyUsage,
    pub validity_days: u32,
    pub san: Vec<SubjectAltName>,
}

// EDICT response
pub struct IssuedCertificate {
    pub cert_pem: String,
    pub chain_pem: String,
    pub serial: String,
    pub not_after: DateTime<Utc>,
}
```

### 2. INT3RCEPTOR → NOWARU (Event Streaming)

INT3RCEPTOR emits structured events for every intercepted request/response.

**Event Schema:**
```rust
pub struct TrafficEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub request: HttpRequest,
    pub response: Option<HttpResponse>,
    pub timing: TimingInfo,
    pub tags: Vec<String>,
    pub matched_rules: Vec<RuleMatch>,
}
```

**Transport:** WebSocket stream at `ws://nowaru:9001/ingest`

**Flow:**
1. INT3RCEPTOR captures request/response pair
2. Enriches with timing, rule matches, scope info
3. Serializes to JSON and pushes to NOWARU
4. NOWARU acknowledges receipt with event ID

### 3. MYRMIDON → INT3RCEPTOR (Orchestration)

MYRMIDON coordinates INT3RCEPTOR nodes for distributed operations.

**Command Types:**
```rust
pub enum OrchestratorCommand {
    UpdateScope(ScopeConfig),
    DeployRules(Vec<Rule>),
    StartIntruder(IntruderJob),
    PauseInterception,
    ResumeInterception,
    RotateCertificates,
}
```

**Transport:** gRPC over mTLS (certificates from EDICT)

**Acknowledgment:** Each command returns execution status and affected resources.

### 4. NOWARU → MYRMIDON (Intelligence Feedback)

NOWARU provides analytics signals for adaptive orchestration.

**Signal Types:**
```rust
pub struct IntelligenceSignal {
    pub signal_type: SignalType,
    pub confidence: f32,
    pub source_events: Vec<Uuid>,
    pub recommendation: Option<Action>,
}

pub enum SignalType {
    AnomalyDetected,
    RateLimitTriggered,
    AuthenticationPattern,
    ErrorSpike,
    SuccessPattern,
}
```

### 5. VOIDWALKER → INT3RCEPTOR (Payload Delivery)

VOIDWALKER provides transformed payloads for injection into traffic flows.

**Payload Package:**
```rust
pub struct PayloadPackage {
    pub id: Uuid,
    pub payload: Vec<u8>,
    pub encoding: PayloadEncoding,
    pub opsec_profile: OpsecProfile,
    pub injection_points: Vec<InjectionPoint>,
}
```

## Shared Data Models

### HTTP Primitives

All modules share common HTTP data structures:

```rust
pub struct HttpRequest {
    pub method: Method,
    pub url: Url,
    pub headers: HeaderMap,
    pub body: Option<Body>,
}

pub struct HttpResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Option<Body>,
}
```

### Scope Definition

Unified scope format across INT3RCEPTOR and MYRMIDON:

```rust
pub struct ScopeConfig {
    pub include: Vec<ScopePattern>,
    pub exclude: Vec<ScopePattern>,
}

pub struct ScopePattern {
    pub protocol: Option<Protocol>,
    pub host: HostPattern,
    pub port: Option<u16>,
    pub path: Option<PathPattern>,
}
```

## Communication Protocols

| Channel | Protocol | Auth | Purpose |
|---------|----------|------|---------|
| INT3RCEPTOR ↔ NOWARU | WebSocket | mTLS | Real-time event streaming |
| MYRMIDON ↔ INT3RCEPTOR | gRPC | mTLS | Command & control |
| MYRMIDON ↔ NOWARU | gRPC | mTLS | Intelligence exchange |
| EDICT ↔ All | REST/gRPC | mTLS | Certificate operations |
| VOIDWALKER ↔ INT3RCEPTOR | REST | mTLS | Payload retrieval |

## Security Boundaries

### Trust Hierarchy

```
EDICT Root CA (Air-gapped)
├── Interception Intermediate CA
│   └── Per-target MITM certificates (INT3RCEPTOR)
├── Control Intermediate CA
│   ├── MYRMIDON server cert
│   ├── INT3RCEPTOR agent certs
│   └── NOWARU ingest cert
└── Signing Intermediate CA
    └── VOIDWALKER payload signatures
```

### Authentication Requirements

| Operation | Auth Method | Required Claims |
|-----------|-------------|-----------------|
| Deploy rules | mTLS + JWT | `scope:rules:write` |
| Start intruder | mTLS + JWT | `scope:intruder:execute` |
| View traffic | mTLS + JWT | `scope:traffic:read` |
| Issue certs | mTLS + JWT | `scope:pki:issue` |

## Deployment Topologies

### Single-Node (Development)

All modules run on single host, communicating via localhost.

```
┌─────────────────────────────────────────────┐
│                 Single Host                  │
│  ┌─────────┐ ┌─────────┐ ┌────────────────┐ │
│  │INT3RCPT │ │MYRMIDON │ │ NOWARU+EDICT   │ │
│  │ :8080   │ │ :9000   │ │ :9001 + :9002  │ │
│  └─────────┘ └─────────┘ └────────────────┘ │
└─────────────────────────────────────────────┘
```

### Distributed (Production)

Modules deployed across multiple hosts with network segmentation.

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Edge Node  │     │ Control Node │     │ Analytics    │
│ INT3RCEPTOR  │◄───►│   MYRMIDON   │◄───►│   NOWARU     │
│   VOIDWLKR   │     │    EDICT     │     │              │
└──────────────┘     └──────────────┘     └──────────────┘
```

## Error Handling

### Circuit Breaker Pattern

All inter-module communication implements circuit breakers:

- **Closed:** Normal operation, requests flow through
- **Open:** Failures exceeded threshold, requests fail fast
- **Half-Open:** Testing if service recovered

### Retry Policy

```rust
pub struct RetryPolicy {
    pub max_attempts: u32,      // Default: 3
    pub base_delay_ms: u64,     // Default: 100
    pub max_delay_ms: u64,      // Default: 5000
    pub exponential_base: f32,  // Default: 2.0
}
```

### Fallback Behavior

| Scenario | Fallback |
|----------|----------|
| EDICT unreachable | Use cached certificates, log warning |
| NOWARU unreachable | Buffer events locally, replay when available |
| MYRMIDON unreachable | Continue with last known config |
| VOIDWALKER unreachable | Skip payload injection, pass traffic through |

## Observability

### Metrics (Prometheus)

All modules expose metrics at `/metrics`:

- `s1b_requests_total{module, status}`
- `s1b_request_duration_seconds{module, operation}`
- `s1b_active_connections{module}`
- `s1b_certificate_expiry_seconds{subject}`

### Tracing (OpenTelemetry)

Distributed tracing with correlation IDs across all modules:

```
X-S1b-Trace-Id: <uuid>
X-S1b-Span-Id: <uuid>
X-S1b-Parent-Span-Id: <uuid>
```

### Logging

Structured JSON logging with consistent fields:

```json
{
  "timestamp": "2025-01-01T00:00:00Z",
  "level": "info",
  "module": "int3rceptor",
  "trace_id": "...",
  "message": "Request intercepted",
  "fields": {}
}
```