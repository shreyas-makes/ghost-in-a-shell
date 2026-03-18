---
shaping: true
---

# X3 Spike: Minimal Companion UI Surface

## Context

Shape C already says the product should feel native to Ghostty while still providing better recovery, legibility, and onboarding than existing multiplexers. [spike-r4-ghostty-native.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-r4-ghostty-native.md) established that Ghostty should remain the visible terminal surface, and [spike-c1-2-workspace-model.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-c1-2-workspace-model.md) established the continuity contract and workspace metadata model above Ghostty primitives.

What remains is the UI question:

If Ghostty stays in charge of terminal rendering, what is the smallest product surface we still need outside Ghostty so users can create, recover, understand, and relaunch workspaces without learning mux vocabulary?

## Goal

Define the minimum companion UI surface for v0: which actions require dedicated product UI, which information must remain legible, and how onboarding should appear without creating a second competing shell.

## Questions

| # | Question |
|---|----------|
| **X3-Q1** | What actions require a dedicated launcher, palette, overlay, or companion UI rather than terminal-only controls? |
| **X3-Q2** | What state must remain always visible to satisfy the “know what’s going on” requirement without visual clutter? |
| **X3-Q3** | Which onboarding cues belong in first-run flow versus progressive disclosure during normal use? |

## Acceptance

Spike is complete when we can describe the smallest UX surface that delivers recovery, legibility, and onboarding while still feeling Ghostty-native.

---

## Findings

### F1: Ghostty already solves the “live terminal” problem

Ghostty already provides the things users need once they are inside active shell surfaces:

- terminal rendering
- tabs, splits, and windows
- focus/navigation between surfaces
- command palette and terminal-centric actions
- visible titles and cwd-derived search affordances

So our UI should not duplicate normal in-terminal activity. The product surface should begin where Ghostty becomes weak:

- cross-workspace recognition
- interruption recovery
- relaunch decisions
- first-run explanation of the continuity model

### F2: Workspace operations need a product-level launcher

A user cannot reasonably create, switch, recover, rename, inspect, or relaunch named workspaces through Ghostty alone, because those are product objects above Ghostty primitives. This makes one small workspace launcher non-negotiable.

That launcher should handle:

- create workspace
- open existing workspace
- recover interrupted workspace
- rename workspace
- inspect workspace
- relaunch terminals from saved intent

This is the primary product entry point outside Ghostty.

### F3: Recovery needs a distinct surface, not just a menu item

Recovery is not just “open workspace.” The user has to understand:

- which workspace is likely the right one
- what each terminal was for
- what is still live
- what needs relaunch

That requires richer legibility than a single palette row can provide. A lightweight recovery view is necessary because recovery is the product’s core value proposition, not a secondary settings flow.

### F4: Persistent status should be lightweight and optional

The product should expose status, but not with a heavy persistent dashboard. Since Ghostty remains the visible app, always-on UI should be minimal. The right shape is:

- launcher for deliberate actions
- recovery view for interruption states
- subtle status indicator for quick orientation

The status indicator can be a small menu bar item, dock badge, or lightweight overlay entry point. Its job is not to manage terminals. Its job is to answer:

- which workspace is active
- whether something needs attention
- whether recovery is available

### F5: First-run onboarding should be a guided success path, not documentation

Onboarding should only teach the smallest model needed for the first success:

- a workspace is a named bundle of recoverable terminal work
- the app can reopen it with enough context to continue
- roles and launch intent make recovery more legible

Anything beyond that should be progressive disclosure after the user has successfully created and reopened a workspace once.

### F6: The UI vocabulary should stay recognition-first

To stay aligned with the selected shape, the surface should use labels like:

- `Create workspace`
- `Open workspace`
- `Recover workspace`
- `Relaunch terminal`
- `Needs attention`

And avoid terms like:

- session
- attach
- detach
- pane tree
- resurrect process

This is a language constraint as much as a layout constraint.

---

## Recommended v0 Surface

v0 should have exactly three UI surfaces outside Ghostty:

### 1. Workspace launcher

Primary purpose:

- create
- open
- switch
- recover
- rename
- inspect

Recommended shape:

- command-palette style window or launcher
- keyboard-first
- searchable by workspace name, cwd root, tags, and terminal labels

Core rows should show:

- workspace name
- primary cwd root
- top-level status
- last active time

### 2. Recovery view

Primary purpose:

