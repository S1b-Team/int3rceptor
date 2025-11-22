# Quick Start Tutorial

**Goal**: Intercept your first HTTPS request in 5 minutes

This tutorial assumes you've already [installed Int3rceptor](01-getting-started.md) and configured your browser.

## What You'll Learn

-   ✅ Start Int3rceptor
-   ✅ Capture HTTP/HTTPS traffic
-   ✅ Inspect requests and responses
-   ✅ Modify and replay a request
-   ✅ Use basic filtering

**Time**: ~5 minutes

## Step 1: Start Int3rceptor (30 seconds)

```bash
cd interceptor
./target/release/interceptor
```

**Expected output**:

```
🚀 Int3rceptor v2.0.0 starting...
✓ Proxy server listening on http://127.0.0.1:8080
✓ API server listening on http://127.0.0.1:3000
✓ Dashboard available at http://127.0.0.1:3000
```

## Step 2: Open the Dashboard (15 seconds)

1. Open your browser
2. Navigate to `http://127.0.0.1:3000`
3. You should see the Int3rceptor dashboard

![Dashboard Overview](../assets/screenshots/dashboard.png)

## Step 3: Capture Your First Request (1 minute)

1. In the dashboard, click the **Traffic** tab
2. In your browser (with proxy configured), visit `https://httpbin.org/get`
3. Return to the Int3rceptor dashboard
4. You should see the request appear in the traffic list

![Traffic Tab](../assets/screenshots/traffic-tab.png)

### Understanding the Traffic View

| Column     | Description                           |
| ---------- | ------------------------------------- |
| **Method** | HTTP method (GET, POST, etc.)         |
| **URL**    | Request URL                           |
| **Status** | Response status code (200, 404, etc.) |
| **Size**   | Response body size                    |
| **Time**   | Request timestamp                     |

**Color coding**:

-   🟢 Green (2xx): Success
-   🟡 Yellow (3xx): Redirect
-   🔴 Red (4xx, 5xx): Error

## Step 4: Inspect Request/Response (1 minute)

1. Click on the request you just captured
2. The request details panel opens on the right
3. Explore the tabs:
    - **Request**: Headers, body, method, URL
    - **Response**: Status, headers, body
    - **Raw**: Complete HTTP message

![Request Details](../assets/screenshots/request-details.png)

### Try This:

Visit different websites and observe:

-   Different HTTP methods (GET, POST, PUT, DELETE)
-   Various content types (JSON, HTML, images)
-   Request/response headers
-   Query parameters

## Step 5: Use the Repeater (1 minute)

The Repeater lets you modify and replay requests.

1. Right-click on any request in the Traffic tab
2. Select **"Send to Repeater"**
3. Click the **Repeater** tab
4. Modify the request (try changing a header or URL parameter)
5. Click **"Send"** button
6. View the response

![Repeater Tab](../assets/screenshots/repeater-tab.png)

### Example: Modify a Request

Original request:

```http
GET /get?name=Alice HTTP/1.1
Host: httpbin.org
```

Modified request:

```http
GET /get?name=Bob HTTP/1.1
Host: httpbin.org
User-Agent: Int3rceptor/2.0
```

Click **Send** and observe how the response changes.

## Step 6: Filter Traffic (1 minute)

As you browse, the traffic list can get crowded. Use filters to focus on what matters.

### Quick Filters

In the Traffic tab, use the search box:

-   **By URL**: `httpbin.org`
-   **By method**: `POST`
-   **By status**: `200`
-   **By content type**: `json`

### Scope Management

For persistent filtering:

1. Click the **Scope** tab
2. Add include patterns:
    ```
    *.httpbin.org
    api.example.com
    ```
3. Add exclude patterns:
    ```
    *.google-analytics.com
    *.cdn.com
    ```
4. Click **Save**

Now only matching traffic will be captured.

## Congratulations! 🎉

You've successfully:

-   ✅ Started Int3rceptor
-   ✅ Captured HTTPS traffic
-   ✅ Inspected requests and responses
-   ✅ Modified and replayed a request
-   ✅ Filtered traffic

## Next Steps

### Learn Advanced Features

-   **[Intruder/Fuzzer](04-advanced-features.md#intruder-fuzzer)** - Automated payload injection with 4 attack modes
-   **[Rule Engine](04-advanced-features.md#rule-engine)** - Automatic traffic modification
-   **[WebSocket Interception](04-advanced-features.md#websocket-interception)** - Capture WebSocket frames

### Real-World Use Cases

-   **[API Security Testing](05-use-cases.md#api-testing)** - Test REST APIs for vulnerabilities
-   **[Web App Pentesting](05-use-cases.md#web-app-pentesting)** - Find XSS, SQLi, CSRF
-   **[Bug Bounty Hunting](05-use-cases.md#bug-bounty)** - Efficient bug hunting workflows

## Quick Reference

### Keyboard Shortcuts

| Shortcut       | Action                  |
| -------------- | ----------------------- |
| `Ctrl/Cmd + F` | Search traffic          |
| `Ctrl/Cmd + R` | Send to Repeater        |
| `Ctrl/Cmd + I` | Send to Intruder        |
| `Delete`       | Delete selected request |
| `Ctrl/Cmd + E` | Export traffic          |

### Common Tasks

**Clear traffic history**:

-   Traffic tab → Click "Clear All" button

**Export traffic**:

-   Traffic tab → Select requests → Click "Export" → Choose format (JSON/CSV/HAR)

**Stop capturing**:

-   Click the "Pause" button in the toolbar

**Resume capturing**:

-   Click the "Resume" button

## Troubleshooting

**Problem**: No traffic appearing

**Solutions**:

1. Verify proxy settings: `curl -x http://127.0.0.1:8080 http://example.com`
2. Check browser proxy configuration
3. Ensure scope isn't filtering everything out

---

**Problem**: Certificate warnings

**Solutions**:

1. Verify CA certificate is installed
2. Restart browser after installing certificate
3. See [Certificate Installation](01-getting-started.md#certificate-installation)

---

**Need more help?** See the [Troubleshooting Guide](06-troubleshooting.md)

---

**Previous**: [Getting Started](01-getting-started.md) | **Next**: [Core Features](03-core-features.md) →
