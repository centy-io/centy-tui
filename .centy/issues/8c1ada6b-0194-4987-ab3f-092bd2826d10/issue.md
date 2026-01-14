# Add TUI support for standalone workspaces (no issue required)

Add TUI interface elements to create and manage standalone workspaces.

## Depends On

* centy-daemon: Support creating workspaces without an issue (standalone workspaces)

## Implementation

### 1. New Menu Option

Add ‘New Standalone Workspace’ option in the workspace menu that allows users to:

* Create a workspace without selecting an issue
* Enter optional name/description
* Configure TTL and agent

### 2. Workspace List Updates

Update the workspace list view to:

* Display standalone workspaces with a distinct indicator (e.g., \[standalone\] tag)
* Show workspace name/description for standalone workspaces
* Support filtering/grouping by workspace type

### 3. Keyboard Shortcuts

Add keyboard shortcut for quick standalone workspace creation (e.g., `n` for new workspace)

## Tasks

1. Add menu option for standalone workspace creation
1. Create input form for workspace name/description
1. Update workspace list rendering to show standalone workspaces
1. Add gRPC client call to `OpenStandaloneWorkspace` RPC
1. Update help/keybinding documentation