- explain interrupted work clearly
- distinguish restored context from relaunchable context
- let the user reopen the right workspace in seconds

Recommended shape:

- compact window or sheet opened on startup after interruption, or from launcher
- workspace cards with 2-4 terminal summaries
- explicit status badges and relaunch actions

Each workspace card should show:

- `display_name`
- primary cwd root
- interruption reason if known
- top-level status
- last active time

Each terminal summary should show:

- role or label
- cwd target
- launch intent summary
- `live` / `stopped` / `unknown` / `relaunchable`
- relaunch button when appropriate

### 3. Lightweight status entry point

Primary purpose:

- provide quick orientation without taking over the desktop

Recommended shape:

- menu bar item on macOS for v0

It should expose:

- current or most recent workspace
- count of recoverable or attention-needed workspaces
- quick actions: `Open launcher`, `Recover`, `New workspace`

If this proves unnecessary in implementation, it can degrade to a global hotkey that opens the launcher. But the product still needs a fast re-entry point outside Ghostty.

---

## What Should Not Exist In v0

To keep the product Ghostty-native, v0 should explicitly avoid:

- a persistent multi-pane dashboard competing with Ghostty
- a separate terminal renderer
- a full project-management sidebar that stays open beside terminals
- deep configuration screens before first value
- a dense status bar that tries to mirror every Ghostty surface live

The product should feel like a continuity layer with a few sharp surfaces, not a second desktop shell.

---

## Action Allocation

| Action | Ghostty | Our UI |
|--------|---------|--------|
| Focus existing terminal | ✅ | |
| Use terminal normally | ✅ | |
| Split/tab/window operations during live work | ✅ | |
| Create named workspace | | ✅ |
| Switch between named workspaces | Partial | ✅ |
| Recover interrupted workspace | | ✅ |
| Explain terminal purpose and relaunchability | | ✅ |
| Relaunch terminal from saved intent | Partial | ✅ |
| Show top-level workspace attention state | | ✅ |
| First-run explanation of continuity model | | ✅ |

---

## Onboarding Model

### First run should include

- one short explanation of what a workspace is
- one starter workspace template
- one visible success path: create a workspace, close/reopen, recover it
- one prompt to name key terminal roles if the template did not already supply them

### Normal use should include

- contextual shortcut hints after successful actions
- optional prompts like `Save this command as launch intent`
- optional relabel suggestions when a terminal looks important but unlabeled

### Not first-run material

- exhaustive keybindings
- advanced layout customization
- process semantics and caveats in detail
- cross-platform capability comparisons

---

## Answers

| Question | Answer |
|----------|--------|
| **X3-Q1** | A dedicated product launcher is required for create/open/switch/recover/rename/inspect actions, and a dedicated recovery view is required for interruption handling. |
| **X3-Q2** | The always-visible state should be minimal: active or recent workspace, attention/recovery presence, and a fast entry point back into the launcher or recovery flow. |
| **X3-Q3** | First run should teach only the workspace continuity model and walk the user through one successful creation-and-recovery loop. Everything else should be progressive disclosure. |

---

## Risks

| ID | Risk | Consequence |
|----|------|-------------|
| **X3-K1** | Recovery view could grow into a heavy dashboard | Product starts competing with Ghostty visually |
| **X3-K2** | Launcher could become a dumping ground for settings | First-run simplicity erodes |
| **X3-K3** | Menu bar/status entry point may feel redundant | v0 should keep it minimal and removable |
| **X3-K4** | Too much ambient status could imply stronger runtime certainty than we actually have | UI must stay aligned with `live` / `unknown` / `relaunchable` confidence semantics |

---

## Recommendation

The smallest honest companion surface for v0 is:

- one workspace launcher
- one recovery view
- one lightweight status entry point

This is enough to satisfy recovery, legibility, and onboarding without creating a second shell around Ghostty.

## Proposed Update To Shape C

The most accurate refinement to [shaping.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/shaping.md) is:

- `C4` should be interpreted as the workspace launcher plus recovery entry points, not as a generic command palette placeholder
- `C6` onboarding should explicitly center one successful creation-and-recovery loop
- `X3` should be considered shaped once the minimal three-surface model is adopted

## Decision

Proceed with a minimal companion UI made of:

- workspace launcher
- recovery view
- lightweight status entry point

## Sources

- [R4 spike](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-r4-ghostty-native.md)
- [C1.2 workspace model spike](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-c1-2-workspace-model.md)
