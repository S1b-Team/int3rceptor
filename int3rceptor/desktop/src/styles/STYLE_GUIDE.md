# INT3RCEPTOR UI Style Guide

## Overview

This document defines the visual design system and guidelines for the INT3RCEPTOR interface, ensuring consistency across all views and components.

## Color Palette

### Primary Colors

| Color | Hex | Usage | Contrast Ratio |
|--------|------|--------|---------------|
| Cyan | `#00d4ff` | Primary actions, active states, success indicators | 9.5:1 |
| Magenta | `#ff006e` | Secondary actions, warnings, important alerts | 7.2:1 |
| Orange | `#ffb800` | Highlights, pending states, client-to-server | 8.1:1 |
| Purple | `#8b5cf6` | Tertiary actions, special features, ping frames | 6.8:1 |

### Semantic Colors

| Color | Hex | Usage |
|--------|------|--------|
| Green | `#22c55e` | Success, additions, pong frames |
| Red | `#ef4444` | Error, deletions, critical issues |
| Gray | `#6b7280` | Neutral, disabled states |

### Background Colors

| Color | Hex | Usage |
|--------|------|--------|
| Primary BG | `#0a0a0f` | Main background |
| Secondary BG | `#12121a` | Cards, panels |
| Tertiary BG | `#1a1a24` | Inputs, dropdowns |
| Hover BG | `#1e1e2a` | Hover states |

## Typography

### Font Families

| Usage | Font Family | Weight |
|--------|--------------|--------|
| Headings | `Orbitron` | 700 |
| UI Text | `Inter` | 400, 500, 600 |
| Monospace | `Fira Code` | 400 |

### Typography Scale

| Token | Value | Usage |
|--------|--------|--------|
| `--text-xs` | 0.75rem (12px) | Labels, badges |
| `--text-sm` | 0.875rem (14px) | Body text, buttons |
| `--text-base` | 1rem (16px) | Default text |
| `--text-lg` | 1.125rem (18px) | Subheadings |
| `--text-xl` | 1.25rem (20px) | Section headers |
| `--text-2xl` | 1.5rem (24px) | Page headers |
| `--text-3xl` | 1.875rem (30px) | Hero text |

### Typography Hierarchy

| Level | Size | Weight | Usage |
|--------|-------|--------|--------|
| H1 | `--text-3xl` (30px) | 700 | Page titles, main headers |
| H2 | `--text-2xl` (24px) | 700 | Section headers, card titles |
| H3 | `--text-xl` (20px) | 600 | Subsection headers |
| H4 | `--text-lg` (18px) | 600 | Component headers |
| Body | `--text-sm` (14px) | 400 | Default content text |
| Caption | `--text-xs` (12px) | 400 | Labels, metadata |

## Spacing System

| Token | Value | Usage |
|--------|--------|--------|
| `--spacing-xs` | 4px | Tight spacing, icon gaps |
| `--spacing-sm` | 8px | Small gaps, padding inside components |
| `--spacing-md` | 16px | Default spacing, component padding |
| `--spacing-lg` | 20px | Large gaps, section spacing |
| `--spacing-xl` | 24px | Extra large gaps, section margins |
| `--spacing-2xl` | 32px | Very large gaps, page margins |
| `--spacing-3xl` | 48px | Hero sections, major spacing |

## Border Radius

| Token | Value | Usage |
|--------|--------|--------|
| `--radius-sm` | 4px | Small elements, badges |
| `--radius-md` | 8px | Default elements, inputs |
| `--radius-lg` | 12px | Cards, panels |
| `--radius-xl` | 16px | Large cards, modals |

## Shadows

