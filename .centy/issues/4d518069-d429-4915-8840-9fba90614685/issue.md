# Refactor sidebar into shared component with reusable vertical centering

Extract the sidebar into a shared component with reusable logic for vertically centering buttons in the middle of the screen.

## Current Problem

The vertical centering logic (using Constraint::Min(0) for top/bottom flex padding) is duplicated between:

* Sidebar in layout.rs (lines 68-120)
* Action panel in mod.rs

## Proposed Solution

1. Create a shared ‘VerticalButtonGroup’ component that handles:
   
   * Vertical centering of buttons in available space
   * Dynamic number of buttons
   * Consistent button spacing
   * Mouse click detection for button selection
1. Extract sidebar into its own module (src/ui/sidebar/)

1. Refactor action panel to use the shared component

## Files to modify

* centy-tui/src/ui/layout.rs - Extract sidebar logic
* centy-tui/src/ui/components/button.rs - May need updates
* centy-tui/src/ui/mod.rs - Action panel uses shared component
* New: centy-tui/src/ui/components/vertical_button_group.rs
