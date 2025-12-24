---
title: "Input Guidelines"
createdAt: "2025-12-24T22:32:01.998776+00:00"
updatedAt: "2025-12-24T22:32:01.998776+00:00"
---

# Input Guidelines

# Input Guidelines

## Mouse and Keyboard Support

Every functionality in the TUI must be accessible via both mouse and keyboard inputs.

### Requirements

1. **Keyboard First**: All features must have keyboard shortcuts
2. **Mouse Support**: All interactive elements should respond to mouse clicks
3. **Consistent Navigation**: Tab/Shift-Tab should work for focus navigation
4. **Visual Feedback**: Focused elements should have clear visual indicators

### Implementation Checklist

When adding new features, ensure:

- [ ] Keyboard shortcut is defined and documented in status bar hints
- [ ] Mouse click handlers are implemented for interactive elements
- [ ] Focus state is visually distinct (e.g., border color change)
- [ ] Navigation flows naturally with Tab key

### Examples

| Action | Keyboard | Mouse |
|--------|----------|-------|
| Select item | Enter | Click |
| Navigate | j/k or arrows | Scroll/Click |
| Switch focus | Tab | Click on panel |
| Execute action | Enter | Click button |
