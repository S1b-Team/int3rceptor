# Pending Mockup Generation

## Status

Image generation quota exhausted. Resets at: **2025-11-22 05:04:27 UTC**

## Pending Mockups (with Int3rceptor branding)

### 1. Intruder/Fuzzer Tab

**Prompt**: Int3rceptor Intruder/Fuzzer tab interface. Use the exact Int3rceptor logo (hexagonal with circuit board design). Color scheme: black #0a0a0f background, cyan #00d4ff primary, magenta #ff006e secondary, orange #ffb800 tertiary. Show HTTP request template with payload markers §username§ and §password§ in magenta. Attack type dropdown with "Cluster Bomb" selected. Two payload columns with cyan headers. Results table with cyan columns. "Start Attack" button in cyan hexagonal shape.

**Reference Logo**: `/home/il1v3y/.gemini/antigravity/brain/ea9b7f2c-2347-422a-999f-b17402c14e13/uploaded_image_1763770017731.jpg`

### 2. Repeater Tab

**Prompt**: Int3rceptor Request Repeater split-screen interface. Include Int3rceptor hexagonal logo. Colors: black #0a0a0f, cyan #00d4ff, magenta #ff006e, orange #ffb800. Left panel: editable HTTP request with method dropdown, URL input, headers (keys in cyan), JSON body with syntax highlighting. "Send" button in cyan hexagonal shape. Right panel: HTTP response with status badge in cyan, headers accordion, syntax-highlighted response body. Toggle buttons "Raw"/"Formatted".

**Reference Logo**: `/home/il1v3y/.gemini/antigravity/brain/ea9b7f2c-2347-422a-999f-b17402c14e13/uploaded_image_1763770017731.jpg`

### 3. WebSocket Tab

**Prompt**: Int3rceptor WebSocket interception tab. Show Int3rceptor logo. Colors: black #0a0a0f, cyan #00d4ff, magenta #ff006e, orange #ffb800. Top: active connections list. Main area: bidirectional frame timeline with cyan arrows for client→server (cyan left border) and magenta arrows for server→client (magenta left border). Frame details with syntax-highlighted JSON. Right panel: metadata with cyan/magenta/orange stats. "Pause Capture" button in orange.

**Reference Logo**: `/home/il1v3y/.gemini/antigravity/brain/ea9b7f2c-2347-422a-999f-b17402c14e13/uploaded_image_1763770017731.jpg`

### 4. Rules & Scope Tab (Bonus)

**Prompt**: Int3rceptor Rules and Scope tab. Int3rceptor logo in top left. Colors: black #0a0a0f, cyan #00d4ff, magenta #ff006e, orange #ffb800. Left: Rules list with cyan checkboxes, rule names, conditions. Right: Scope configuration with "Include Patterns" (cyan header) and "Exclude Patterns" (magenta header). Toggle switches in cyan. Stats at bottom.

**Reference Logo**: `/home/il1v3y/.gemini/antigravity/brain/ea9b7f2c-2347-422a-999f-b17402c14e13/uploaded_image_1763770017731.jpg`

## How to Generate

After quota resets, run:

```bash
# Use generate_image tool with ImagePaths parameter pointing to the logo
generate_image(
    ImageName="intruder_branded",
    ImagePaths=["/home/il1v3y/.gemini/antigravity/brain/ea9b7f2c-2347-422a-999f-b17402c14e13/uploaded_image_1763770017731.jpg"],
    Prompt="[Use prompt from above]"
)
```

Then copy to assets:

```bash
cp /home/il1v3y/.gemini/antigravity/brain/ea9b7f2c-2347-422a-999f-b17402c14e13/intruder_branded_*.png assets/screenshots/intruder-tab.png
cp /home/il1v3y/.gemini/antigravity/brain/ea9b7f2c-2347-422a-999f-b17402c14e13/repeater_branded_*.png assets/screenshots/repeater-tab.png
cp /home/il1v3y/.gemini/antigravity/brain/ea9b7f2c-2347-422a-999f-b17402c14e13/websocket_branded_*.png assets/screenshots/websocket-tab.png
```

## Completed Mockups

-   ✅ Dashboard (`assets/screenshots/dashboard.png`)
-   ✅ Traffic Tab (`assets/screenshots/traffic-tab.png`)
