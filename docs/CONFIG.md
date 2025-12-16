# ⚙️ Configuration

Interceptor can be configured via environment variables or a configuration file.

## Environment Variables

| Variable       | Description                                 | Default                      |
| -------------- | ------------------------------------------- | ---------------------------- |
| `PORT`         | Port for the Web UI and API                 | `3000`                       |
| `PROXY_PORT`   | Port for the HTTP/HTTPS Proxy               | `8080`                       |
| `DATABASE_URL` | Path to SQLite database                     | `sqlite:data/interceptor.db` |
| `RUST_LOG`     | Log level (error, warn, info, debug, trace) | `info`                       |
| `CA_CERT_PATH` | Path to custom CA certificate               | `certs/ca.pem`               |
| `CA_KEY_PATH`  | Path to custom CA private key               | `certs/ca.key`               |

## Configuration File

You can also use a `config.toml` file in the root directory:

```toml
[server]
port = 3000
host = "127.0.0.1"

[proxy]
port = 8080
host = "127.0.0.1"
intercept_https = true

[database]
url = "sqlite:data/interceptor.db"
```

## Certificate Installation

To intercept HTTPS traffic, you must install the generated CA certificate in your browser or system trust store.

1. **Start Interceptor**.
2. **Download Cert**: Go to `http://interceptor.local/cert` (or export from UI).
3. **Install**:
    - **Firefox**: Settings -> Privacy & Security -> Certificates -> View Certificates -> Import.
    - **Chrome/System**: Double click `.crt` file and add to "Trusted Root Certification Authorities".
