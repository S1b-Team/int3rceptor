<div align="center">

![Interceptor Banner](interceptor/assets/banner.png)

# 🎯 Interceptor

### Professional HTTP/HTTPS Intercepting Proxy

[![License: Proprietary](https://img.shields.io/badge/License-Proprietary-red.svg?style=for-the-badge)](interceptor/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Vue](https://img.shields.io/badge/vue-3.4%2B-green.svg?style=for-the-badge&logo=vue.js&logoColor=white)](https://vuejs.org/)
[![Commercial License](https://img.shields.io/badge/Commercial-Available-blue.svg?style=for-the-badge)](interceptor/LICENSE_COMMERCIAL.md)

**A modern, high-performance security testing platform built with Rust and Vue.js**

[Features](#-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Commercial License](#-commercial-licensing)

</div>

---

## 🌟 Overview

**Interceptor** is a **proprietary** HTTP/HTTPS intercepting proxy designed for security professionals, penetration testers, and developers. Built with **Rust** for maximum performance and **Vue.js** for a modern user experience, it rivals commercial tools like Burp Suite but with a focus on speed and modern architecture.

### 🔒 Licensing

- ✅ **Free for Personal/Non-Commercial Use**
- ✅ **Source Available** for transparency and auditing
- 💼 **Commercial License Required** for business use
- 🔒 **Proprietary** - All rights reserved by S1BGr0uP

**Key Advantages:**

- ⚡ **10x Faster** - Rust's zero-cost abstractions eliminate JVM overhead
- 🎨 **Modern UX** - Real-time WebSocket dashboard, no page refreshes
- 💾 **Low Memory** - Efficient resource usage for long testing sessions
- 🔓 **Transparent** - Source available for security audit
- 💰 **Fair Pricing** - Free for personal use, affordable for professionals

---

## ✨ Features

### 🔍 **Traffic Interception**

Full HTTP/HTTPS proxy with automatic TLS MITM, self-signed CA certificate generation, and real-time WebSocket capture. Supports HTTP/2, HTTP/1.1, and connection pooling for optimal performance.

### 🎯 **Intruder / Fuzzer**

Automated payload injection with **4 attack modes:**

- **Sniper** - Single payload position iteration
- **Battering Ram** - Same payload in all positions
- **Pitchfork** - Multiple synchronized payloads
- **Cluster Bomb** - All payload combinations

Includes pre-loaded SecLists wordlists, custom payload support, rate limiting, and grep matching for vulnerability detection.

### 🔧 **Rule Engine**

Powerful traffic modification system with pattern matching:

- **Match by:** URL (regex/wildcard), Headers, Body content
- **Actions:** Replace text, Set/Remove headers, Drop requests
- **Live editing** - Modify requests/responses before forwarding

### 🔄 **Request Repeater**

Unlimited request modification and resending with:

- Tab management for multiple sessions
- Response comparison (diff view)
- Variable substitution for dynamic payloads
- Export to Fuzzer for automated attacks

### 📜 **History & Search**

Complete traffic capture with SQLite-based search:

- Advanced filters (method, status, host, size, content-type)
- Export formats: HAR, JSON, CSV, cURL commands
- Timeline visualization
- Automatic session recovery

### 🔧 **Decoder/Encoder Suite**

Essential encoding tools for security testing:

- URL, Base64, Hex, Binary, HTML Entities
- JWT decoder (header + payload inspection)
- Hash calculators (MD5, SHA-1, SHA-256, SHA-512)

### 🎨 **Comparer**

Visual diff tool for:

- Request vs Request comparison
- Response vs Response analysis
- Syntax-aware JSON/XML structural diff
- Word & character-level highlighting

### 🎯 **Scope Management**

Intelligent traffic filtering:

- Include/Exclude rules (regex & wildcards)
- Protocol filtering (HTTP/HTTPS/WebSocket)
- Auto-scope learning from browsing patterns
- Sensitive data protection (exclude banking, auth servers)

### 🚀 **Coming in v2.0**

- ⚡ **Advanced Regex Matchers** - Capture groups with intelligent caching
- 🔌 **Enhanced WebSocket Interception** - Frame-level analysis (Text, Binary, Ping/Pong)
- 🤖 **AI-Powered Vulnerability Detection**
- 🔗 **Collaboration Mode** - Team sharing and session sync

---

## 🎪 Real-World Use Cases

<details>
<summary><b>For Penetration Testers</b></summary>

- ✅ **SQL Injection Testing** - Systematic parameter fuzzing
- ✅ **Session Fixation Analysis** - Cookie generation pattern tracking
- ✅ **CSRF Detection** - Token comparison across requests
- ✅ **XXE Attacks** - Real-time XML payload modification
- ✅ **SSRF Exploration** - Outbound request monitoring

</details>

<details>
<summary><b>For Bug Bounty Hunters</b></summary>

- ✅ **Hidden Parameter Discovery** - Wordlist-based fuzzing
- ✅ **Race Condition Testing** - Parallel request execution
- ✅ **JWT Token Analysis** - Decode & modify claims
- ✅ **API Endpoint Enumeration** - Pattern-based discovery
- ✅ **IDOR Testing** - Systematic ID manipulation

</details>

<details>
<summary><b>For Developers</b></summary>

- ✅ **API Debugging** - Payload inspection & modification
- ✅ **Rate Limiting Tests** - Throttling behavior verification
- ✅ **Error Handling** - Edge case testing
- ✅ **Mobile App Traffic** - TLS communication decryption
- ✅ **WebSocket Debugging** - Real-time message monitoring

</details>

<details>
<summary><b>For Security Researchers</b></summary>

- ✅ **Protocol Analysis** - Proprietary format reverse engineering
- ✅ **Malware Traffic Analysis** - Safe MITM inspection
- ✅ **IoT Device Testing** - Embedded systems traffic capture
- ✅ **Cloud API Research** - Endpoint discovery & analysis
- ✅ **Supply Chain Attacks** - Third-party request monitoring

</details>

## ❓ Frequently Asked Questions

### Is int3rceptor better than Burp Suite?

**Performance:** Yes - Rust provides 3-5x faster request processing than Java  
**UI:** Modern Vue.js dashboard vs outdated Swing interface  
**Price:** $99/year vs $449/year for Burp Pro

### Can I use int3rceptor for bug bounty hunting?

Absolutely! Includes all essential tools: interceptor, repeater, fuzzer, decoder.

### Does it work with mobile apps (iOS/Android)?

Yes - configure your device's proxy settings to point to int3rceptor.

### What's the difference from OWASP ZAP?

- **Speed:** Rust vs Python (10x faster on large responses)
- **UI:** Real-time WebSocket updates vs polling
- **UX:** Designed for modern workflows

### Is the source code auditable?

Yes - source is available for review and security auditing under proprietary license.

### Can I use it offline?

Yes - all features work locally without internet connection.

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.70+
- **Node.js** 18+

### Installation

```bash
# Clone the repository
git clone https://github.com/S1b-Team/int3rceptor.git
cd int3rceptor/interceptor

# Build and Run
cargo run --release
```

For full installation details, see the [Development Guide](interceptor/docs/DEVELOPMENT.md).

---

## 📖 Documentation

### User Guides

- **[Traffic Tab](interceptor/docs/TRAFFIC.md)**: Capture and analyze traffic
- **[Intruder Tab](interceptor/docs/INTRUDER.md)**: Automated fuzzing
- **[Rules Tab](interceptor/docs/RULES.md)**: Traffic modification
- **[Scope Tab](interceptor/docs/SCOPE.md)**: Filter traffic
- **[Repeater Tab](interceptor/docs/REPEATER.md)**: Replay requests

### Technical Docs

- **[Architecture](interceptor/docs/ARCHITECTURE.md)**
- **[API Reference](interceptor/docs/API.md)**
- **[Configuration](interceptor/docs/CONFIG.md)**

---

<div align="center">
## 🏗️ Architecture

int3rceptor/
├── core/ # Proxy engine & Logic (Rust + Tokio)
│ ├── proxy.rs # Main proxy logic
│ ├── tls.rs # TLS 1.2/1.3 interception
│ ├── intruder.rs # Fuzzer engine
│ ├── rules.rs # Traffic modification
│ ├── scope.rs # Scope management
│ └── storage.rs # SQLite persistence
├── api/ # Web server (Axum)
│ ├── routes.rs # REST API endpoints
│ └── websocket.rs # Real-time updates
├── ui/ # Vue.js 3 frontend
│ ├── components/ # Reusable UI
│ ├── views/ # Page layouts
│ └── store/ # Pinia state
└── cli/ # Command-line interface
└── main.rs

</div>

**Tech Stack:**

- **Backend:** Rust 1.75+, Tokio (async), Hyper (HTTP), Rustls (TLS), Axum (web framework)
- **Frontend:** Vue.js 3, TypeScript, Vite, Pinia, Monaco Editor
- **Storage:** SQLite (requests), Redis (sessions - optional)

## 💰 Licensing

int3rceptor uses a dual-licensing model to remain sustainable:

### 📖 Personal/Non-Commercial Use

**FREE for:**

- Individual security researchers
- Students learning web security
- Personal bug bounty hunting
- Open-source project testing
- Educational purposes

### 💼 Commercial Use

**License Required ($99/year) for:**

- Professional penetration testing
- Security consulting services
- Corporate security teams
- Paid bug bounty programs
- Government/enterprise use

**Enterprise:** Custom pricing with SSO, priority support, SLA

### 🔓 Source Transparency

- ✅ Full source code available on GitHub for review
- ✅ Security auditing encouraged - no black boxes
- ✅ Issue tracking & contributions welcome
- 🔒 **Proprietary license** - all rights reserved by S1BGr0uP
- ❌ Redistribution prohibited without permission

📧 **Contact**: s1bgr0up.root@gmail.com  
📄 **Details**: [LICENSE_COMMERCIAL.md](interceptor/LICENSE_COMMERCIAL.md)

---

<div align="center">

**⭐ Star us on GitHub — it motivates us a lot!**

Made with 🦀 Rust and 🖼️ Vue.js by [S1BGr0uP](https://github.com/S1BGr0uP)

</div>
