# ghost-in-a-shell

`ghost-in-a-shell` saves and restores Ghostty workspaces on macOS. It restores windows, tabs, terminals, working directories, titles where Ghostty allows them, and approximate split layout for common nested arrangements.

It does not restore prior process state. If a terminal has no saved launch command, it comes back as a shell and is marked `needs rerun`.

## Requirements

You need macOS, [Ghostty](https://ghostty.org/), and a working Rust toolchain. If `cargo` resolves to the wrong Rust install, run `export PATH="$HOME/.cargo/bin:$PATH"` first.

## Usage

From the repo root, run `cargo test` to verify the build. Open Ghostty, arrange your windows, tabs, and splits, then save the workspace with `cargo run -- save demo`. You can inspect saved snapshots with `cargo run -- list`, restore one with `cargo run -- restore demo`, and browse them in the terminal UI with `cargo run -- tui`.

## Notes

Save from inside Ghostty if you want the current arrangement captured. For the best restore result, save a fresh snapshot with the current build. Restore aims to preserve layout hierarchy rather than exact divider ratios.
