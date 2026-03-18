---
shaping: true
---

# X6 Spike: Split Topology Inference

## Context

[spike-x5-ghostty-restore-fidelity.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x5-ghostty-restore-fidelity.md) proved that Ghostty can restore:

- window count
- tab count and order
- pane count and creation order
- cwd targets
- visible tab and surface titles

But a real user test with a `1 tab / 5 split` arrangement showed a remaining failure:

- the saved snapshot captures the correct number of panes
- restore recreates the right number of panes
- restore does **not** recreate the exact split arrangement

The current saved model only persists a flat pane list, not a split tree. This spike investigates whether Ghostty exposes enough information to derive the split topology anyway.

## Goal

Determine whether Ghostty exposes split topology directly or indirectly strongly enough that we can infer a tab’s split graph from a live arrangement and persist it for later restore.

## Questions

| # | Question |
|---|----------|
| **X6-Q1** | Does Ghostty AppleScript expose parent-child split relationships, split direction per pane, or geometry data such as bounds, sizes, or positions directly? |
| **X6-Q2** | If direct topology is not exposed, can we infer directional adjacency between panes by combining `focused terminal` with Ghostty actions such as `goto_split:left/right/up/down`? |
| **X6-Q3** | Is directional adjacency enough to reconstruct an exact split tree for later restore, or only enough for a partial approximation? |
| **X6-Q4** | Can macOS accessibility or another local surface expose splitters, bounds, or UI geometry that Ghostty AppleScript does not expose? |
| **X6-Q5** | What product or implementation consequences follow if exact split topology cannot be captured reliably from arbitrary existing Ghostty layouts? |

## Acceptance

Spike is complete when we can describe:

- whether Ghostty exposes split topology directly
- whether adjacency can be inferred indirectly
- whether that inference is strong enough for exact restore or only approximate restore
- whether any non-AppleScript local surface is viable for extracting geometry
- the resulting product boundary for “save arbitrary existing layout” versus “restore exact layout”

---

## Findings

### F1: Ghostty AppleScript exposes a flat terminal list, not a split tree

The local scripting dictionary at [Ghostty.sdef](/Applications/Ghostty.app/Contents/Resources/Ghostty.sdef) exposes:

- `window.tabs`
- `tab.terminals`
- `tab.focused terminal`
- `split`
- `focus`

It does **not** expose:

- a parent terminal or parent split relationship
- split orientation per existing pane
- pane bounds
- pane size ratios
- pane coordinates

So there is no direct AppleScript query for “what split tree does this tab have?”

### F2: Directional adjacency can be inferred indirectly through focus navigation

Ghostty’s action reference in [ghostty.5.md](/Applications/Ghostty.app/Contents/Resources/ghostty/doc/ghostty.5.md) documents `goto_split:<direction>`, and live probing confirmed that this action can be driven through AppleScript `perform action`.

Using a synthetic known layout:

1. create root terminal `A`
2. split `A` to the right, creating `B`
3. split `B` downward, creating `C`

then probing each terminal with:

- `focus <terminal>`
- `perform action "goto_split:left"`
- `perform action "goto_split:right"`
- `perform action "goto_split:up"`
- `perform action "goto_split:down"`
- reading back `tab.focused terminal`

produced a directional neighbor map consistent with the known layout.

That means Ghostty does expose enough behavior to derive a **directional adjacency graph** between panes in a tab.

### F3: Adjacency is useful, but it is not the same thing as an exact split tree

Directional adjacency tells us things like:

- pane `A` has `B` to its right
- pane `B` has `C` below it

That is a meaningful topology signal.

But adjacency alone does **not** necessarily give:

- the full recursive split history
- unique reconstruction for every complex arrangement
- exact size ratios
- exact coordinates

So adjacency is stronger than the current flat pane list, but weaker than a true persisted split tree with geometry.

### F4: macOS accessibility did not produce a viable geometry path in this environment

Probing `System Events` against the Ghostty process timed out repeatedly when trying to inspect:

- window contents
- splitters
- groups
- front window UI contents

This means accessibility is not currently a reliable extraction path in this setup.

That may be due to:

- Automation / Accessibility permission friction
- Ghostty’s UI hierarchy not being cheaply traversable through `System Events`
- the approach being too fragile for a product promise even if it can be made to work manually

For shaping purposes, this is not a dependable v0 boundary.

### F5: The product boundary splits into two different promises

There are now two materially different promises we could make:

1. **Save arbitrary existing Ghostty layouts exactly**
   - requires extracting split topology from live Ghostty
   - currently unproven for exact reconstruction

2. **Save and restore layouts created or normalized by our own model**
   - we can persist our own split tree as we create it
   - exact restore becomes feasible because the app owns the topology

This is the key product consequence of the spike.

---

## Answers

| Question | Answer |
|----------|--------|
| **X6-Q1** | No direct topology API was found. Ghostty AppleScript exposes a flat list of terminals in a tab, but not parent-child split relationships, bounds, or size ratios. |
| **X6-Q2** | Yes, directional adjacency can be inferred indirectly by focusing a terminal, invoking `goto_split:<direction>`, and reading back the resulting `focused terminal`. |
| **X6-Q3** | Not fully. Adjacency is enough to derive a neighbor graph, but not enough to guarantee a unique exact split tree or exact geometry for arbitrary complex layouts. |
| **X6-Q4** | Not reliably in this environment. `System Events` probing against Ghostty timed out, so accessibility is not currently a dependable implementation path. |
| **X6-Q5** | Exact restore of arbitrary preexisting Ghostty split layouts remains unproven. A stronger exact-layout promise is safer if the app owns the topology from creation time rather than reverse-engineering arbitrary live layouts. |

---

## Practical Consequences

### What we can say now

- Pane count and terminal metadata are capturable.
- Directional neighbor relationships are probably capturable.
- Exact split geometry for arbitrary live layouts is still not proven.

### What this means for v0

The current v0 promise should stay:

- recognition-first restore
- correct windows/tabs/pane counts
- cwd and title restoration
- honest partial restore

It should **not** promise:

- exact reproduction of arbitrary user-authored nested split geometry

### What would improve the situation

The most promising next implementation shape is:

- capture directional adjacency as an intermediate topology hint
- evaluate whether common layouts can be reconstructed from that graph
- if exact fidelity is still ambiguous, move the stronger promise to app-owned layouts rather than arbitrary captured layouts

## Sources

- Local scripting dictionary: [Ghostty.sdef](/Applications/Ghostty.app/Contents/Resources/Ghostty.sdef)
- Local action reference: [ghostty.5.md](/Applications/Ghostty.app/Contents/Resources/ghostty/doc/ghostty.5.md)
- Live probing on this machine via `osascript` against Ghostty on **March 18, 2026**
