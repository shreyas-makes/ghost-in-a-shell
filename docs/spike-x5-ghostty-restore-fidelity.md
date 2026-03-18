---
shaping: true
---

# X5 Spike: Ghostty Restore Fidelity

## Context

Shape `C` is selected in [shaping.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/shaping.md), and the current implementation already proves that the product can:

- save a named Ghostty snapshot
- persist window, tab, pane, cwd, and title-like metadata
- list and inspect saved snapshots
- run a restore flow and render a recovery summary

What is not yet proven is the most important macOS-native part of `C3`:

- exact tab recreation
- exact split recreation
- reliable pane targeting and creation order
- applying titles or labels back onto restored Ghostty surfaces

The current Rust adapter is close enough to validate the product boundary, but not yet concrete enough to claim that Ghostty restore fidelity is solved.

## Goal

Determine how Ghostty's real AppleScript behavior on macOS maps to the saved workspace model, and identify the concrete adapter steps needed to rebuild saved windows, tabs, panes, cwd targets, and visible labels reliably enough for v0.

## Questions

| # | Question |
|---|----------|
| **X5-Q1** | What AppleScript commands and object relationships does Ghostty actually expose at runtime for creating windows, tabs, and splits, and what values do those commands return that can be used as stable in-memory anchors during one restore operation? |
| **X5-Q2** | How should the adapter map a saved logical layout like `1 window -> 2 tabs -> 4 panes` into a deterministic sequence of Ghostty automation calls so tab order and split order match the saved snapshot? |
| **X5-Q3** | What metadata can be applied back onto restored Ghostty surfaces in practice: working directory, title, tab name, terminal label, initial command, or other visible state? |
| **X5-Q4** | Which restore failures should degrade to `needs rerun` for one terminal or one tab, and which ones mean the whole restore is structurally wrong enough that the adapter should stop? |
| **X5-Q5** | What exact manual validation scenarios do we need on a live macOS Ghostty instance to prove v0 restore fidelity for windows, tabs, panes, cwd placement, and visible recognition cues? |

## Acceptance

Spike is complete when we can describe:

- the real Ghostty AppleScript object model and creation commands we can depend on during one restore run
- the ordered adapter algorithm needed to rebuild saved windows, tabs, and panes deterministically
- which saved metadata can be restored visibly versus only kept as recovery hints
- the degradation rules for partial restore versus structural failure
- the live manual test matrix required to validate Ghostty restore fidelity on macOS

## Why This Spike Exists

This is not about exact process resurrection. That is already out of scope for `R2`.

This spike is about the narrower but still critical promise inside `R4` and `C3`:

- Ghostty should remain the real terminal surface
- the product should rebuild recognizable saved Ghostty structure honestly
- restore should feel materially closer to “bring my setup back” than “open a few best-effort shells”

## Investigation Target

The primary boundary under investigation is:

- [src/ghostty_adapter.rs](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/src/ghostty_adapter.rs)

The product-level behavior this must satisfy is defined in:

- [PLAN.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/PLAN.md)
- [shaping.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/shaping.md)
- [spike-r4-ghostty-native.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-r4-ghostty-native.md)
- [spike-c1-2-workspace-model.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-c1-2-workspace-model.md)

## Expected Outputs

The spike should produce:

1. A concrete adapter call sequence for restoring one saved workspace.
2. A table of saved fields vs actual restorable Ghostty fields.
3. A partial-failure policy for terminal, tab, and window creation.
4. A live manual validation checklist for Ghostty on macOS.

---

## Findings

### F1: Ghostty 1.3.1 exposes enough AppleScript structure to rebuild layout deterministically

Live probing on this machine shows Ghostty **1.3.1** and confirms that the scripting dictionary returns runtime IDs for:

- `window.id`
- `tab.id`
- `terminal.id`

The local Ghostty dictionary at [Ghostty.sdef](/Applications/Ghostty.app/Contents/Resources/Ghostty.sdef) defines:

- `new window` returning a `window`
- `new tab` returning a `tab`
- `split` returning a `terminal`
- `window.selected tab`
- `tab.index`
- `tab.selected`
- `tab.focused terminal`
- `window.tabs`
- `tab.terminals`

This is enough to build a restore sequence that keeps stable in-memory anchors for one restore run without persisting Ghostty object IDs to disk.

