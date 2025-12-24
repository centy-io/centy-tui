# TUI: Get entity actions from daemon via gRPC

Replace hardcoded action panel items with dynamic actions fetched from the daemon's new GetEntityActions gRPC endpoint.

## Context
The daemon now provides a GetEntityActions endpoint that returns available actions (create, delete, duplicate, move, status changes, open in vscode) based on entity type and current state.

## Tasks
- [ ] Sync proto file from centy-daemon
- [ ] Add get_entity_actions method to DaemonClient
- [ ] Update action panel in issues.rs to use dynamic actions
- [ ] Update action panel for PRs and Docs similarly
- [ ] Implement missing actions: Delete, Duplicate, Move
- [ ] Use 'enabled' flag to gray out unavailable actions
- [ ] Use 'disabled_reason' for status line messages
- [ ] Use 'keyboard_shortcut' for keybindings
- [ ] Group actions by category in UI
