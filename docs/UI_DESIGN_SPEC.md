# Int3rceptor UI Design Specifications

## Brand Identity

### Logo

The Int3rceptor logo features a hexagonal design with circuit board patterns, representing network interception and cybersecurity.

### Color Palette

| Color                   | Hex Code  | Usage                                  | RGB                |
| ----------------------- | --------- | -------------------------------------- | ------------------ |
| **Background**          | `#0a0a0f` | Main background, dark areas            | rgb(10, 10, 15)    |
| **Background Alt**      | `#1a1a2e` | Secondary background, panels           | rgb(26, 26, 46)    |
| **Cyan (Primary)**      | `#00d4ff` | Primary accent, links, active states   | rgb(0, 212, 255)   |
| **Magenta (Secondary)** | `#ff006e` | Secondary accent, highlights, warnings | rgb(255, 0, 110)   |
| **Orange (Tertiary)**   | `#ffb800` | Tertiary accent, warnings, stats       | rgb(255, 184, 0)   |
| **Purple**              | `#8b5cf6` | Gradients, subtle accents              | rgb(139, 92, 246)  |
| **Text Primary**        | `#ffffff` | Main text                              | rgb(255, 255, 255) |
| **Text Secondary**      | `#a0a0a0` | Secondary text, labels                 | rgb(160, 160, 160) |
| **Text Muted**          | `#606060` | Disabled, placeholders                 | rgb(96, 96, 96)    |

### Typography

-   **Code/Monospace**: `'Fira Code', 'JetBrains Mono', 'Consolas', monospace`
-   **UI Text**: `'Inter', 'Roboto', 'Segoe UI', sans-serif`
-   **Headings**: `'Orbitron', 'Rajdhani', sans-serif` (cyberpunk aesthetic)

### UI Elements

#### Buttons

-   **Primary**: Cyan background `#00d4ff`, black text, hexagonal shape
-   **Secondary**: Transparent with cyan border, cyan text
-   **Danger**: Magenta background `#ff006e`, white text
-   **Success**: Orange background `#ffb800`, black text

#### Badges

-   **Status 200-299**: Cyan `#00d4ff`
-   **Status 300-399**: Purple `#8b5cf6`
-   **Status 400-499**: Magenta `#ff006e`
-   **Status 500-599**: Orange-red `#ff4500`
-   **Method GET**: Cyan `#00d4ff`
-   **Method POST**: Magenta `#ff006e`
-   **Method PUT**: Orange `#ffb800`
-   **Method DELETE**: Red `#ff0000`

#### Borders & Dividers

-   **Default**: `#2a2a3e` (subtle dark)
-   **Active/Focus**: Cyan `#00d4ff`
-   **Highlight**: Magenta `#ff006e`

#### Shadows & Glows

-   **Cyan glow**: `0 0 10px rgba(0, 212, 255, 0.3)`
-   **Magenta glow**: `0 0 10px rgba(255, 0, 110, 0.3)`
-   **Elevation shadow**: `0 4px 12px rgba(0, 0, 0, 0.5)`

---

## Component Specifications

### 1. Dashboard Overview

**Layout**: Header + Sidebar + Main Content + Details Panel

**Header** (60px height):

-   Background: `#1a1a2e`
-   Logo: Left aligned, 40px height
-   Navigation tabs: Cyan text, active tab has cyan bottom border (3px)
-   Metrics bar: Right aligned
    -   "Requests: X" in cyan
    -   "Memory: X MB" in orange
    -   "Connections: X" in magenta

**Main Content**:

-   Background: `#0a0a0f`
-   Table headers: Cyan text `#00d4ff`, 14px font
-   Rows: Alternating `#0a0a0f` and `#1a1a2e`
-   Hover: Cyan left border (3px)
-   Selected: Cyan left border (5px) + background `#1a2a3e`

**Details Panel** (Right sidebar, 400px width):

-   Background: `#1a1a2e`
-   Section headers: Cyan text
-   Syntax highlighting:
    -   Keys: Cyan `#00d4ff`
    -   Strings: Magenta `#ff006e`
    -   Numbers: Orange `#ffb800`
    -   Booleans: Purple `#8b5cf6`
    -   Null: Gray `#606060`

**Hexagonal Elements**:

