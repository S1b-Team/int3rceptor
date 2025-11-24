# ⚡ Rules Tab Documentation

The **Rules Tab** allows you to define automatic modification rules for traffic, enabling you to tamper with requests and responses on the fly without manual intervention.

## Rule Types

### 1. Request Modification

-   **Match**: Define criteria (e.g., Domain matches `example.com`, Header contains `User-Agent`).
-   **Action**: Set actions (e.g., Replace Header, Delete Parameter, Update Body).

### 2. Response Modification

-   **Match**: Define criteria (e.g., Status Code is `200`, Body contains `admin`).
-   **Action**: Set actions (e.g., Replace Body, Add Header).

## Creating a Rule

1. **Click "New Rule"**: Open the rule creation dialog.
2. **Set Name**: Give your rule a descriptive name.
3. **Define Matchers**:
    - Select the part of the request/response to match.
    - Choose the operator (Equals, Contains, Regex).
    - Enter the value.
4. **Define Actions**:
    - Select the action type.
    - Enter the target and value.
5. **Save**: Enable the rule to start modifying traffic.

## Examples

-   **Bypass Client-Side Validation**: Remove `disabled` attribute from HTML responses.
-   **Spoof User Agent**: Automatically change `User-Agent` header to a mobile device.
-   **Inject Headers**: Add `X-Custom-Header: true` to all requests.
