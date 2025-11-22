# Int3rceptor API Documentation

## Overview

Int3rceptor provides a RESTful API that allows you to programmatically control the proxy, access captured traffic, manage rules, and configure the system. This API is designed to facilitate integration with CI/CD pipelines, automated testing frameworks, and custom security tools.

## Base URL

By default, the API is served at:

```
http://localhost:3000/api/v1
```

## Authentication

Currently, the API is protected via a simple token-based authentication mechanism if configured.

Include the `X-API-Token` header in your requests:

```http
X-API-Token: your-secret-token
```

(Note: Authentication is optional in the default development configuration but recommended for production or shared environments).

## Key Resources

-   **Traffic**: Access and filter captured HTTP/HTTPS requests and responses.
-   **Interception**: Control the interception queue (hold, forward, drop requests).
-   **Rules**: Manage automated modification rules.
-   **Scope**: Configure target scopes (include/exclude patterns).
-   **Scan**: Initiate and manage active scans (Intruder/Fuzzer).

## OpenAPI Specification

A complete OpenAPI 3.0 specification is available in [openapi.yaml](openapi.yaml). You can import this file into tools like Postman, Insomnia, or Swagger UI to explore and test the API interactively.

## Usage Examples

### Get Captured Traffic

```bash
curl -X GET "http://localhost:3000/api/v1/traffic?limit=10" \
     -H "X-API-Token: your-token"
```

### Add a Scope Rule

```bash
curl -X POST "http://localhost:3000/api/v1/scope" \
     -H "Content-Type: application/json" \
     -d '{
           "type": "include",
           "pattern": "*.example.com",
           "enabled": true
         }'
```

### Start a Scan

```bash
curl -X POST "http://localhost:3000/api/v1/scan" \
     -H "Content-Type: application/json" \
     -d '{
           "target": "https://example.com/login",
           "profile": "sql-injection-basic"
         }'
```
