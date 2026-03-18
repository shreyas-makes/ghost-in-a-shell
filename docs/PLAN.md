# Ghost-in-a-Shell v0 Implementation Plan

## Summary

v0 is a **macOS-first Rust CLI/TUI** that lets a user:

- take a snapshot of the current Ghostty state
- save that snapshot with a name
- list and switch between named snapshots
- restore a named snapshot after interruption
- see which terminals were restored versus which need rerunning

v0 does **not** attempt exact process resurrection. It restores recognizable Ghostty structure and enough context to continue.

Default v0 product decisions:

- snapshots are **manually saved**
- users can have **multiple named snapshots**
- primary UI is an **in-terminal TUI**, with CLI commands as the underlying interface
- macOS Ghostty orchestration is done through **AppleScript automation**

## Product Surface

The binary should be a single Rust executable named `ghost-in-a-shell`.

Primary commands:

- `ghost-in-a-shell tui`
  Opens the main TUI with `Save current setup`, `Restore`, `Switch`, `Rename`, and `Delete`.
- `ghost-in-a-shell save <name>`
  Captures the current Ghostty layout and stores it as a named snapshot.
- `ghost-in-a-shell list`
  Lists saved snapshots sorted by most recently updated.
- `ghost-in-a-shell restore <name>`
  Rebuilds the named snapshot inside Ghostty.
- `ghost-in-a-shell switch <name>`
  Alias of restore-oriented behavior for user language; it restores or focuses the named snapshot.
- `ghost-in-a-shell rename <old> <new>`
  Renames a saved snapshot.
- `ghost-in-a-shell delete <name>`
  Removes a saved snapshot.

TUI behavior:

- Default view is a searchable list of named snapshots with `name`, `updated_at`, terminal count, and a one-line summary.
- Selecting a snapshot opens a preview pane showing windows, tabs, panes, terminal labels, remembered locations, and last intended commands.
- Restore flow shows per-terminal result states: `restored` or `needs rerun`.
- First-run empty state explains the core model in plain language: “Save the Ghostty setup you want to bring back later.”

## Implementation Changes

### 1. Rust app structure

Create a single Rust crate with these modules:

- `cli`
  Clap command parsing and top-level command dispatch.
- `tui`
  `ratatui` + `crossterm` application for list, preview, save, restore, rename, and delete flows.
- `model`
  Serializable `WorkspaceSnapshot`, `TerminalSnapshot`, `WindowSnapshot`, `TabSnapshot`, `PaneSnapshot`, and status enums.
- `store`
  Snapshot persistence, slug validation, overwrite rules, and listing/sorting.
- `ghostty_adapter`
  AppleScript-backed capture, restore, and focus logic.
- `recovery`
  Computes restore results and maps adapter/runtime outcomes into `restored` / `needs_rerun`.

Recommended crates:

- `clap`
- `ratatui`
- `crossterm`
- `serde`, `serde_json`
- `uuid`
- `chrono`
- `camino`
- `thiserror`
- `tracing`, `tracing-subscriber`

### 2. Snapshot model

Persist one JSON file per named snapshot at:

- `~/Library/Application Support/ghost-in-a-shell/workspaces/<slug>.json`

Top-level schema:

- `workspace_id`
- `name`
- `slug`
- `created_at`
- `updated_at`
- `source = "manual_save"`
- `windows[]`
- `terminals[]`

`WindowSnapshot` fields:

- `window_index`
- `tabs[]`

`TabSnapshot` fields:

- `tab_index`
- `title`
- `panes[]`

`PaneSnapshot` fields:

- `pane_index`
- `terminal_id`
- `layout_slot`

`TerminalSnapshot` fields:

- `terminal_id`
- `label`
- `role`
- `working_directory`
- `surface_title`
- `launch_intent`
- `restore_status`
- `last_seen_at`

`launch_intent` policy for v0:

- store a string command only when the user explicitly provides it during save or later edit
- if no command is known, restore the shell surface and mark the terminal `needs_rerun`

