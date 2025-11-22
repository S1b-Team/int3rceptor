# Int3rceptor UI Mockup Generation Prompts

## 📋 Instructions

Use these prompts in any AI image generator (Midjourney, DALL-E 3, Stable Diffusion, etc.) to create brand-consistent UI mockups for Int3rceptor.

**Important**: Upload the Int3rceptor logo as a reference image when available to ensure brand consistency.

---

## 🎨 Brand Colors Reference

-   **Background**: `#0a0a0f` (solid black)
-   **Primary Accent**: `#00d4ff` (cyan/turquoise)
-   **Secondary Accent**: `#ff006e` (hot pink/magenta)
-   **Tertiary Accent**: `#ffb800` (orange/gold)
-   **Text**: `#ffffff` (white) and `#a0a0a0` (gray)

---

## 📸 Prompt 1: Dashboard Overview

```
Int3rceptor security tool dashboard interface. Modern cyberpunk aesthetic.

COLORS (EXACT):
- Background: solid black #0a0a0f
- Primary accent: cyan #00d4ff
- Secondary accent: hot pink/magenta #ff006e
- Tertiary accent: orange #ffb800

LAYOUT:
- Top left: Int3rceptor hexagonal logo with circuit board design
- Navigation tabs (cyan text): Traffic, Intruder, Repeater, Rules, Scope
- Top right metrics bar: "Requests: 1,247" (cyan), "Memory: 45 MB" (orange), "Connections: 12" (magenta)

MAIN CONTENT:
- HTTP request table with cyan column headers: Method, URL, Status, Size, Time
- Rows alternating dark gray (#1a1a2e) and black
- Method badges: GET (cyan), POST (magenta), PUT (orange)
- Status codes: 200 (cyan), 404 (magenta), 500 (orange-red)

RIGHT SIDEBAR:
- Request details panel (dark gray background #1a1a2e)
- Section headers in cyan
- JSON syntax highlighting: keys (cyan), strings (magenta), numbers (orange)

STYLE:
- Hexagonal UI elements throughout
- Subtle circuit board patterns in background (very faint cyan glow)
- Professional security tool interface
- Clean, modern, cyberpunk aesthetic
- Monospace font for code sections
```

---

## 📸 Prompt 2: Traffic Interception Tab

```
Int3rceptor Traffic tab interface for HTTP/HTTPS request interception.

COLORS (EXACT):
- Background: solid black #0a0a0f
- Accents: cyan #00d4ff, magenta #ff006e, orange #ffb800

HEADER:
- Int3rceptor hexagonal logo (top left)
- Search bar with cyan border when focused
- Filter chips with cyan borders

MAIN TABLE:
- Columns in cyan text: Checkbox, Method, URL, Status, Size, Time
- Method badges (pill-shaped):
  - GET: cyan background
  - POST: magenta background
  - PUT: orange background
  - DELETE: red background
- Status codes colored:
  - 200-299: green-cyan
  - 300-399: purple
  - 400-499: magenta
  - 500-599: orange-red
- Selected row: cyan left border (5px thick)
- Hover state: subtle cyan glow

RIGHT PANEL:
- Request details with tabs: Headers | Body | Raw
- Active tab: cyan underline
- Headers section: keys in cyan, values in white
- JSON body with syntax highlighting:
  - Keys: cyan
  - Strings: magenta
  - Numbers: orange
  - Booleans: purple

BOTTOM:
- Pagination: "Showing 1-50 of 1,247 requests" (gray text)

STYLE:
- Hexagonal design elements
- Subtle circuit patterns
- Monospace font for code
- Professional cyberpunk security tool aesthetic
```

---

## 📸 Prompt 3: Intruder/Fuzzer Tab

