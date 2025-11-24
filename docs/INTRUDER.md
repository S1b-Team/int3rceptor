# ⚔️ Intruder Tab Documentation

The **Intruder Tab** is a powerful tool for automated payload fuzzing, allowing you to test for vulnerabilities like SQL injection, XSS, and more.

## Attack Modes

### 1. Sniper

-   **Description**: Places each payload into each position, one by one.
-   **Use Case**: Testing individual parameters for vulnerabilities.

### 2. Battering Ram

-   **Description**: Places the same payload into all positions simultaneously.
-   **Use Case**: Testing for consistency across multiple parameters.

### 3. Pitchfork

-   **Description**: Uses multiple payload sets, placing payload 1 into position 1, payload 2 into position 2, etc.
-   **Use Case**: Testing related parameters (e.g., username/password pairs).

### 4. Cluster Bomb

-   **Description**: Tests every combination of payloads across all positions.
-   **Use Case**: Exhaustive testing of multiple parameters.

## How to Use

1. **Send to Intruder**: Right-click a request in the Traffic tab and select "Send to Intruder".
2. **Define Positions**: Highlight the parts of the request you want to fuzz and click "Add §".
3. **Configure Payloads**: Select a payload list (e.g., SQLi, XSS) or create a custom one.
4. **Start Attack**: Click "Start Attack" to begin fuzzing.
5. **Analyze Results**: Sort by status code or length to identify anomalies.
