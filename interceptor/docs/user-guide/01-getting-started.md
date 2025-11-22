# Getting Started with Int3rceptor

This guide will help you install and configure Int3rceptor for the first time.

## Prerequisites

Before installing Int3rceptor, ensure you have:

-   **Rust** 1.70 or later ([Install Rust](https://rustup.rs/))
-   **Node.js** 18 or later ([Install Node.js](https://nodejs.org/))
-   **npm** or **yarn** package manager
-   **Git** for cloning the repository

### System Requirements

| Component  | Minimum                   | Recommended |
| ---------- | ------------------------- | ----------- |
| CPU        | 2 cores                   | 4+ cores    |
| RAM        | 2 GB                      | 4+ GB       |
| Disk Space | 500 MB                    | 1 GB        |
| OS         | Linux, macOS, Windows 10+ | Linux/macOS |

## Installation

### Option 1: From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/S1b-Team/int3rceptor.git
cd interceptor

# Build the backend (Rust)
cargo build --release

# Build the frontend (Vue.js)
cd ui
npm install
npm run build
cd ..

# Verify installation
./target/release/interceptor --version
```

**Build time**: ~5-10 minutes on first build (Rust compilation)

### Option 2: Using Docker

```bash
# Pull the Docker image
docker pull s1bteam/int3rceptor:latest

# Run the container
docker run -d \
  -p 8080:8080 \
  -p 3000:3000 \
  --name interceptor \
  s1bteam/int3rceptor:latest
```

### Option 3: Pre-built Binaries

Download pre-built binaries from the [Releases page](https://github.com/S1b-Team/int3rceptor/releases):

-   **Linux**: `int3rceptor-linux-x64.tar.gz`
-   **macOS**: `int3rceptor-macos-x64.tar.gz`
-   **Windows**: `int3rceptor-windows-x64.zip`

```bash
# Extract and run
tar -xzf int3rceptor-linux-x64.tar.gz
cd int3rceptor
./interceptor
```

## First Run

Start Int3rceptor with default settings:

```bash
./target/release/interceptor
```

You should see output similar to:

```
🚀 Int3rceptor v2.0.0 starting...
✓ Proxy server listening on http://127.0.0.1:8080
✓ API server listening on http://127.0.0.1:3000
✓ Dashboard available at http://127.0.0.1:3000
✓ CA certificate ready (use --export-ca to save)
```

### Default Configuration

-   **Proxy Address**: `http://127.0.0.1:8080`
-   **Dashboard**: `http://127.0.0.1:3000`
-   **Database**: `data/interceptor.sqlite`
-   **CA Certificate**: Auto-generated on first run

## Certificate Installation

For HTTPS interception, you must install Int3rceptor's CA certificate in your system/browser.

### Step 1: Export the Certificate

```bash
# Export CA certificate to file
./target/release/interceptor --export-ca ./interceptor-ca.pem
```

Or download from the dashboard:

1. Open `http://127.0.0.1:3000`
2. Click **"Download CA Certificate"** button
3. Save as `interceptor-ca.pem`

### Step 2: Install Certificate (OS-Specific)

#### macOS

1. **Open Keychain Access** (Applications → Utilities → Keychain Access)
2. Select **System** keychain in the left sidebar
3. Go to **File → Import Items**
4. Select `interceptor-ca.pem`
5. Double-click the imported certificate ("Int3rceptor CA")
6. Expand the **Trust** section
7. Set **"When using this certificate"** to **"Always Trust"**
8. Close the window (you'll be prompted for your password)

**Verification**: The certificate should show a blue "+" icon indicating it's trusted.

#### Windows

1. Press `Win + R` and type `certmgr.msc`, press Enter
2. Navigate to **Trusted Root Certification Authorities → Certificates**
3. Right-click on **Certificates** → **All Tasks → Import**
4. Click **Next**, then **Browse** and select `interceptor-ca.pem`
5. Select **"Place all certificates in the following store"**
6. Choose **"Trusted Root Certification Authorities"**
7. Click **Next**, then **Finish**

**Verification**: You should see "The import was successful" message.

#### Linux (System-wide)

```bash
# Copy certificate to system CA directory
sudo cp interceptor-ca.pem /usr/local/share/ca-certificates/interceptor.crt

# Update CA certificates
sudo update-ca-certificates
```

**Expected output**: `1 added, 0 removed; done.`

**Note**: This works for Chrome, curl, wget, and most CLI tools.

#### Linux (Firefox only)

Firefox uses its own certificate store:

1. Open Firefox
2. Go to **Settings → Privacy & Security**
3. Scroll to **Certificates** section
4. Click **View Certificates**
5. Go to **Authorities** tab
6. Click **Import**
7. Select `interceptor-ca.pem`
8. Check **"Trust this CA to identify websites"**
9. Click **OK**

## Browser Configuration

Configure your browser to use Int3rceptor as a proxy.

### Firefox (Recommended)

1. Open **Settings** (or type `about:preferences` in address bar)
2. Scroll to **Network Settings**
3. Click **Settings** button
4. Select **Manual proxy configuration**
5. Set:
    - **HTTP Proxy**: `127.0.0.1` Port: `8080`
    - **HTTPS Proxy**: `127.0.0.1` Port: `8080`
    - Check **"Also use this proxy for HTTPS"**
    - Check **"Also use this proxy for FTP"** (optional)
6. Leave **SOCKS Host** empty
7. Add to **No Proxy for**: `localhost, 127.0.0.1`
8. Click **OK**

### Chrome / Edge / Brave

These browsers use system proxy settings:

**macOS**:

1. System Settings → Network
2. Select your network connection
3. Click **Details** → **Proxies**
4. Check **Web Proxy (HTTP)** and **Secure Web Proxy (HTTPS)**
5. Set both to `127.0.0.1:8080`
6. Click **OK**

**Windows**:

1. Settings → Network & Internet → Proxy
2. Under **Manual proxy setup**, click **Set up**
3. Enable **Use a proxy server**
4. Set **Address**: `127.0.0.1` Port: `8080`
5. Add `localhost;127.0.0.1` to **"Don't use proxy for"**
6. Click **Save**

**Linux**:

```bash
# Set environment variables
export http_proxy="http://127.0.0.1:8080"
export https_proxy="http://127.0.0.1:8080"
export no_proxy="localhost,127.0.0.1"
```

### Browser Extensions (Alternative)

For easier proxy switching, use browser extensions:

-   **Firefox**: [FoxyProxy](https://addons.mozilla.org/firefox/addon/foxyproxy-standard/)
-   **Chrome**: [Proxy SwitchyOmega](https://chrome.google.com/webstore/detail/proxy-switchyomega/padekgcemlokbadohgkifijomclgjgif)

## Verification

Test that everything is working:

### 1. Check Proxy Connection

```bash
# Test with curl
curl -x http://127.0.0.1:8080 http://example.com
```

You should see the HTML content of example.com, and the request should appear in the Int3rceptor dashboard.

### 2. Test HTTPS Interception

1. Open your browser (with proxy configured)
2. Visit `https://example.com`
3. Check for certificate warnings:
    - ✅ **No warning** = Certificate installed correctly
    - ❌ **Warning** = Certificate not trusted, review installation steps

### 3. Check Dashboard

1. Open `http://127.0.0.1:3000` in your browser
2. You should see the Int3rceptor dashboard
3. Navigate to the **Traffic** tab
4. You should see captured HTTP/HTTPS requests

## Configuration Options

### Command-line Arguments

```bash
./interceptor [OPTIONS]

OPTIONS:
    --listen <ADDR>        Proxy listen address (default: 127.0.0.1:8080)
    --api <ADDR>           API server address (default: 127.0.0.1:3000)
    --db <PATH>            Database file path (default: data/interceptor.sqlite)
    --export-ca <PATH>     Export CA certificate and exit
    --version              Print version and exit
    --help                 Print help information
```

### Environment Variables

| Variable                      | Default                   | Description                    |
| ----------------------------- | ------------------------- | ------------------------------ |
| `INTERCEPTOR_DB_PATH`         | `data/interceptor.sqlite` | SQLite database location       |
| `INTERCEPTOR_API_TOKEN`       | None                      | API authentication token       |
| `INTERCEPTOR_MAX_BODY_BYTES`  | `2097152` (2MB)           | Max request/response body size |
| `INTERCEPTOR_MAX_CONCURRENCY` | `64`                      | Max concurrent connections     |

### Example: Custom Configuration

```bash
# Run on custom ports with larger body size limit
export INTERCEPTOR_MAX_BODY_BYTES="10485760"  # 10MB

./interceptor \
  --listen 0.0.0.0:9090 \
  --api 0.0.0.0:4000 \
  --db /var/lib/interceptor/data.db
```

## Next Steps

Now that Int3rceptor is installed and configured:

1. **[Quick Start Tutorial](02-quick-start.md)** - Learn the basics in 5 minutes
2. **[Core Features](03-core-features.md)** - Explore traffic interception
3. **[Advanced Features](04-advanced-features.md)** - Master the Intruder and Rule Engine

## Troubleshooting

### Common Issues

**Problem**: "Address already in use" error

**Solution**: Another process is using port 8080 or 3000. Either stop that process or use different ports:

```bash
./interceptor --listen 127.0.0.1:9090 --api 127.0.0.1:4000
```

---

**Problem**: Certificate warnings in browser

**Solution**:

1. Verify certificate is installed in correct store
2. Restart browser after installing certificate
3. Check certificate is set to "Always Trust" (macOS) or in "Trusted Root" (Windows)

---

**Problem**: No traffic appearing in dashboard

**Solution**:

1. Verify browser proxy settings are correct
2. Check proxy is running: `curl -x http://127.0.0.1:8080 http://example.com`
3. Check dashboard is accessible: `http://127.0.0.1:3000`
4. Review scope settings (may be filtering traffic)

---

For more troubleshooting, see the [Troubleshooting Guide](06-troubleshooting.md).

---

**Previous**: [User Guide Home](README.md) | **Next**: [Quick Start Tutorial](02-quick-start.md) →
