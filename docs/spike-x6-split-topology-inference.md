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

But a real user test with a `1 tab / 5 split` arrangement originally showed a remaining failure:

- the saved snapshot captures the correct number of panes
- restore recreates the right number of panes
- restore does **not** recreate the exact split arrangement

The current saved model only persists a flat pane list plus topology hints, not a true split tree. This spike investigates whether Ghostty exposes enough information to derive the split topology anyway and whether that is sufficient for useful reconstruction.

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

In practice, that turned out to be enough for an important middle ground:

- capture directional adjacency as topology hints
- recognize common layout archetypes from that graph
- replay a restore sequence that preserves layout hierarchy convincingly

Live implementation and user verification now confirm this works for common nested arrangements such as:

- dominant left pane
- stacked right-side panes
- bottom-right sub-split

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

Since the original spike, the product has landed in a practical middle position:

1. arbitrary live layouts do not restore with mathematically exact geometry
2. common captured layouts can restore with a close visual approximation that preserves major regions and nesting

---

## Answers

| Question | Answer |
|----------|--------|
| **X6-Q1** | No direct topology API was found. Ghostty AppleScript exposes a flat list of terminals in a tab, but not parent-child split relationships, bounds, or size ratios. |
| **X6-Q2** | Yes, directional adjacency can be inferred indirectly by focusing a terminal, invoking `goto_split:<direction>`, and reading back the resulting `focused terminal`. |
| **X6-Q3** | Not fully. Adjacency is enough to derive a neighbor graph and reconstruct common layout hierarchy convincingly, but not enough to guarantee a unique exact split tree or exact geometry for arbitrary complex layouts. |
| **X6-Q4** | Not reliably in this environment. `System Events` probing against Ghostty timed out, so accessibility is not currently a dependable implementation path. |
| **X6-Q5** | Exact restore of arbitrary preexisting Ghostty split layouts remains unproven. A stronger exact-layout promise is safer if the app owns the topology from creation time rather than reverse-engineering arbitrary live layouts. |

---

## Practical Consequences

### What we can say now

- Pane count and terminal metadata are capturable.
- Directional neighbor relationships are capturable.
- Common nested layout hierarchy is reconstructable from that graph.
- Exact split geometry for arbitrary live layouts is still not proven.

### What this means for v0

The current v0 promise should be:

- recognition-first restore
- correct windows/tabs/pane counts
- cwd and title restoration
- approximate layout hierarchy preservation for common nested arrangements
- honest partial restore

It should **not** promise:

- exact reproduction of arbitrary user-authored nested split geometry

### What would improve the situation

The promising direction from this spike has now been implemented:

- directional adjacency is captured as a topology hint during save
- common layouts are reconstructed from that graph during restore
- the stronger exactness promise is still reserved for future model evolution, not claimed for arbitrary captured geometry

## Outcome

This spike is partially resolved and operationalized.

It did not unlock exact topology capture. It did unlock something more useful for v0:

- enough structural information to preserve the shape of common pane arrangements
- a principled boundary for what the product can and cannot claim

The resulting v0 statement is:

- save captures directional topology hints
- restore preserves common layout hierarchy convincingly
- exact arbitrary divider geometry remains out of scope

## Sources

- Local scripting dictionary: [Ghostty.sdef](/Applications/Ghostty.app/Contents/Resources/Ghostty.sdef)
- Local action reference: [ghostty.5.md](/Applications/Ghostty.app/Contents/Resources/ghostty/doc/ghostty.5.md)
- Live probing on this machine via `osascript` against Ghostty on **March 18, 2026**
