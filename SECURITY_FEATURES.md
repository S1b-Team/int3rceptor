# INT3RCEPTOR API Security Features

This document describes the security enhancements added to the INT3RCEPTOR API server.

## Overview

The following security features have been implemented:

1. **HTTPS/TLS Support** - Encrypted communication using TLS
2. **CSRF Protection** - Protection against Cross-Site Request Forgery attacks
3. **IP Allowlist/Blocklist** - IP-based access control
4. **Audit Logging** - Comprehensive logging of sensitive operations

---

## 1. HTTPS/TLS Support

### Configuration

Enable TLS by setting the following environment variables:

```bash
# Path to TLS certificate (PEM format)
export TLS_CERT_PATH="/path/to/certificate.pem"

# Path to TLS private key (PEM format)
export TLS_KEY_PATH="/path/to/private-key.pem"

# Enable HTTP/2 (optional, default: true)
export TLS_HTTP2="true"
```

### How It Works

-   When TLS environment variables are set, the server automatically starts in HTTPS mode
-   Supports HTTP/2 for improved performance
-   Uses `rustls` for TLS implementation (modern, memory-safe TLS library)
-   Falls back to HTTP if no TLS configuration is found

### Example: Generating Self-Signed Certificate (Development Only)

```bash
# Generate self-signed certificate (not recommended for production!)
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/CN=localhost"

# Set environment variables
export TLS_CERT_PATH="./cert.pem"
export TLS_KEY_PATH="./key.pem"
```

**⚠️ Warning**: Self-signed certificates should only be used for development/testing!

### Production Deployment

For production, use certificates from a trusted Certificate Authority (CA) like:

-   Let's Encrypt (free, automated)
-   DigiCert
-   Sectigo
-   Your organization's internal CA

---

## 2. CSRF Protection

### Configuration

Enable CSRF protection with environment variables:

```bash
# Enable CSRF protection
export CSRF_PROTECTION="true"

# Set a secret key (optional, auto-generated if not provided)
export CSRF_SECRET="your-secret-key-here"
```

### How It Works

1. **Token Generation**: Obtain a CSRF token via `GET /api/csrf-token`
2. **Token Validation**: Include the token in the `X-CSRF-Token` header for state-changing requests (POST, PUT, DELETE, PATCH)
3. **Timing-Safe Comparison**: Uses constant-time comparison to prevent timing attacks
4. **Automatic Expiration**: Tokens expire after 1 hour

### API Endpoints

#### Get CSRF Token

```bash
curl https://api.example.com/api/csrf-token \
  -H "Authorization: Bearer YOUR_API_TOKEN"
```

Response:

```json
{
    "csrf_token": "a1b2c3d4e5f6...",
    "header_name": "X-CSRF-Token",
    "expires_in_secs": 3600
}
```

#### Use CSRF Token

```bash
curl -X POST https://api.example.com/api/requests/1/repeat \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "X-CSRF-Token: a1b2c3d4e5f6..." \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

### Best Practices

-   Store CSRF tokens securely (e.g., in session storage, not localStorage)
-   Obtain a new token for each session
-   Use HTTPS to prevent token interception

---

## 3. IP Allowlist/Blocklist

### Configuration

Configure IP filtering via environment variable:

```bash
export IP_FILTER_CONFIG='{
  "mode": "allowlist",
  "allowed_ips": ["192.168.1.100", "10.0.0.50"],
  "blocked_ips": ["1.2.3.4"],
  "allow_private": true,
  "allow_loopback": true
}'
```

### Operating Modes

-   **`off`**: IP filtering disabled (default)
-   **`allowlist`**: Only explicitly allowed IPs can access the API
-   **`blocklist`**: All IPs allowed except those explicitly blocked

### API Endpoints

#### Get IP Filter Configuration

```bash
curl https://api.example.com/api/security/ip-filter \
  -H "Authorization: Bearer YOUR_API_TOKEN"
```

#### Update IP Filter Configuration

```bash
curl -X PUT https://api.example.com/api/security/ip-filter \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "mode": "allowlist",
    "allowed_ips": ["192.168.1.0/24"],
    "blocked_ips": [],
    "allow_private": true,
    "allow_loopback": true
  }'
