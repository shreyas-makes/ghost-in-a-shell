---
shaping: true
---

# X7 Spike: In-Ghostty Save Invocation

## Context

The shaping direction already says v0 should avoid a second shell and make save, switch, and recovery terminal-hosted where possible.

This spike narrows the question to the user journey we actually want:

- run the CLI or TUI from a normal Ghostty shell pane
- accept that the invoking pane is part of the saved workspace

That is the intended v0 behavior we want to validate.

## Goal

Determine whether the current save flow can be invoked from Ghostty and whether that already supports the intended workflow of saving the live Ghostty setup, including the invoking pane when it is genuinely part of the work.

## Questions

| # | Question |
|---|----------|
| **X7-Q1** | How does the current save path capture Ghostty state, and does the caller need to live outside Ghostty for that to work? |
| **X7-Q2** | What happens if `save` or `tui` is run from a Ghostty pane that belongs to the live layout being captured? |
| **X7-Q3** | Does the current behavior already satisfy the intended v0 user journey of saving directly from a Ghostty pane? |

## Acceptance

Spike is complete when we can describe whether the current product already supports save from Ghostty and whether the resulting capture behavior matches the intended v0 user journey.

---

## Findings

### F1: The save command is already callable from Ghostty

The current CLI `save` path does not inspect its own host terminal. It just calls `adapter.capture_workspace(...)`, and the adapter shells out to AppleScript to ask Ghostty for its current object graph.

Evidence:

- [src/cli.rs](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/src/cli.rs#L53) routes `save` directly to `adapter.capture_workspace(&args.name)`
- [src/ghostty_adapter.rs](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/src/ghostty_adapter.rs#L81) captures the workspace by querying Ghostty state
- [README.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/README.md#L17) already says: "Save from inside Ghostty if you want the current arrangement captured."

So the answer to the narrow technical question is yes: the command can already be run from a Ghostty shell pane.

### F2: Capture scope is the live Ghostty state

The adapter's AppleScript capture walks:

- all Ghostty windows
- all tabs in each window
- all terminals in each tab

There is no exclusion rule for the pane that launched the command. That means the save flow captures the live Ghostty state as it exists at the moment of capture, including the invoking pane.

### F3: Running the command inside Ghostty matches the intended save journey

If the user runs:

```bash
cargo run -- save demo
```

from a Ghostty pane, that pane is part of the live Ghostty layout being enumerated. The same is true if the user runs:

```bash
cargo run -- tui
```

and then presses `s`; the TUI terminal itself is still a Ghostty terminal.

That behavior matches the intended v0 journey:

- work inside Ghostty
- when the setup feels right, run one obvious command from a pane you are already using
- save the setup exactly as it exists in that moment

### F4: The current behavior already satisfies the intended v0 contract

Given the accepted workflow, there is no blocker here. The current implementation already supports:

- terminal-hosted save from inside Ghostty
- capture of the real live arrangement the user is currently in
- CLI or TUI entry points without needing a second terminal app

The outside-terminal path can still exist as an optional workflow, but it is not required for the intended v0 journey.

---

## Answers

| Question | Answer |
|----------|--------|
| **X7-Q1** | Yes, the current save flow can be run from Ghostty. The caller does not need to live outside Ghostty because the adapter captures Ghostty by AppleScript, not by inspecting the caller terminal. |
| **X7-Q2** | The invoking Ghostty pane or TUI surface will be included in the snapshot because capture currently walks the live Ghostty windows, tabs, and terminals. |
| **X7-Q3** | Yes. The current behavior already matches the intended v0 user journey, because saving from the pane you are actively using is acceptable and desirable. |

---

## Recommendation

Adopt the current behavior as the explicit v0 contract:

- save is run from inside Ghostty
- the command can be invoked from a normal working pane or from the in-terminal TUI
- the invoking pane is part of the saved workspace because it is part of the user's live setup

The next product move is not capture-boundary work. It is ergonomics:

- install the binary so it is runnable anywhere
- provide one obvious command inside Ghostty
- document the in-Ghostty save journey as the default workflow
