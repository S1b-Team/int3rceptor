# Troubleshooting

This guide covers common issues you might encounter when using Int3rceptor and how to resolve them.

## Certificate Issues

### "Your connection is not private"

This is the most common issue and usually means the Int3rceptor CA certificate is not properly installed or trusted by your browser.

**Solution:**

1.  Ensure you have exported the certificate: `./interceptor --export-ca ca.pem`.
2.  **Firefox**: Go to Settings > Privacy & Security > Certificates > View Certificates > Authorities > Import. Select `ca.pem` and check "Trust this CA to identify websites".
3.  **Chrome/Edge**: These browsers use the system store. Ensure you've added the certificate to your OS Trusted Root Certification Authorities.
4.  Restart your browser.

## Connection Issues

### "Proxy Refusing Connections"

If your browser cannot connect at all:

1.  Ensure Int3rceptor is running.
2.  Check the port configuration. By default, it listens on `8080`.
3.  Verify your browser proxy settings are pointing to `127.0.0.1` port `8080`.
4.  If running in Docker, ensure port mapping is correct (`-p 8080:8080`).

### "Gateway Timeout" or Slow Requests

If requests are hanging:

1.  Check if you have **Interception** enabled and paused. If the queue is paused, requests will wait indefinitely for your action.
2.  Check the **Dashboard** or **Intercept** tab and click "Forward" or disable interception.

## Docker Issues

### "Connection Refused" inside Docker

If you are trying to proxy traffic from another container through Int3rceptor:

1.  Ensure both containers are on the same Docker network.
2.  Use the container name (e.g., `interceptor`) as the proxy host, not `localhost`.

## Reporting Bugs

If you encounter a bug not listed here, please report it via:

-   **GitHub Issues**: Include your OS, Int3rceptor version, and steps to reproduce.
-   **Matrix Channel**: Reach out to the community for quick help.
