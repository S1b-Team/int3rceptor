# 🔌 API Reference

Interceptor exposes a REST API for automation and integration.

**Base URL**: `http://127.0.0.1:3000/api`

## Authentication

Currently, the API is accessible locally without authentication. For remote access (future), an API token will be required.

## Endpoints

### Traffic

-   **GET /traffic**

    -   Retrieve a list of captured requests.
    -   Query Params: `limit`, `offset`, `filter`.

-   **GET /traffic/:id**

    -   Retrieve full details of a specific request.

-   **DELETE /traffic**
    -   Clear the traffic history.

### Proxy Control

-   **POST /proxy/start**

    -   Start the proxy server.

-   **POST /proxy/stop**

    -   Stop the proxy server.

-   **GET /proxy/status**
    -   Get current status (running/stopped, port).

### Rules

-   **GET /rules**

    -   List all active rules.

-   **POST /rules**

    -   Create a new rule.

-   **DELETE /rules/:id**
    -   Delete a rule.

### Scope

-   **GET /scope**

    -   Get current scope configuration.

-   **POST /scope**
    -   Update scope configuration.
