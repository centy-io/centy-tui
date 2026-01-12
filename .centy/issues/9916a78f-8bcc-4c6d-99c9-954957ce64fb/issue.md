# Support GetSupportedEditors RPC and provide editor selection in UI

## Overview

The daemon now supports a `GetSupportedEditors` RPC that returns available workspace editors (VS Code, Terminal) with their availability status.

## Requirements

1. **Fetch available editors** - call `GetSupportedEditors` RPC on startup or when opening workspace
1. **Display editor options** - show available editors in the workspace open dialog/menu
1. **Visual indicators** - indicate which editors are available vs unavailable
1. **Selection handling** - allow user to select preferred editor for workspace operations
1. **Remember preference** - optionally remember last selected editor

## API Reference

````protobuf
rpc GetSupportedEditors(GetSupportedEditorsRequest) returns (GetSupportedEditorsResponse);

message EditorInfo {
  EditorType editor_type = 1;  // VSCODE or TERMINAL
  string name = 2;             // "VS Code" or "Terminal"
  string description = 3;      // Brief description
  bool available = 4;          // Whether available on system
}
````

## UI/UX Considerations

* Show editor selector when user triggers “Open in Workspace” action
* Gray out or hide unavailable editors
* Show editor description as tooltip or secondary text
* Consider keyboard shortcuts for quick editor selection (v for VS Code, t for Terminal)

## Acceptance Criteria

* [ ] Editor selection UI appears before opening workspace
* [ ] Only available editors are selectable
* [ ] VS Code and Terminal options display correctly
* [ ] Selected editor is used for `OpenInTempVscode` or `OpenInTempTerminal` RPC