```

#### Add IP to Allowlist

```bash
curl -X POST https://api.example.com/api/security/ip-filter/allow \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"ip": "192.168.1.100"}'
```

#### Add IP to Blocklist

```bash
curl -X POST https://api.example.com/api/security/ip-filter/block \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"ip": "1.2.3.4"}'
```

### Private IP Ranges

The following IP ranges are considered private:

-   **IPv4**: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `169.254.0.0/16` (link-local)
-   **IPv6**: `fc00::/7` (ULA), `fe80::/10` (link-local)

### Best Practices

-   Use `allowlist` mode in high-security environments
-   Enable `allow_private` for internal deployments
-   Regularly review the IP filter configuration
-   Use CIDR notation for IP ranges (if supported by your configuration)

---

## 4. Audit Logging

### Configuration

Enable audit logging:

```bash
# Path to audit log file
export AUDIT_LOG_PATH="/var/log/interceptor/audit.log"

# Optional: Enable console output for audit events
export AUDIT_LOG_CONSOLE="true"
```

### Logged Events

Audit logs capture:

-   **Authentication Events**: Login attempts, token validation failures
-   **Authorization Events**: Access denied, permission changes
-   **Data Access**: Viewing sensitive data
-   **Data Modification**: Create, update, delete operations
-   **Configuration Changes**: IP filter updates, rule modifications
-   **System Changes**: Server startup/shutdown, log rotation
-   **Security Events**: IP blocks, CSRF failures, rate limiting

### Log Format

Each audit entry is a JSON object:

```json
{
    "timestamp": "2024-12-14T17:30:00Z",
    "severity": "warning",
    "category": "configuration",
    "action": "ip_filter_config_updated",
    "actor": "admin",
    "ip_address": "192.168.1.100",
    "resource": "/api/security/ip-filter",
    "details": {
        "mode": "allowlist",
        "allowed_count": 5,
        "blocked_count": 2
    },
    "success": true,
    "error": null
}
```

### API Endpoints

#### Get Audit Log Info

```bash
curl https://api.example.com/api/security/audit-log/info \
  -H "Authorization: Bearer YOUR_API_TOKEN"
```

Response:

```json
{
    "enabled": true,
    "log_path": "/var/log/interceptor/audit.log"
}
```

#### Rotate Audit Log

```bash
curl -X POST https://api.example.com/api/security/audit-log/rotate \
  -H "Authorization: Bearer YOUR_API_TOKEN"
```

### Severity Levels

-   **info**: Normal operations
-   **warning**: Important changes, potential security issues
-   **critical**: Security violations, system failures

### Best Practices

-   Store audit logs on a separate, secure filesystem
-   Rotate logs regularly to prevent disk space issues
-   Monitor audit logs for suspicious activity
-   Retain logs according to compliance requirements
-   Use log aggregation tools (e.g., ELK stack, Splunk) for analysis

---

## Complete Configuration Example

```bash
# API Server
export API_ADDR="0.0.0.0:8443"
export INTERCEPTOR_API_TOKEN="your-secure-api-token"

# TLS/HTTPS
export TLS_CERT_PATH="/etc/interceptor/certs/server.pem"
export TLS_KEY_PATH="/etc/interceptor/certs/server-key.pem"
export TLS_HTTP2="true"

# CSRF Protection
export CSRF_PROTECTION="true"
export CSRF_SECRET="$(openssl rand -hex 32)"

# IP Filtering
export IP_FILTER_CONFIG='{
  "mode": "allowlist",
  "allowed_ips": ["192.168.1.0/24", "10.0.0.0/8"],
  "blocked_ips": [],
  "allow_private": true,
  "allow_loopback": true
}'

# Audit Logging
export AUDIT_LOG_PATH="/var/log/interceptor/audit.log"
export AUDIT_LOG_CONSOLE="false"

# Rate Limiting (already exists)
# 100 requests per minute per IP