Label policy for v0:

- default label order is `user label` → `surface title` → `terminal <n>`

### 3. Capture flow

`save <name>` and the TUI save action should:

- query current Ghostty windows, tabs, and panes through AppleScript
- capture titles and working-directory information where Ghostty exposes it
- build a normalized logical layout independent of Ghostty object IDs
- prompt in the TUI for optional labels and optional launch commands for each terminal
- write the snapshot file atomically

Overwrite behavior:

- CLI requires `--force` to overwrite an existing name
- TUI prompts for confirmation

### 4. Restore and switch flow

Restore behavior for `restore <name>` and `switch <name>`:

- load the named snapshot
- create a new Ghostty window/tab/pane arrangement that matches the saved logical layout
- apply titles/labels where possible
- open terminals in the remembered working directory
- if a terminal has explicit `launch_intent`, offer rerun in the restore summary before executing it
- if no launch intent exists, restore the shell surface only and mark it `needs rerun`

v0 restore rule:

- do **not** attempt to merge with arbitrary existing Ghostty state
- restoring a snapshot always materializes a fresh copy of that saved setup
- switching means “restore this named setup now,” not “diff my current setup into it”

Recovery summary:

- after restore, print or render a summary showing each terminal as `restored` or `needs rerun`
- if a rerunnable command exists, expose a `Run now` action in the TUI

### 5. Ghostty macOS adapter

Adapter responsibilities:

- enumerate current Ghostty windows
- enumerate tabs per window
- enumerate panes/terminals per tab
- read titles and working-directory-like metadata where available
- create windows
- create tabs
- create splits
- focus a specific created surface
- send input for rerun actions when the user explicitly chooses it

Adapter constraints:

- all Ghostty object IDs are treated as ephemeral and never persisted as product identity
- adapter errors must degrade to partial restore results, not crash the whole restore flow
- all AppleScript interactions should be wrapped in typed Rust functions with parsed result structs

### 6. Storage and config

App directory:

- `~/Library/Application Support/ghost-in-a-shell/`

Files:

- `config.json`
- `workspaces/<slug>.json`
- `logs/app.log`

`config.json` v0 fields:

- `default_open = "tui"`
- `confirm_overwrite = true`
- `auto_prompt_labels = true`

No background daemon in v0. The app runs on demand.

## Test Plan

Core scenarios:

- save a Ghostty setup with multiple windows, tabs, and panes under a new name
- save a second differently named setup and verify both are listed
- restore snapshot A and verify the expected window/tab/pane structure is rebuilt
- restore snapshot B immediately after and verify switching works as a restore of a different named setup
- rename a snapshot and verify the old name is gone and the new name is present
- delete a snapshot and verify it no longer appears in list or TUI

Recovery and edge cases:

- restore a snapshot where some terminals have no launch intent and verify they are marked `needs rerun`
- restore a snapshot where working directory metadata is missing and verify the terminal still opens with degraded summary output
- overwrite an existing snapshot with `--force` and verify atomic replacement
- attempt overwrite without `--force` and verify refusal
- adapter failure to create one pane should surface partial failure while still restoring the remaining panes

Implementation-level tests:

- unit tests for slug normalization, overwrite rules, snapshot sorting, and JSON round-trip
- unit tests for restore-result computation
- parser tests for AppleScript output into Rust structs
- integration test fixtures for store read/write using temp directories

Manual acceptance:

- user can arrange Ghostty, save it with a name, close or move on, then later restore or switch back to it through one obvious command or the TUI
- user sees enough plain-language context to recognize which snapshot to bring back
- product never claims exact process resurrection

## Assumptions

- v0 targets **macOS only**
- Ghostty AppleScript support is available and sufficient for enumerating and rebuilding windows, tabs, and panes
- snapshot capture is **manual only** in v0
- switch behavior is implemented as restore of a selected named snapshot
- no separate Raycast app, menu bar app, or embedded Ghostty plugin is part of v0