### F2: Window, tab, and split creation order is recoverable if we replay the saved tree in pre-order

Live probing confirmed:

- `new tab in winRef` appends a tab to the target window
- `tab.index` is 1-based and reflects visible order
- `split terminal 1 of tabRef direction right` creates a second terminal in that tab
- `tab.terminals` enumerates terminals in creation order

For v0, the adapter can restore saved structure reliably enough by replaying:

1. each saved window in order
2. each tab within that window in order
3. each pane within that tab in saved pane order

This does not prove arbitrary geometric equivalence for complex split trees, but it does give a deterministic restore algorithm for the current linear pane list model in `workspace_snapshot`.

### F3: Working directory is restorable, but it may not be observable immediately

The `surface configuration` record in [Ghostty.sdef](/Applications/Ghostty.app/Contents/Resources/Ghostty.sdef) supports:

- `initial working directory`
- `command`
- `initial input`
- `wait after command`
- `environment variables`

Live probing confirmed that `initial working directory` does apply to newly created terminals, but `terminal.working directory` may be empty immediately after creation and only becomes available after a short delay once shell integration reports it.

Implication:

- setting cwd during restore is viable
- verifying cwd synchronously right after creation is not reliable
- adapter verification should tolerate a short observation delay or treat cwd as best-effort postcondition

### F4: Title and tab naming are not writable properties, but they are still controllable indirectly

The scripting dictionary exposes:

- `window.name` as read-only
- `tab.name` as read-only
- `terminal.name` as read-only

There is no direct AppleScript setter for these names in [Ghostty.sdef](/Applications/Ghostty.app/Contents/Resources/Ghostty.sdef).

However, live probing confirmed that `perform action` works on a terminal, including:

- `perform action "set_surface_title:demo-surface" on termRef`
- `perform action "set_tab_title:demo-tab" on termRef`

These actions successfully changed `terminal.name` and `tab.name` at runtime.

Implication:

- visible terminal and tab labels are restorable in v0
- they should be treated as action-driven restore steps, not property assignments
- there is still no direct AppleScript mechanism here for a separate product-level label beyond title overrides

### F5: Partial restore should degrade per surface, not abort the whole workspace

The current product promise for `R2` is recognition-first continuity, not exact resurrection.

That means adapter failure policy should be:

- if one window cannot be created, all terminals under that window become `needs rerun`
- if one tab cannot be created, all terminals under that tab become `needs rerun`, but sibling tabs/windows continue
- if one split cannot be created, that terminal becomes `needs rerun`, but previously restored siblings remain valid
- if title-setting or tab-title-setting fails, the terminal is still considered structurally restored if the shell surface exists in the right window/tab/pane slot

The adapter should only hard-stop the whole restore when the failure removes the ability to continue deterministic reconstruction, for example:

- Ghostty is unavailable
- AppleScript is disabled
- the initial root window creation fails and no restore target can be materialized at all

### F6: The current saved model is good enough for v0 tabs and pane counts, but not for arbitrary split geometry

The saved snapshot currently stores:

- ordered windows
- ordered tabs
- ordered panes
- `layout_slot` strings such as `pane-1`, `pane-2`

This is enough to restore:

- window count
- tab count
- terminal count per tab
- pane creation order

It is not enough to restore an arbitrary split tree with exact left/right/up/down nesting, because the current model does not persist parent-child split structure.

Implication:

- exact layout fidelity is only answerable up to the current model boundary
- if we want deeper split fidelity later, `workspace_snapshot` must evolve beyond a flat pane list

---

## Answers

| Question | Answer |
|----------|--------|
| **X5-Q1** | Ghostty exposes `new window`, `new tab`, `split`, `focus`, `input text`, `send key`, and `perform action`, with runtime `id` values on windows, tabs, and terminals that are stable enough to use as anchors during one restore run. |
| **X5-Q2** | The adapter should restore deterministically by replaying the saved workspace in order: create window root terminal, append tabs in saved order, then create additional panes within each tab in saved pane order using the first terminal in that tab as the split anchor unless a richer split tree is later persisted. |
| **X5-Q3** | Working directory is directly restorable via `initial working directory`. Commands and initial text are directly restorable via `command`, `initial input`, and `wait after command`. Tab and surface titles are indirectly restorable through `perform action "set_tab_title:..."` and `perform action "set_surface_title:..."`. There is no direct writable AppleScript property for product-level labels. |
| **X5-Q4** | Restore failures should degrade at the smallest honest scope: terminal-level for split or rerun failures, tab-level for tab creation failures, window-level for window creation failures, and whole-restore failure only when Ghostty or AppleScript cannot materialize any root surface. |
| **X5-Q5** | The live validation matrix should prove: single-window single-tab restore, multi-tab restore order, multi-split restore count/order, cwd restoration after a short delay, title override application, rerunnable command execution, and honest partial-failure summaries when one creation step is intentionally broken. |