-   Subtle hexagon patterns in background (opacity 0.05)
-   Circuit board lines connecting elements (cyan, very subtle)

---

### 2. Traffic Tab

**Search Bar**:

-   Background: `#1a1a2e`
-   Border: `#2a2a3e`, focus: cyan `#00d4ff`
-   Placeholder: Gray `#606060`
-   Icon: Cyan `#00d4ff`

**Table**:

-   Columns: Method | URL | Status | Size | Time
-   Method badges: Pill-shaped, colored by method type
-   Status codes: Colored by range (see badges section)
-   URL: Truncated with ellipsis, full URL on hover tooltip
-   Size: Orange text `#ffb800`
-   Time: Gray text `#a0a0a0`

**Request Details Panel**:

-   Tabs: Headers | Body | Raw
-   Active tab: Cyan underline
-   Headers: Key-value pairs, keys in cyan
-   Body: Syntax-highlighted JSON/XML/HTML
-   Raw: Monospace font, white text

**Filters**:

-   Filter chips: Cyan border, transparent background
-   Active filter: Cyan background, black text
-   Remove icon: Magenta `#ff006e`

---

### 3. Intruder/Fuzzer Tab

**Request Template**:

-   Background: `#1a1a2e`
-   Payload markers `§param§`: Magenta background `#ff006e`, black text
-   Monospace font
-   Line numbers: Gray `#606060`

**Attack Type Dropdown**:

-   Options: Sniper, Battering Ram, Pitchfork, Cluster Bomb
-   Selected: Cyan background `#00d4ff`, black text
-   Hover: Cyan border

**Payload Lists**:

-   Headers: "Payload Set 1", "Payload Set 2" in cyan
-   Items: White text on dark background
-   Add button: Cyan `+` icon
-   Remove button: Magenta `×` icon

**Results Table**:

-   Columns: Request # | Payload 1 | Payload 2 | Status | Length | Time
-   Interesting responses: Magenta left border (5px)
-   Status anomalies: Orange background (subtle, 10% opacity)

**Start Attack Button**:

-   Hexagonal shape
-   Cyan background `#00d4ff`
-   Black text, bold
-   Hover: Cyan glow effect

---

### 4. Repeater Tab

**Split Layout**: 50/50 Request | Response

**Request Panel** (Left):

-   Method dropdown: Cyan when open
-   URL input: Full width, cyan border on focus
-   Headers section: Collapsible accordion
    -   Key input: Cyan placeholder
    -   Value input: White text
    -   Add header: Cyan `+` button
-   Body editor: Syntax-highlighted
-   Send button: Cyan hexagonal button at bottom

**Response Panel** (Right):

-   Status badge: Top right, colored by status code
-   Headers: Collapsed by default, cyan accordion
-   Body: Syntax-highlighted, auto-detect format
-   Toggle buttons: Raw | Formatted
    -   Active: Cyan background
    -   Inactive: Transparent with cyan border

**Divider**:

-   Vertical line: `#2a2a3e`
-   Draggable handle: Cyan on hover

---

### 5. WebSocket Tab

**Connections List** (Top, 200px height):

-   Background: `#1a1a2e`
-   Each connection: URL in cyan, status badge
-   Connected: Cyan badge
-   Closed: Gray badge
-   Selected: Cyan left border

**Frame Timeline** (Main area):

-   Client→Server frames:
    -   Left border: Cyan `#00d4ff` (5px)
    -   Arrow icon: Cyan →
-   Server→Client frames:
    -   Left border: Magenta `#ff006e` (5px)
    -   Arrow icon: Magenta ←

**Frame Details**:

-   Timestamp: Gray `#a0a0a0`, small font
-   Frame type badge: Orange `#ffb800`
    -   Text, Binary, Ping, Pong, Close
-   Payload size: Orange text
-   Content preview: Truncated, monospace

**Expanded Frame**:

-   Full payload: Syntax-highlighted JSON
-   Metadata: Cyan labels, white values

**Metadata Panel** (Right, 300px):

-   Connection ID: Cyan
-   Duration: Orange
-   Frames sent: Cyan counter
-   Frames received: Magenta counter
-   Frame types breakdown: Pie chart (cyan/magenta/orange)

**Controls**:

-   Pause/Resume: Orange button
-   Clear: Magenta button
-   Export: Cyan button

