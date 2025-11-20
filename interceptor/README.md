<div align="center">

![int3rceptor Banner](assets/project_banner.png)

[![License: Proprietary](https://img.shields.io/badge/License-Proprietary-red.svg?style=for-the-badge)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Vue](https://img.shields.io/badge/vue-3.4%2B-green.svg?style=for-the-badge&logo=vue.js&logoColor=white)](https://vuejs.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=for-the-badge)](CONTRIBUTING.md)
[![Commercial License](https://img.shields.io/badge/Commercial-Available-blue.svg?style=for-the-badge)](LICENSE_COMMERCIAL.md)

**A modern, high-performance security testing platform built with Rust and Vue.js**

[Features](#-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Contributing](#-contributing) • [Commercial License](#-commercial-licensing)

</div>

---

## 🌟 Overview

**Interceptor** is a **proprietary** HTTP/HTTPS intercepting proxy designed for security professionals, penetration testers, and developers. Built with Rust for maximum performance and Vue.js for a modern user experience, it rivals commercial tools like Burp Suite.

### 🔒 Licensing

-   **Free for Personal/Non-Commercial Use** - View source, report bugs, suggest features
-   **Commercial License Required** - For business use, see [Commercial Licensing](#-commercial-licensing)
-   **Source Available** - Code is visible for transparency and security auditing
-   **Proprietary** - All rights reserved by S1BGr0uP

### Why Interceptor?

-   **🚀 Blazing Fast**: Rust-powered async I/O handles thousands of concurrent connections
-   **🎨 Modern UI**: Beautiful Vue 3 interface with real-time updates via WebSocket
-   **🔧 Powerful Features**: Rule engine, intruder, scope management, regex matchers, WebSocket interception
-   **🔍 Source Available**: Code is visible for transparency and security auditing
-   **💼 Commercial Licensing**: Flexible licensing options for businesses

---

## ✨ Features

### Core Capabilities

#### 🔍 **Traffic Interception**

-   Full HTTP/HTTPS proxy with TLS MITM
-   Automatic certificate generation and management
-   Real-time traffic capture and analysis
-   WebSocket-based live updates

#### 🎯 **Intruder / Fuzzer**

-   **4 Attack Types**: Sniper, Battering Ram, Pitchfork, Cluster Bomb
-   Automated payload injection
-   Pre-loaded security payloads (SQL injection, XSS, path traversal)
-   Visual attack configuration
-   Real-time results with status coloring

#### 🔍 **Regex Matchers** ⚡ NEW in v2.0

-   **Advanced Pattern Matching**: Use regex in URL, headers, and body
-   **Capture Groups**: Extract and reuse matched patterns with $1, $2, etc.
-   **Intelligent Caching**: 10-100x performance improvement
-   **Powerful Replacements**: Transform traffic with regex-based rules
-   **Examples**:
    -   Redact API keys: `"api_key":\s*"([^"]+)"` → `"api_key": "REDACTED"`
    -   Validate URLs: `^https://api\.example\.com/v[0-9]+/`
    -   Transform headers with capture groups

#### 🔌 **WebSocket Interception** 🆕 NEW in v2.0

-   **Full Frame Capture**: Text, Binary, Ping, Pong, Close frames
-   **Bidirectional Monitoring**: Track Client↔Server communication
-   **Connection Tracking**: Lifecycle management and metadata
-   **Frame Analysis**: Timestamp, direction, payload, mask status
-   **Memory Efficient**: FIFO storage with configurable limits
-   **REST API**: Query connections and frames programmatically

#### 🔧 **Rule Engine**

-   Automatic traffic modification
-   Match conditions: URL, Header, Body
-   Actions: Replace, Set Header, Remove Header
-   Request/Response filtering

#### 🎨 **Syntax Highlighting**

-   Auto-detection: JSON, XML, HTML, CSS, JavaScript
-   Pretty-printing with indentation
-   Copy-to-clipboard functionality
-   Binary file detection

#### 🎯 **Scope Management**

-   Include/Exclude pattern matching
-   Reduce noise by filtering unwanted traffic
-   Smart logic with priority rules

#### 🔄 **Request Repeater**

-   Modify and replay requests
-   Edit method, URL, headers, body
-   Real-time response viewing

#### 💾 **Persistence & Export**

-   SQLite-backed storage
-   Advanced filtering and search
-   Export to JSON, CSV, HAR formats

---

## 🚀 Quick Start

### Prerequisites

-   **Rust** 1.70+ ([Install](https://rustup.rs/))
-   **Node.js** 18+ ([Install](https://nodejs.org/))
-   **npm** or **yarn**

### Installation

```bash
# Clone the repository
git clone https://github.com/S1b-Team/int3rceptor.git
cd interceptor

# Build the backend
cargo build --release

# Build the frontend
cd ui
npm install
npm run build
cd ..

# Run Interceptor
./target/release/interceptor
```

### Default Configuration

-   **Proxy**: `http://127.0.0.1:8080`
-   **Dashboard**: `http://127.0.0.1:3000`

### Browser Configuration

Configure your browser to use `127.0.0.1:8080` as the HTTP/HTTPS proxy.

**Firefox**: Settings → Network Settings → Manual proxy configuration  
**Chrome**: Settings → System → Open proxy settings

---

## 🔐 Certificate Installation

For HTTPS interception, install the CA certificate:

### Export Certificate

```bash
# Option 1: CLI
./target/release/interceptor --export-ca ./interceptor-ca.pem

# Option 2: Dashboard
# Visit http://127.0.0.1:3000 and click "Download CA"
```

### Install Certificate

<details>
<summary><b>macOS</b></summary>

1. Open **Keychain Access**
2. Select **System** keychain
3. File → Import Items → Select `interceptor-ca.pem`
4. Double-click the certificate
5. Expand **Trust** section
6. Set "When using this certificate" to **Always Trust**

</details>

<details>
<summary><b>Windows</b></summary>

1. Press `Win + R`, type `certmgr.msc`
2. Navigate to **Trusted Root Certification Authorities** → **Certificates**
3. Right-click → All Tasks → Import
4. Select `interceptor-ca.pem`
5. Complete the wizard

</details>

<details>
<summary><b>Linux</b></summary>

```bash
# System-wide (Chrome, curl, etc.)
sudo cp interceptor-ca.pem /usr/local/share/ca-certificates/interceptor.crt
sudo update-ca-certificates

# Firefox (manual)
# Settings → Privacy & Security → Certificates → View Certificates
# Import interceptor-ca.pem
```

</details>

---

## 📖 Documentation

### User Guides

-   **[Traffic Tab](docs/TRAFFIC.md)**: Capture and analyze HTTP/HTTPS traffic
-   **[Intruder Tab](docs/INTRUDER.md)**: Automated payload fuzzing
-   **[Rules Tab](docs/RULES.md)**: Automatic traffic modification
-   **[Scope Tab](docs/SCOPE.md)**: Filter captured traffic
-   **[Repeater Tab](docs/REPEATER.md)**: Modify and replay requests

### Technical Documentation

-   **[Architecture](docs/ARCHITECTURE.md)**: System design and components
-   **[API Reference](docs/API.md)**: REST API endpoints
-   **[Configuration](docs/CONFIG.md)**: Environment variables and settings
-   **[Development](docs/DEVELOPMENT.md)**: Contributing guide

---

## 🎯 Usage Examples

### Example 1: SQL Injection Testing

```bash
1. Navigate to Intruder tab
2. Load template:
   POST /api/user?id=§id§

3. Add payloads:
   1
   1' OR '1'='1
   1; DROP TABLE users--

4. Select "Sniper" attack type
5. Click "Generate Attack"
6. Analyze results for anomalies
```

### Example 2: API Key Redaction

```bash
1. Go to Rules tab
2. Create rule:
   - Type: Response
   - Condition: Body Contains "api_key"
   - Action: Replace Body "api_key": ".*" → "api_key": "REDACTED"
3. All future responses automatically redacted
```

### Example 3: Scope Configuration

```bash
1. Navigate to Scope tab
2. Add includes:
   - *.company.com
   - api.partner.com
3. Add excludes:
   - *.analytics.com
   - *.cdn.com
4. Save configuration
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Vue 3 + TypeScript)             │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐  │
│  │ Traffic  │  Rules   │  Scope   │ Repeater │ Intruder │  │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │ REST API + WebSocket
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      API Layer (Axum)                        │
│  /api/requests  /api/rules  /api/scope  /api/intruder      │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Core Engine (Rust + Tokio)                │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐  │
│  │  Proxy   │  Rules   │  Scope   │ Intruder │   TLS    │  │
│  │  Server  │  Engine  │ Manager  │          │   MITM   │  │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
                            ▼
                    ┌───────────────┐
                    │    SQLite     │
                    └───────────────┘
```

### Technology Stack

-   **Backend**: Rust, Tokio, Hyper, Axum, Rustls
-   **Frontend**: Vue 3, TypeScript, Vite, Axios
-   **Database**: SQLite
-   **Build**: Cargo, npm

---

## ⚙️ Configuration

### Environment Variables

| Variable                      | Default                   | Description                        |
| ----------------------------- | ------------------------- | ---------------------------------- |
| `INTERCEPTOR_DB_PATH`         | `data/interceptor.sqlite` | SQLite database location           |
| `INTERCEPTOR_API_TOKEN`       | None                      | API authentication token           |
| `INTERCEPTOR_MAX_BODY_BYTES`  | `2097152` (2MB)           | Maximum request/response body size |
| `INTERCEPTOR_MAX_CONCURRENCY` | `64`                      | Maximum concurrent connections     |

### Example Configuration

```bash
export INTERCEPTOR_DB_PATH="/var/lib/interceptor/data.db"
export INTERCEPTOR_API_TOKEN="your-secret-token"
export INTERCEPTOR_MAX_BODY_BYTES="10485760"  # 10MB
export INTERCEPTOR_MAX_CONCURRENCY="128"

./target/release/interceptor --listen 0.0.0.0:8080 --api 0.0.0.0:3000
```

---

## 🔒 Security Considerations

### Best Practices

1. **Certificate Management**: Keep CA private key secure
2. **API Authentication**: Always set `INTERCEPTOR_API_TOKEN` in production
3. **Network Isolation**: Run on localhost or isolated network
4. **Database Encryption**: Consider encrypting SQLite database at rest
5. **Scope Configuration**: Use excludes to avoid capturing sensitive domains

### Responsible Disclosure

If you discover a security vulnerability, please email s1bgr0up.root@gmail.com. Do not open public issues for security concerns.

---

## 🤝 Contributing

We welcome contributions from the community! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/S1b-Team/int3rceptor.git
cd interceptor

# Backend development
cargo build
cargo test
cargo run

# Frontend development
cd ui
npm install
npm run dev
```

### Code Style

-   **Rust**: Follow `rustfmt` and `clippy` recommendations
-   **TypeScript**: Follow Vue 3 + TypeScript best practices
-   **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/)

---

## 📊 Comparison

| Feature                 | Interceptor  | Burp Suite Pro | Mitmproxy | ZAP      |
| ----------------------- | ------------ | -------------- | --------- | -------- |
| **Price**               | **FREE**     | $449/year      | **FREE**  | **FREE** |
| **Language**            | Rust         | Java           | Python    | Java     |
| **UI**                  | Vue 3        | Swing          | CLI/Web   | Swing    |
| **Intruder**            | ✅ (4 modes) | ✅ (4 modes)   | ❌        | ✅       |
| **Rule Engine**         | ✅           | ✅             | ✅        | ❌       |
| **Syntax Highlighting** | ✅           | ✅             | ✅        | ✅       |
| **Real-time WebSocket** | ✅           | ❌             | ❌        | ❌       |
| **Performance**         | ⚡⚡⚡       | 🐌             | ⚡        | 🐌       |

---

---

## 💼 **Commercial Licensing**

### Free vs Commercial Use

| Use Case                            | Free | Commercial License Required |
| ----------------------------------- | ---- | --------------------------- |
| Personal projects                   | ✅   | ❌                          |
| Learning/Education                  | ✅   | ❌                          |
| Security research                   | ✅   | ❌                          |
| Bug bounty hunting                  | ✅   | ❌                          |
| **Business/Corporate**              | ❌   | ✅                          |
| **Penetration testing services**    | ❌   | ✅                          |
| **Security consulting**             | ❌   | ✅                          |
| **Any revenue-generating activity** | ❌   | ✅                          |

### Commercial License Tiers

#### 🏢 **Enterprise License** - $2,499/year

-   Unlimited commercial use
-   Deploy on unlimited servers
-   Modify source code for internal use
-   Priority support (48h response)
-   Security updates and quarterly features

#### 🚀 **Startup License** - $499/year

-   Commercial use (up to 10 users)
-   Deploy on up to 5 servers
-   Email support (72h response)
-   Security updates

#### 👤 **Individual License** - $99/year

-   Commercial use (single user)
-   Deploy on up to 2 servers
-   Community support

### How to Purchase

📧 **Email**: s1bgr0up.root@gmail.com  
📄 **Details**: See [LICENSE_COMMERCIAL.md](LICENSE_COMMERCIAL.md)

---

## 📜 License

### Proprietary License

**Copyright © 2025 S1BGr0uP. All rights reserved.**

This software is licensed under a **proprietary license**:

-   ✅ **Free for personal/non-commercial use**
-   ✅ **Source code available** for transparency
-   ✅ **Contributions welcome** (see [CONTRIBUTING.md](CONTRIBUTING.md))
-   ❌ **Commercial use requires a license**
-   ❌ **No redistribution or forking** for derivative works
-   ❌ **No modification** without permission

**Full license**: [LICENSE](LICENSE)  
**Commercial licensing**: [LICENSE_COMMERCIAL.md](LICENSE_COMMERCIAL.md)

### Why Proprietary?

1. **Sustainability** - Commercial licenses fund development
2. **Quality** - Professional support for paying customers
3. **Control** - Maintain project direction and quality
4. **Transparency** - Source code visible for security auditing

### Contributing

We welcome contributions! By contributing, you agree that:

-   All contributions become property of S1BGr0uP
-   You'll be credited in project documentation
-   See [CONTRIBUTING.md](CONTRIBUTING.md) for details

---

## 🙏 Acknowledgments

-   Built with ❤️ by [S1BGr0uP](https://github.com/S1BGr0uP)
-   Inspired by Burp Suite, Mitmproxy, and Charles Proxy
-   Special thanks to the Rust and Vue.js communities
-   Contributors: See [CONTRIBUTORS.md](CONTRIBUTORS.md)

---

## 📞 Support

### Community Support (Free)

-   **Issues**: [GitHub Issues](https://github.com/S1b-Team/int3rceptor/issues)
-   **Discussions**: [GitHub Discussions](https://github.com/S1b-Team/int3rceptor/discussions)
-   **Matrix**: @ind4skylivey:matrix.org

### Commercial Support (Paid)

-   **Email**: s1bgr0up.root@gmail.com
-   **Matrix**: @ind4skylivey:matrix.org
-   **Priority Support**: Available with Enterprise/Startup licenses
-   **Custom Development**: Contact s1bgr0up.root@gmail.com

---

## ⚠️ Legal Notice

**Interceptor is a security testing tool. Use responsibly and only on systems you own or have explicit permission to test.**

-   ✅ Authorized security testing
-   ✅ Educational purposes
-   ✅ Personal projects
-   ❌ Unauthorized access
-   ❌ Illegal activities
-   ❌ Violating terms of service

**S1BGr0uP is not responsible for misuse of this software.**

---

<div align="center">

**⭐ Star us on GitHub — it motivates us a lot!**

Made with 🦀 Rust and 🖼️ Vue.js

</div>
