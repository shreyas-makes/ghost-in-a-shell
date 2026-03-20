# ghost-in-a-shell

Save and restore Ghostty workspaces on macOS.

It currently restores:

- window count
- tab count
- terminal count
- working directories
- titles where Ghostty allows them
- approximate split layout for common nested arrangements

It does not yet restore prior process state. Terminals without a saved launch command come back as shells and are marked `needs rerun`.

## Requirements

- macOS
- [Ghostty](https://ghostty.org/)
- Rust toolchain

If `cargo` resolves to the wrong Rust install, run:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Run locally

From the repo root:

```bash
cargo test
cargo run -- save demo
cargo run -- list
cargo run -- restore demo
cargo run -- tui
```

## Typical flow

1. Open Ghostty and arrange windows, tabs, and splits.
2. From inside Ghostty, save the workspace:

```bash
cargo run -- save demo
```

3. Restore it later:

```bash
cargo run -- restore demo
```

4. Browse snapshots in the TUI from inside Ghostty:

```bash
cargo run -- tui
```

## Notes

- The default workflow is to run save and restore from inside Ghostty.
- The snapshot captures the live Ghostty arrangement at the moment you invoke it, including the pane running the command if that pane is part of your setup.
- For best layout restore results, save a fresh snapshot with the current build.
- Restore aims to preserve layout hierarchy, not exact divider ratios.
