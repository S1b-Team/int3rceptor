# Advanced Features

Once you've mastered the core basics, Int3rceptor offers powerful tools for automated testing and complex workflows.

## The Intruder (Fuzzer)

The **Intruder** is a powerful fuzzing tool used to automate customized attacks against web applications. It allows you to configure a request template and inject payloads into specific positions.

### Attack Modes

Int3rceptor supports four attack modes:

1.  **Sniper**: Uses a single payload set. It targets each position in turn and places each payload into that position. Useful for fuzzing individual parameters.
2.  **Battering Ram**: Uses a single payload set. It iterates through the payloads and places the _same_ payload into _all_ defined positions at once.
3.  **Pitchfork**: Uses multiple payload sets. It iterates through all payload sets simultaneously and places one payload into each position. (e.g., Payload 1 into Position 1, Payload 2 into Position 2).
4.  **Cluster Bomb**: Uses multiple payload sets. It iterates through each payload set in turn so that all permutations of payload combinations are tested. This can generate a very large number of requests.

### Setting Up an Attack

1.  Send a request to Intruder (Right-click in Traffic tab -> **Send to Intruder**).
2.  In the **Positions** tab, highlight the parts of the request you want to fuzz and click **Add §**.
3.  Select your **Attack Mode**.
4.  Go to the **Payloads** tab and define the list of payloads for each position (e.g., a list of usernames or common passwords).
5.  Click **Start Attack**.

## Automation Rules

The **Rules** engine allows you to modify traffic on the fly without manual intervention. This is useful for bypassing client-side restrictions, stripping headers, or mocking responses.

### Creating a Rule

1.  Navigate to the **Rules** tab.
2.  Click **Add Rule**.
3.  **Condition**: Define when the rule should apply (e.g., `URL contains /api/v1/user`).
4.  **Action**: Define what to do.
    -   **Replace**: Swap text in the header or body.
    -   **Remove**: Delete a header.
    -   **Delay**: Introduce artificial latency (useful for testing race conditions).

## Scope Configuration

Managing scope is critical when testing complex applications to avoid attacking out-of-scope targets (like third-party analytics or CDNs).

### Include vs. Exclude

-   **Include Patterns**: Only traffic matching these patterns will be shown in the Traffic tab and processed by tools.
-   **Exclude Patterns**: Traffic matching these patterns will be ignored, even if it matches an Include pattern.

### Wildcards

You can use simple wildcards:

-   `*.example.com` matches `www.example.com`, `api.example.com`, etc.
-   `https://example.com/api/*` matches any path under `/api/`.
