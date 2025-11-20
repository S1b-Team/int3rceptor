# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to **s1bgr0up.root@gmail.com**.

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the following information in your report:

-   Type of issue (e.g. buffer overflow, SQL injection, cross-site scripting, etc.)
-   Full paths of source file(s) related to the manifestation of the issue
-   The location of the affected source code (tag/branch/commit or direct URL)
-   Any special configuration required to reproduce the issue
-   Step-by-step instructions to reproduce the issue
-   Proof-of-concept or exploit code (if possible)
-   Impact of the issue, including how an attacker might exploit it

This information will help us triage your report more quickly.

## Preferred Languages

We prefer all communications to be in English or Spanish.

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any similar problems
3. Prepare fixes for all supported releases
4. Release new security fix versions as soon as possible

## Comments on this Policy

If you have suggestions on how this process could be improved, please submit a pull request.

## Security Best Practices

When using Interceptor, we recommend:

### 1. Certificate Management

-   Keep the CA private key secure
-   Rotate certificates periodically
-   Never share the CA certificate publicly

### 2. API Authentication

-   Always set `INTERCEPTOR_API_TOKEN` in production
-   Use strong, random tokens (32+ characters)
-   Rotate tokens regularly

### 3. Network Security

-   Run on localhost or isolated network
-   Use firewall rules to restrict access
-   Consider using a VPN for remote access

### 4. Database Security

-   Encrypt SQLite database at rest
-   Restrict file system permissions
-   Regular backups with encryption

### 5. Scope Configuration

-   Use exclude patterns for sensitive domains
-   Avoid capturing authentication tokens
-   Review captured data regularly

### 6. Updates

-   Keep Interceptor updated to the latest version
-   Subscribe to security advisories
-   Review CHANGELOG.md for security fixes

## Known Security Considerations

### TLS MITM

Interceptor performs TLS MITM by design. This means:

-   All HTTPS traffic is decrypted
-   The CA certificate must be trusted
-   Captured data includes sensitive information

**Use responsibly and only on systems you own or have permission to test.**

### Data Storage

-   Captured traffic is stored in plaintext in SQLite
-   Database contains full request/response bodies
-   Consider encryption for sensitive environments

### API Exposure

-   API has no authentication by default
-   Set `INTERCEPTOR_API_TOKEN` for production use
-   Bind to localhost unless remote access is needed

## Security Hall of Fame

We recognize and thank the following security researchers:

_(No reports yet - be the first!)_

## Contact

For security-related questions or concerns:

-   -   Email: s1bgr0up.root@gmail.com
-   PGP Key: (Coming soon)

For general questions:

-   GitHub Discussions: https://github.com/S1b-Team/int3rceptor/discussions
-   -   Email: s1bgr0up.root@gmail.com
