# Core Features

This guide covers the fundamental features of Int3rceptor that you'll use on a daily basis for security testing and debugging.

## Traffic Interception

The **Traffic** tab is the central hub of Int3rceptor. It displays a real-time feed of all HTTP and HTTPS requests passing through the proxy.

### The Traffic Table

The main table provides a summary of each request:

-   **Method**: The HTTP verb (GET, POST, PUT, DELETE, etc.). Color-coded for quick recognition.
-   **URL**: The target URL. Hover over truncated URLs to see the full path.
-   **Status**: The HTTP response status code (e.g., 200 OK, 404 Not Found).
-   **Size**: The size of the response body.
-   **Time**: The total duration of the request/response cycle.

### Inspecting Details

Clicking on any row in the traffic table opens the **Details Panel** on the right.

-   **Headers**: View and copy request and response headers.
-   **Body**: Inspect the payload. Int3rceptor automatically formats JSON, XML, and HTML for readability.
-   **Raw**: View the raw bytes of the message, useful for debugging encoding issues.

### Filtering and Search

Use the search bar at the top to filter traffic.

-   **Simple Search**: Type any text to match against the URL or method.
-   **Advanced Filters**: (Coming soon) Use specific qualifiers like `method:POST` or `status:404`.

## The Repeater

The **Repeater** allows you to manually modify and resend requests. This is essential for testing how endpoints handle different inputs without needing to trigger them from the browser.

### Sending a Request to Repeater

1. Right-click any request in the **Traffic** tab.
2. Select **Send to Repeater**.
3. Navigate to the **Repeater** tab.

### Modifying Requests

In the Repeater tab, you have full control over the request:

-   **Method**: Change the HTTP verb.
-   **URL**: Edit the path or query parameters.
-   **Headers**: Add, remove, or modify headers (e.g., change `User-Agent` or `Authorization`).
-   **Body**: Edit the request body (JSON, form-data, etc.).

### Analyzing Responses

Click **Send** to execute the request. The response will appear in the right-hand panel. You can toggle between **Formatted** and **Raw** views to analyze the output.

## WebSocket Interception

Int3rceptor provides first-class support for WebSockets.

1. Navigate to the **WebSocket** tab.
2. Select an active connection from the list on the left.
3. View the real-time stream of messages (frames) in the center panel.
4. Messages are color-coded by direction:
    - **Blue/Cyan**: Client to Server
    - **Pink/Magenta**: Server to Client
5. Click any message to inspect its payload in the details panel.