---

## Responsive Design

### Breakpoints

-   Desktop: 1920px+ (default)
-   Laptop: 1366px - 1919px (scale UI 90%)
-   Tablet: 768px - 1365px (stack panels vertically)
-   Mobile: < 768px (not primary target, but should be usable)

### Scaling

-   Font sizes: Use `rem` units
-   Spacing: Use `em` or `rem` for consistency
-   Icons: SVG for crisp rendering at any size

---

## Accessibility

### Contrast Ratios

-   Cyan on black: 9.5:1 (AAA)
-   Magenta on black: 7.2:1 (AA)
-   Orange on black: 8.1:1 (AAA)
-   White on black: 21:1 (AAA)

### Focus States

-   All interactive elements: Cyan outline (2px)
-   Keyboard navigation: Visible focus indicators
-   Skip links: For screen readers

### ARIA Labels

-   All icons: `aria-label` attributes
-   Tables: Proper `<th>` headers
-   Forms: Associated `<label>` elements

---

## Animation & Transitions

### Durations

-   Instant: 0ms (state changes)
-   Fast: 150ms (hover, focus)
-   Normal: 300ms (panel open/close)
-   Slow: 500ms (page transitions)

### Easing

-   Standard: `cubic-bezier(0.4, 0.0, 0.2, 1)`
-   Decelerate: `cubic-bezier(0.0, 0.0, 0.2, 1)`
-   Accelerate: `cubic-bezier(0.4, 0.0, 1, 1)`

### Effects

-   Hover: Subtle glow (cyan/magenta)
-   Click: Scale down to 0.95, then back
-   Panel slide: Translate with easing
-   Fade: Opacity transition

---

## Implementation Notes

### Vue.js Components

-   Use Composition API
-   TypeScript for type safety
-   Scoped styles with CSS modules
-   Tailwind CSS for utility classes (configure custom colors)

### CSS Variables

```css
:root {
    --color-bg-primary: #0a0a0f;
    --color-bg-secondary: #1a1a2e;
    --color-accent-cyan: #00d4ff;
    --color-accent-magenta: #ff006e;
    --color-accent-orange: #ffb800;
    --color-accent-purple: #8b5cf6;
    --color-text-primary: #ffffff;
    --color-text-secondary: #a0a0a0;
    --color-text-muted: #606060;
    --font-mono: "Fira Code", monospace;
    --font-ui: "Inter", sans-serif;
    --font-heading: "Orbitron", sans-serif;
}
```

### Dark Mode Only

-   Int3rceptor is a security tool used in low-light environments
-   No light mode needed
-   Optimize for extended viewing sessions (reduce eye strain)

---

## Assets Needed

### Icons

-   Use [Heroicons](https://heroicons.com/) or [Lucide](https://lucide.dev/)
-   Style: Outline for inactive, solid for active
-   Color: Inherit from parent (use CSS `currentColor`)

### Illustrations

-   Circuit board patterns (SVG)
-   Hexagonal grid backgrounds (SVG)
-   Loading animations (Lottie or CSS)

### Logo Variations

-   Full logo (with text)
-   Icon only (hexagon)
-   Monochrome version (for favicons)

---

## Performance Considerations

### Optimization

-   Lazy load components (Vue async components)
-   Virtual scrolling for large request lists (use `vue-virtual-scroller`)
-   Debounce search inputs (300ms)
-   Throttle scroll events (16ms, 60fps)

### Bundle Size

-   Code splitting by route
-   Tree-shake unused Tailwind classes
-   Compress images (WebP format)
-   Minify CSS and JS

---

## Browser Support

### Minimum Versions

-   Chrome 90+
-   Firefox 88+
-   Edge 90+
-   Safari 14+ (macOS only)

### Features Required

-   CSS Grid
-   CSS Custom Properties
-   ES2020 JavaScript
-   WebSocket API
-   Fetch API

---

## Design System Tools

### Recommended

-   **Figma**: For design mockups and prototypes
-   **Storybook**: For component library documentation
-   **Chromatic**: For visual regression testing

### Design Tokens

-   Export color palette as JSON
-   Use in both design tools and code
-   Maintain single source of truth

---

**Version**: 1.0  
**Last Updated**: 2025-11-22  
**Maintained by**: S1BGr0uP Design Team
