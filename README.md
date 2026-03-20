# ghost-in-a-shell

`ghost-in-a-shell` saves and restores Ghostty workspaces on macOS. It restores windows, tabs, terminals, working directories, titles where Ghostty allows them, and approximate split layout for common nested arrangements.

It does not restore prior process state. If a terminal has no saved launch command, it comes back as a shell and is marked `needs rerun`.

## Requirements

You need macOS, [Ghostty](https://ghostty.org/), and a working Rust toolchain. If `cargo` resolves to the wrong Rust install, run `export PATH="$HOME/.cargo/bin:$PATH"` first.

## Usage

From the repo root, run `cargo test` to verify the build. Open Ghostty, arrange your windows, tabs, and splits, then run the commands from inside Ghostty: `cargo run -- save demo`, `cargo run -- list`, `cargo run -- restore demo`, or `cargo run -- tui`.

## Notes

Run save and restore from inside Ghostty as the default workflow. The snapshot captures the live Ghostty arrangement at the moment you invoke it, including the pane running the command if that pane is part of your setup. For the best restore result, save a fresh snapshot with the current build. Restore aims to preserve layout hierarchy rather than exact divider ratios.
