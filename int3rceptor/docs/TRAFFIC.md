# 🚦 Traffic Tab Documentation

The **Traffic Tab** is the central hub for monitoring and analyzing HTTP/HTTPS traffic flowing through the Interceptor proxy.

## Features

### 1. Real-time Capture

-   View requests and responses as they happen.
-   WebSocket support for real-time data streams.
-   Automatic decoding of various content types (JSON, XML, HTML, etc.).

### 2. Filtering & Search

-   **Quick Filter**: Filter by method, domain, or status code.
-   **Advanced Search**: Use regex to find specific patterns in headers or bodies.
-   **Highlighting**: Color-code requests based on specific criteria.

### 3. Request Details

-   **Headers**: View and edit request/response headers.
-   **Body**: Pretty-print JSON, XML, and other formats.
-   **Hex View**: Inspect binary data.

## How to Use

1. **Start the Proxy**: Ensure the Interceptor proxy is running.
2. **Configure Browser**: Set your browser's proxy to `127.0.0.1:8080`.
3. **Navigate**: Browse the web. Requests will appear in the Traffic list.
4. **Inspect**: Click on any row to view full details in the right pane.
