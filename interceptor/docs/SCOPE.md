# 🎯 Scope Tab Documentation

The **Scope Tab** helps you focus your testing by filtering out noise. You can define which domains and URLs are in scope for your assessment.

## Configuration

### Include List

-   Traffic matching these rules will be **shown** and **processed**.
-   Supports wildcards (e.g., `*.example.com`) and Regex.

### Exclude List

-   Traffic matching these rules will be **ignored** or **hidden**.
-   Useful for excluding analytics, ads, and out-of-scope domains.

## Features

-   **Global Scope**: Applies to Traffic, Intruder, and Scanner.
-   **Toggle**: Quickly enable/disable scope filtering.
-   **Import/Export**: Save scope configurations for different projects.

## Best Practices

1. **Be Specific**: Define scope as tightly as possible to avoid attacking unauthorized targets.
2. **Exclude Noise**: Add common ad networks and analytics domains to the Exclude list.
3. **Verify**: Check the Traffic tab to ensure only intended traffic is being captured.