# Body Size Limits
export INTERCEPTOR_MAX_BODY_BYTES=2097152  # 2MB
export INTERCEPTOR_MAX_CONCURRENCY=64
```

---

## Security Checklist

-   [ ] **HTTPS/TLS enabled** with valid certificates
-   [ ] **CSRF protection enabled** for all state-changing operations
-   [ ] **IP filtering configured** (allowlist or blocklist)
-   [ ] **Audit logging enabled** and monitored
-   [ ] **API token set** (`INTERCEPTOR_API_TOKEN`)
-   [ ] **Rate limiting active** (default: 100 req/min per IP)
-   [ ] **Security headers applied** (CSP, X-Frame-Options, etc.)
-   [ ] **Request size limits enforced**
-   [ ] **Logs retained and monitored**
-   [ ] **Regular security audits scheduled**

---

## Testing

### Test TLS Connection

```bash
curl -v https://localhost:8443/api/health \
  -H "Authorization: Bearer YOUR_API_TOKEN"
```

### Test CSRF Protection

```bash
# This should fail (403 Forbidden)
curl -X POST https://localhost:8443/api/rules \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"pattern": "test"}'

# This should succeed
CSRF_TOKEN=$(curl -s https://localhost:8443/api/csrf-token \
  -H "Authorization: Bearer YOUR_API_TOKEN" | jq -r '.csrf_token')

curl -X POST https://localhost:8443/api/rules \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "X-CSRF-Token: $CSRF_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"pattern": "test"}'
```

### Test IP Filtering

```bash
# Block your own IP
curl -X POST https://localhost:8443/api/security/ip-filter/block \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"ip": "YOUR_IP_ADDRESS"}'

# Verify you're blocked (should return 403)
curl https://localhost:8443/api/health \
  -H "Authorization: Bearer YOUR_API_TOKEN"
```

### Check Audit Logs

```bash
# View recent audit events
tail -f /var/log/interceptor/audit.log | jq .

# Search for specific events
grep "ip_filter_config_updated" /var/log/interceptor/audit.log | jq .
```

---

## Troubleshooting

### TLS Certificate Errors

**Error**: `Failed to load TLS certificate and key`

**Solution**:

-   Verify file paths are correct
-   Check file permissions (readable by the API server)
-   Ensure certificate and key are in PEM format
-   Validate certificate: `openssl x509 -in cert.pem -text -noout`

### CSRF Token Validation Failures

**Error**: `CSRF token missing or invalid`

**Solution**:

-   Obtain a fresh token via `/api/csrf-token`
-   Include the `X-CSRF-Token` header in your request
-   Check token hasn't expired (1-hour lifetime)
-   Verify you're using the correct client identifier

### IP Filter Blocking Access

**Error**: `Access denied: IP address not allowed`

**Solution**:

-   Check current configuration: `GET /api/security/ip-filter`
-   Verify your IP is in the allowlist (if in allowlist mode)
-   Ensure you're not in the blocklist
-   Check `allow_loopback` if accessing from localhost
-   Temporarily set mode to `"off"` to regain access

### Audit Log Not Writing

**Error**: Audit events not appearing in log file

**Solution**:

-   Check `AUDIT_LOG_PATH` is set correctly
-   Verify directory exists and is writable
-   Check disk space
-   Review application logs for error messages
-   Enable `AUDIT_LOG_CONSOLE="true"` to see events in stdout

---

## Performance Considerations

-   **TLS Overhead**: HTTPS adds ~5-10% latency; use HTTP/2 to mitigate
-   **CSRF Validation**: Minimal overhead (<1ms per request)
-   **IP Filtering**: Very fast lookup (hash map); negligible overhead
-   **Audit Logging**: Asynchronous writes; minimal impact on request latency
-   **Combined**: All features enabled add ~10-15ms to request latency

---

## Further Reading

-   [OWASP CSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
-   [Mozilla TLS Configuration Guide](https://wiki.mozilla.org/Security/Server_Side_TLS)
-   [NIST Audit Logging Guidelines](https://csrc.nist.gov/publications/detail/sp/800-92/final)
-   [IP Filtering Best Practices](https://www.cloudflare.com/learning/access-management/what-is-an-ip-allow-list/)