| Token | Value | Usage |
|--------|--------|--------|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.1)` | Small elements |
| `--shadow-md` | `0 4px 6px rgba(0,0,0,0.15)` | Cards, panels |
| `--shadow-lg` | `0 10px 15px rgba(0,0,0,0.2)` | Modals, dropdowns |
| `--shadow-xl` | `0 20px 25px rgba(0,0,0,0.25)` | Hero elements |

### Glow Effects

| Token | Value | Usage |
|--------|--------|--------|
| `--glow-cyan` | `0 0 10px rgba(0,212,255,0.5)` | Cyan accents, active states |
| `--glow-magenta` | `0 0 10px rgba(255,0,110,0.5)` | Magenta accents, warnings |
| `--glow-orange` | `0 0 10px rgba(255,184,0,0.5)` | Orange accents, highlights |
| `--glow-purple` | `0 0 10px rgba(139,92,246,0.5)` | Purple accents, special features |

## Animations

### Durations

| Token | Value | Usage |
|--------|--------|--------|
| `--duration-fast` | 150ms | Micro-interactions, hover states |
| `--duration-normal` | 300ms | Default transitions |
| `--duration-slow` | 500ms | Complex animations, page transitions |

### Easing

| Token | Value | Usage |
|--------|--------|--------|
| `--ease-out` | `cubic-bezier(0.4, 0, 0.2, 1)` | Default easing |
| `--ease-in-out` | `cubic-bezier(0.4, 0, 0.2, 1)` | Smooth transitions |

### Animation Guidelines

- Keep animations subtle and non-intrusive
- Use `--duration-fast` (150ms) for hover states
- Use `--duration-normal` (300ms) for transitions
- Avoid animations longer than `--duration-slow` (500ms)
- Use `--ease-out` for most transitions
- Provide `prefers-reduced-motion` support

## Components

### Buttons

**Primary Button**
- Background: Cyan gradient
- Text: White
- Shadow: Glow cyan
- Hover: Brighter cyan
- Active: Slightly darker cyan

**Secondary Button**
- Background: Tertiary BG
- Border: Primary border
- Text: Primary text
- Hover: Hover BG
- Active: Secondary BG

**Icon Button**
- Background: Transparent
- Text: Primary text
- Hover: Cyan/10
- Active: Cyan/20

### Inputs

**Text Input**
- Background: Tertiary BG
- Border: Primary border
- Text: Primary text
- Placeholder: Muted text
- Focus: Cyan border with ring
- Error: Red border with ring

**Textarea**
- Background: Tertiary BG
- Border: Primary border
- Text: Primary text
- Focus: Cyan border with ring
- Resize: None (use auto-expand)

### Cards

**Default Card**
- Background: Secondary BG
- Border: Primary border
- Radius: `--radius-lg` (12px)
- Shadow: `--shadow-md`
- Hover: `--shadow-lg` with glow

**Feature Card**
- Background: Secondary BG
- Border: Primary border
- Radius: `--radius-lg` (12px)
- Icon: Large, color-coded
- Hover: Glow effect, scale 1.02

**Metric Card**
- Background: Secondary BG
- Border: Primary border
- Radius: `--radius-lg` (12px)
- Value: Large, bold
- Trend: Color-coded arrow

### Badges

**Default Badge**
- Background: Color/10
- Border: Color/20
- Text: Color
- Radius: `--radius-sm` (4px)
- Padding: `--spacing-xs` `--spacing-sm`

**Status Badge**
- Cyan: Active, success
- Magenta: Warning, important
- Orange: Pending, client
- Purple: Special, tertiary
- Green: Success, additions
- Red: Error, deletions
- Gray: Disabled, neutral

### Tables

**Table Header**
- Background: Tertiary BG
- Border: Primary border
- Text: Muted text
- Sticky: Top

**Table Row**
- Background: Transparent
- Border: Primary border/30
- Hover: Cyan/5
- Selected: Cyan/10 with left border

**Table Cell**
- Padding: `--spacing-sm`
- Text: Primary text
- Alignment: Left (default)

## Layout Patterns

### Two-Column Layout

**Usage:** Traffic, Scanner, Settings, Comparer

- Left Panel: 40% width (adjustable)
- Right Panel: 60% width (adjustable)
- Divider: Draggable
- Collapse: Both panels collapsible

### Split View

**Usage:** Repeater

- Orientation: Horizontal (default), Vertical
- Divider: Draggable
- Min Width: 200px per panel
- Default Split: 50/50

### Grid Layout

**Usage:** Dashboard, Plugins, Intruder

- Columns: 1 (mobile), 2 (tablet), 3-4 (desktop)
- Gap: `--spacing-md` (16px)
- Responsive: Auto-fit

## Icon System

### Sizes

| Token | Value | Usage |
|--------|--------|--------|
| `--icon-xs` | 12px | Badges, inline icons |
| `--icon-sm` | 16px | Buttons, labels |
| `--icon-md` | 20px | Headers, cards |
| `--icon-lg` | 24px | Section headers |
| `--icon-xl` | 32px | Hero elements |
| `--icon-2xl` | 48px | Large displays |

### Colors

| Color | Usage |
|--------|--------|
| Cyan | Primary actions, active states |
| Magenta | Secondary actions, warnings |
| Orange | Highlights, pending states |
| Purple | Tertiary actions, special |
| Green | Success, positive |
| Red | Error, negative |
| Muted | Disabled, neutral |

## Accessibility

### Color Contrast

- Minimum contrast ratio: 4.5:1 (WCAG AA)
- Target contrast ratio: 7:1 (WCAG AAA)
- All primary colors meet WCAG AA
- Cyan: 9.5:1 (AAA)
- Magenta: 7.2:1 (AAA)
- Orange: 8.1:1 (AAA)
- Purple: 6.8:1 (AA)

### Focus States

- All interactive elements must have visible focus states
- Focus ring: 2px, color-matched to element
- Focus outline: 2px solid, offset 2px
- Skip to main content link available

### Keyboard Navigation

- All interactive elements keyboard accessible
- Tab order logical and consistent
- Escape closes modals/dropdowns
- Enter/Space activates buttons

### Screen Readers

- All icons have aria-label
- All form elements have associated labels
- Dynamic content updates announced
- Live regions for status updates

### Reduced Motion

- Respect `prefers-reduced-motion` preference
- Provide option to disable animations
- No essential functionality requires animation

## Responsive Design

### Breakpoints

| Breakpoint | Value | Usage |
|-------------|--------|--------|
| `--breakpoint-sm` | 640px | Mobile |
| `--breakpoint-md` | 768px | Tablet |
| `--breakpoint-lg` | 1024px | Desktop |
| `--breakpoint-xl` | 1280px | Large Desktop |
| `--breakpoint-2xl` | 1536px | Extra Large |

### Mobile (< 768px)

- Single column layouts
- Stacked grids
- Reduced padding
- Simplified navigation
- Touch-friendly targets (min 44px)

### Tablet (768px - 1024px)

- Two column layouts where appropriate
- Responsive grids (2 columns)
- Maintained spacing
- Optimized for touch

### Desktop (> 1024px)

- Full multi-column layouts
- Responsive grids (3-4 columns)
- Full spacing
- Mouse-optimized interactions

## Best Practices

### General

1. **Consistency**: Use design tokens, not hardcoded values
2. **Hierarchy**: Clear visual hierarchy through size, weight, color
3. **Contrast**: Ensure sufficient contrast for accessibility
4. **Spacing**: Use spacing system for consistency
5. **Animation**: Keep animations subtle and purposeful
6. **Feedback**: Provide clear feedback for all interactions
7. **Accessibility**: Design for all users, all abilities

### Performance

1. Minimize reflows and repaints
2. Use CSS transforms for animations
3. Implement virtual scrolling for large lists
4. Lazy load images and content
5. Optimize font loading
6. Use efficient selectors

### Code Organization

1. Use composition API for components
2. Extract reusable logic into composables
3. Use TypeScript for type safety
4. Follow Vue 3 best practices
5. Document complex components
6. Maintain consistent file structure

## Testing Checklist

### Visual Regression

- [ ] All views match approved mockups
- [ ] Color schemes consistent across views
- [ ] Typography hierarchy correct
- [ ] Spacing consistent
- [ ] Icons correct size and color
- [ ] Animations subtle and smooth

### Accessibility

- [ ] All color contrasts meet WCAG AA
- [ ] All interactive elements keyboard accessible
- [ ] All form elements have labels
- [ ] Focus states visible
- [ ] Screen reader announcements present
- [ ] Reduced motion respected

### Responsiveness

- [ ] Mobile layout (< 768px)
- [ ] Tablet layout (768px - 1024px)
- [ ] Desktop layout (> 1024px)
- [ ] Touch targets adequate (min 44px)
- [ ] No horizontal scroll

### Cross-Browser

- [ ] Chrome (latest)
- [ ] Firefox (latest)
- [ ] Edge (latest)
- [ ] Safari (latest)

### Performance

- [ ] Load time < 2s
- [ ] First contentful paint < 1s
- [ ] Time to interactive < 3s
- [ ] No layout shifts
- [ ] Smooth animations (60fps)

## Resources

- Design Tokens: `int3rceptor/desktop/src/styles/tokens.css`
- Icon System: `int3rceptor/desktop/src/components/shared/Icon.vue`
- Shared Components: `int3rceptor/desktop/src/components/shared/`
- Tailwind Config: `int3rceptor/desktop/tailwind.config.js`