```
Int3rceptor Intruder/Fuzzer tab for penetration testing and automated attacks.

COLORS (EXACT):
- Background: black #0a0a0f
- Primary: cyan #00d4ff
- Secondary: magenta #ff006e
- Tertiary: orange #ffb800

HEADER:
- Int3rceptor hexagonal logo (top left)
- Title: "Intruder" in cyan

TOP SECTION - REQUEST TEMPLATE:
- Background: dark gray #1a1a2e
- HTTP request with payload position markers: §username§ and §password§
- Markers highlighted with magenta background and black text
- Monospace font
- Line numbers in gray on the left

MIDDLE SECTION - ATTACK CONFIGURATION:
- Attack type dropdown showing: "Cluster Bomb" (selected with cyan highlight)
- Other options visible: Sniper, Battering Ram, Pitchfork
- Two payload columns side by side:
  - Left: "Payload Set 1" (cyan header) - items: admin, root, test, user
  - Right: "Payload Set 2" (cyan header) - items: password123, admin123, 12345
- Add button: cyan + icon
- Remove button: magenta × icon

BOTTOM SECTION - RESULTS TABLE:
- Columns in cyan: Request #, Payload 1, Payload 2, Status, Length, Time
- Interesting responses: magenta left border (5px)
- Status anomalies: orange background (subtle, 10% opacity)
- Some rows highlighted in yellow for interesting responses

ACTION BUTTON:
- "Start Attack" button: hexagonal shape, cyan background, black bold text
- Hover: cyan glow effect

STYLE:
- Hexagonal UI elements
- Circuit board accents
- Professional penetration testing tool interface
- Cyberpunk aesthetic
```

---

## 📸 Prompt 4: Request Repeater Tab

```
Int3rceptor Request Repeater tab for manual HTTP request modification and replay.

COLORS (EXACT):
- Background: black #0a0a0f
- Accents: cyan #00d4ff, magenta #ff006e, orange #ffb800

HEADER:
- Int3rceptor hexagonal logo (top left)
- Title: "Repeater" in cyan

LAYOUT: Split-screen 50/50 (Request | Response)

LEFT PANEL - REQUEST EDITOR:
- Method dropdown: GET selected (cyan when open)
- URL input field: full width, cyan border on focus
- Headers section (collapsible accordion):
  - Key-value pairs in table format
  - Keys: cyan text
  - Values: white text
  - Add header: cyan + button
- Body editor:
  - JSON syntax highlighting
  - Keys: cyan
  - Strings: magenta
  - Numbers: orange
  - Booleans: purple
- "Send" button at bottom: hexagonal shape, cyan background, black text

RIGHT PANEL - RESPONSE VIEWER:
- Status badge (top right): "200 OK" in cyan
- Response headers: collapsed accordion (cyan)
- Response body:
  - Pretty-printed JSON
  - Same syntax highlighting as request
  - Auto-detect format (JSON/XML/HTML)
- Toggle buttons (top): "Raw" | "Formatted"
  - Active: cyan background, black text
  - Inactive: transparent with cyan border

DIVIDER:
- Vertical line between panels: dark cyan #2a2a3e
- Draggable handle (cyan on hover)

STYLE:
- Hexagonal UI elements
- Monospace font for code sections
- Professional security tool interface
- Clean, modern cyberpunk aesthetic
```

---

## 📸 Prompt 5: WebSocket Interception Tab

```
Int3rceptor WebSocket interception tab showing real-time bidirectional frame capture.

COLORS (EXACT):
- Background: black #0a0a0f
- Cyan: #00d4ff
- Magenta: #ff006e
- Orange: #ffb800

HEADER:
- Int3rceptor hexagonal logo (top left)
- Title: "WebSocket" in cyan

TOP SECTION - CONNECTIONS LIST (200px height):
- Background: dark gray #1a1a2e
- Active WebSocket connections with:
  - URL in cyan
  - Status badge: "Connected" (cyan) or "Closed" (gray)
- Selected connection: cyan left border

MAIN AREA - FRAME TIMELINE:
- Bidirectional message flow visualization
- Client → Server frames:
  - Cyan left border (5px)
  - Cyan arrow icon →
  - Background: very dark gray
- Server → Client frames:
  - Magenta left border (5px)
  - Magenta arrow icon ←
  - Background: very dark gray

FRAME DETAILS (each row):
- Timestamp: gray text, small font (e.g., "14:23:45.123")
- Direction icon: cyan or magenta
- Frame type badge: orange background
  - Types: Text, Binary, Ping, Pong, Close
- Payload size: orange text (e.g., "1.2 KB")
- Content preview: truncated, monospace font, first 50 chars

EXPANDED FRAME (when selected):
- Full payload display
- JSON syntax highlighting:
  - Keys: cyan
  - Strings: magenta
  - Numbers: orange
  - Booleans: purple

RIGHT PANEL - METADATA (300px width):
- Background: dark gray #1a1a2e
- Connection ID: cyan label, white value
- Duration: orange text
- Stats:
  - "Frames Sent: 234" (cyan)
  - "Frames Received: 189" (magenta)
- Frame types breakdown: small pie chart (cyan/magenta/orange slices)

CONTROLS (bottom):
- "Pause Capture" button: orange background, black text
- "Clear" button: magenta border, transparent
- "Export" button: cyan border, transparent

STYLE:
- Hexagonal UI elements
- Circuit board patterns (subtle)
- Professional WebSocket debugging tool
- Cyberpunk aesthetic
- Monospace font for frame content
```