---

## Recommended Adapter Algorithm

For one saved workspace:

1. Create the first terminal of each saved window with `new window`.
2. Save returned `window.id`, `tab.id`, and `terminal.id` as in-memory anchors.
3. For each remaining saved tab in that window, call `new tab in winRef`.
4. For each remaining pane in that tab, call `split anchorTerm direction <dir>`.
5. Apply `initial working directory` in the `surface configuration` for every created terminal.
6. If a saved launch command should run immediately, use `command` or `initial input`, then `wait after command` when needed.
7. After surfaces exist, apply visible naming with:
   - `perform action "set_surface_title:..."`
   - `perform action "set_tab_title:..."`
8. Compute per-terminal restore outcome:
   - `restored` when structure exists and required title/cwd steps did not invalidate recognition
   - `needs rerun` when the shell exists but intended work did not restart, or when the target surface could not be created

## Saved Fields vs Restorable Fields

| Saved field | Restorable in Ghostty? | Mechanism | Notes |
|-------------|------------------------|-----------|-------|
| Window count/order | Yes | `new window` in sequence | Deterministic |
| Tab count/order | Yes | `new tab in <window>` in sequence | `tab.index` confirms order |
| Pane count/order | Yes, within current flat model | `split <terminal> direction ...` in sequence | Exact geometric tree is not preserved yet |
| Working directory | Yes | `initial working directory` | Observation may lag briefly |
| Launch command | Yes | `command` or `initial input` | Choose one policy and keep it consistent |
| Wait-after-exit behavior | Yes | `wait after command` | Useful for one-shot commands |
| Terminal title | Yes | `perform action "set_surface_title:..."` | Indirect, not a writable property |
| Tab title | Yes | `perform action "set_tab_title:..."` | Indirect, not a writable property |
| Product-level label | Partially | map label to title override | No separate native label field in AppleScript |
| Exact process state | No | out of scope | `R2` intentionally rejects this promise |

## Manual Validation Matrix

| ID | Scenario | What must be true |
|----|----------|-------------------|
| **X5-M1** | Save and restore 1 window, 1 tab, 1 terminal | One new window appears and lands in the saved cwd |
| **X5-M2** | Save and restore 1 window, 2 tabs | Tabs reappear in saved order and selected tab behavior is predictable |
| **X5-M3** | Save and restore 1 tab, 3 terminals | The right number of splits reappear and terminals enumerate in saved creation order |
| **X5-M4** | Restore saved cwd targets | `terminal.working directory` eventually reports the expected path after shell startup |
| **X5-M5** | Restore saved titles | `terminal.name` and `tab.name` reflect the intended overrides after `perform action` |
| **X5-M6** | Restore with explicit launch command | Rerunnable command starts in the intended terminal and reports `restored` |
| **X5-M7** | Break one split creation intentionally | Only that terminal degrades to `needs rerun`; sibling tabs/windows continue |
| **X5-M8** | Break one tab creation intentionally | That tab degrades honestly while other tabs/windows continue |

## Decision

`X5` is no longer blocked on basic feasibility.

We now know enough to implement a real v0 adapter that honestly restores:

- window count
- tab count and order
- pane count and creation order
- cwd targets
- visible terminal and tab titles

What remains open is not whether Ghostty can do this, but how far we want to push layout fidelity beyond the current flat pane model.

## Sources

- Official docs: [Ghostty AppleScript docs](https://ghostty.org/docs/features/applescript)
- Local scripting dictionary: [Ghostty.sdef](/Applications/Ghostty.app/Contents/Resources/Ghostty.sdef)
- Local action reference: [ghostty.5.md](/Applications/Ghostty.app/Contents/Resources/ghostty/doc/ghostty.5.md)
- Live probing on this machine via `osascript` against Ghostty **1.3.1**