---

## 📸 Prompt 6: Rules & Scope Tab (Bonus)

```
Int3rceptor Rules and Scope configuration tab for traffic filtering and automation.

COLORS (EXACT):
- Background: black #0a0a0f
- Cyan: #00d4ff
- Magenta: #ff006e
- Orange: #ffb800

HEADER:
- Int3rceptor hexagonal logo (top left)
- Title: "Rules & Scope" in cyan

LAYOUT: Split-screen (Rules | Scope)

LEFT PANEL - RULES (60% width):
- Section header: "Automation Rules" (cyan)
- Rule list items:
  - Cyan checkbox (checked/unchecked)
  - Rule name in white (e.g., "Replace User-Agent")
  - Condition preview in gray (e.g., "If URL contains 'api'")
  - Edit icon: cyan
  - Delete icon: magenta
- "Add Rule" button: cyan + icon, transparent background

RIGHT PANEL - SCOPE (40% width):
- Two sections stacked vertically:

1. INCLUDE PATTERNS:
   - Header: "Include Patterns" (cyan)
   - URL pattern list:
     - *.example.com
     - https://api.target.com/*
     - Each with magenta × remove button
   - "Add Pattern" button: cyan

2. EXCLUDE PATTERNS:
   - Header: "Exclude Patterns" (magenta)
   - URL pattern list:
     - *.google.com
     - */analytics/*
     - Each with magenta × remove button
   - "Add Pattern" button: magenta border

TOGGLE SWITCHES:
- "Enable Rules": cyan when active, gray when inactive
- "Strict Scope": cyan when active, gray when inactive

BOTTOM STATS BAR:
- "Active Rules: 5" (cyan)
- "Include Patterns: 8" (cyan)
- "Exclude Patterns: 12" (magenta)

STYLE:
- Hexagonal design elements
- Circuit board accents (subtle)
- Professional security tool interface
- Clean, modern cyberpunk aesthetic
```

---

## 🎯 Tips for Best Results

### For Midjourney:

-   Add `--ar 16:9` for widescreen mockups
-   Add `--style raw` for more literal interpretation
-   Add `--v 6` for latest version
-   Example: `[prompt] --ar 16:9 --style raw --v 6`

### For DALL-E 3:

-   Use "I NEED to test how the tool works with extremely simple prompts. DO NOT add any detail, just use it AS-IS."
-   Request "wide format" or "landscape orientation"
-   Emphasize "UI design mockup" and "interface screenshot"

### For Stable Diffusion:

-   Add negative prompt: `blurry, low quality, distorted, 3d render, realistic photo, gradients on background`
-   Use models: `SDXL` or `Realistic Vision`
-   CFG Scale: 7-9
-   Steps: 30-50

### General Tips:

1. **Upload the logo** as a reference image when possible
2. **Emphasize exact hex colors** in your prompt
3. **Request "UI mockup" or "interface design"** not "screenshot"
4. **Specify "flat design"** to avoid 3D effects
5. **Mention "dark theme"** explicitly
6. **Use "cyberpunk aesthetic"** for style consistency

---

## 📁 File Naming Convention

When you generate the images, save them as:

-   `dashboard.png` → Dashboard Overview
-   `traffic-tab.png` → Traffic Interception Tab
-   `intruder-tab.png` → Intruder/Fuzzer Tab
-   `repeater-tab.png` → Request Repeater Tab
-   `websocket-tab.png` → WebSocket Interception Tab
-   `rules-scope-tab.png` → Rules & Scope Tab (optional)

Then copy them to: `/media/il1v3y/HD2/HDfiles/shenanigans/Repos/S1BGr0up/proyects/personal/interceptor/assets/screenshots/`

---

## 🖼️ Logo Reference

Use this logo image as a reference for brand consistency:

-   Path: `/home/il1v3y/.gemini/antigravity/brain/ea9b7f2c-2347-422a-999f-b17402c14e13/uploaded_image_1763770017731.jpg`
-   Features: Hexagonal shape, circuit board patterns, cyan/magenta/orange colors
-   Text: "INT3RCEPTOR" in pink/white, "S1bGr0up.inc" in orange

---

**Version**: 1.0  
**Created**: 2025-11-22  
**Purpose**: Generate brand-consistent UI mockups for Int3rceptor documentation
