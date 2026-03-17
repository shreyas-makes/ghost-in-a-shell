---
shaping: true
---

# R4 Spike: Ghostty-Native Integration

## Context

Shape C depends on the product feeling native to Ghostty rather than acting as a separate terminal with its own renderer and identity. That requires a concrete understanding of Ghostty's current automation and control surface, and of the platform differences that affect what "native" can honestly mean.

R4 from [shaping.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/shaping.md) is:

> The product feels native to Ghostty rather than replacing Ghostty with a separate terminal identity.

## Goal

Determine what Ghostty-native integration is technically available today, what is platform-specific, and what architecture that implies for a Rust continuity layer, with the option to narrow scope to macOS first.

## Questions

| # | Question |
|---|----------|
| **X1-Q1** | How can an external Rust process create or focus Ghostty windows, tabs, or equivalent terminal surfaces? |
| **X1-Q2** | What Ghostty state can be addressed or restored externally, and what state must our layer own itself? |
| **X1-Q3** | Are there extension points or automation hooks that make a Ghostty-native integration feasible without modifying Ghostty itself? |

## Acceptance

Spike is complete when we can describe the available Ghostty integration surface, its current platform asymmetry, and the architectural consequences for a Ghostty-native v0.

---

## Findings

### F1: macOS already has a meaningful automation surface

Ghostty exposes a native AppleScript dictionary on macOS. According to the official docs, Ghostty 1.3.0 introduced AppleScript support on **March 9, 2026**. The AppleScript API can:

- Query the object model: application → windows → tabs → terminals
- Create windows, tabs, and splits
- Focus terminals
- Send input text and key events
- Pass surface configuration including working directory, command, initial input, wait behavior, and environment variables

This means a Rust layer on macOS can realistically orchestrate Ghostty rather than replace it. The Rust app can own workspace metadata and call `osascript` or a native Apple event bridge to materialize or recover a workspace inside Ghostty.

### F2: Linux has a much thinner native control surface today

The official Linux docs describe D-Bus and systemd integration for creating new windows efficiently via `ghostty +new-window`. GTK single-instance mode can also route new launches into an existing process. But the surfaced integration documented here is much narrower than macOS AppleScript.

What we can support from primary docs:

- Launch Ghostty
- Open a new window efficiently on systems using the documented D-Bus path
- Rely on single-instance behavior to route windows into an existing app instance

What is not clearly documented in the sources reviewed:

- External creation of tabs or splits on Linux
- Rich querying of current windows/tabs/terminals on Linux
- A Linux-native equivalent to the macOS AppleScript object model

So “Ghostty-native” is currently much more capable on macOS than Linux.

### F3: Ghostty already owns some UX primitives we should reuse, not rebuild

Ghostty already has native notions of:

- Windows, tabs, and splits
- Command palette
- Tab and surface titles
- Session search in the command palette by title or working directory
- Shell integration features such as working-directory inheritance

This matters because our product should avoid recreating these primitives unless Ghostty lacks the persistence and continuity layer we need. The best fit is to add:

- Workspace registry
- Continuity state store
- Restore/recovery orchestration
- New-user workflow around those concepts

And reuse Ghostty for:

- Rendering
- Layout primitives where accessible
- Built-in terminal actions and labels

### F4: “Native to Ghostty” cannot mean “inside Ghostty” yet

The current sources support an external orchestration model, not an embedded plugin model. There is no reviewed evidence here of a stable plugin system or extension API for inserting custom product UI directly into Ghostty chrome.

So the honest meaning of “native to Ghostty” for v0 is:

- Ghostty remains the visible terminal application
- Our system launches and controls Ghostty where the platform permits
- Any extra UI should be minimal and companion-style, not a replacement shell

### F5: There is active motion toward more scripting, but the surface is still evolving

The Ghostty 1.3.0 release notes explicitly describe AppleScript support as a preview feature and say Ghostty 1.4 is expected to improve scriptability further. That is promising, but it also means the integration surface may change. We should treat it as useful but not yet stable enough to overfit our architecture to undocumented behavior.

### F6: macOS-specific UX constraints affect how “native” tabs behave

Ghostty currently uses macOS native tabs, and the docs note this causes tab/window-manager oddities in tools like Yabai and Aerospace. Longer term, Ghostty intends to move toward a custom tabbing solution. For our product, that means we should avoid relying too heavily on macOS tab semantics as a universal abstraction because Ghostty itself may change this layer later.

---

## Answers

| Question | Answer |
|----------|--------|
| **X1-Q1** | On macOS, yes: AppleScript can create windows, tabs, and splits and can focus/query terminals. On Linux, the reviewed native path is much thinner: opening new windows via CLI/D-Bus and relying on single-instance behavior. |
| **X1-Q2** | Ghostty can own rendering, native surfaces, some titles, and terminal layout primitives. Our layer still needs to own workspace identity, recovery metadata, persistence policy, interruption state, and any cross-platform continuity model. |
| **X1-Q3** | Yes, but asymmetrically. macOS has a viable automation hook today. Linux has a lighter launch/control hook. Neither reviewed source suggests an embedded extension surface for product UI inside Ghostty itself. |

---

## Architectural Consequences

### Recommended v0 meaning of "Ghostty-native"

For v0, "Ghostty-native" should mean:

- Ghostty is the only terminal renderer users interact with
- Our Rust app is an external continuity/workspace orchestrator
- The product manages named workspaces, restore flows, and recovery context
- The product uses the most native Ghostty control mechanism available per platform

### Recommended scope decision

If we want a strong first release instead of weak cross-platform parity, we should explicitly choose:

`macOS first`

Reasoning:

- macOS has the only reviewed rich automation surface today
- It supports the Ghostty-native claim much more honestly
- It lets the product focus on DX, onboarding, and continuity instead of fighting platform gaps
- It keeps the architecture clean: design the workspace model once, then add thinner adapters later

### Recommended architecture

| Part | Mechanism |
|------|-----------|
| **R4-A1** | Rust daemon or CLI owns workspace registry, persistence store, and recovery logic |
| **R4-A2** | macOS adapter uses AppleScript or Apple event automation to create/focus/query Ghostty windows, tabs, and splits |
| **R4-A3** | Non-macOS adapters are deferred until the workspace model and recovery UX are proven on macOS |
| **R4-A4** | Minimal companion UI or command launcher exists only for workspace operations Ghostty does not already expose |
| **R4-A5** | Workspace model stays above Ghostty primitives so the app can survive Ghostty API evolution and platform differences |

### Product consequence

We should not start by building a “full Ghostty-native multiplexer.” We should start by building a:

`Ghostty-native persistent workspace orchestrator`

That is a tighter, more honest promise, and it aligns with the actual job to be done.

---

## Risks

| ID | Risk | Consequence |
|----|------|-------------|
| **K1** | AppleScript in Ghostty 1.3 is explicitly a preview feature | Breaking changes may affect the macOS adapter in Ghostty 1.4+ |
| **K2** | Linux control surface appears much thinner than macOS | Cross-platform parity may be poor in early versions |
| **K3** | Ghostty’s tab model is evolving on macOS | Native tab assumptions may become stale |
| **K4** | No reviewed embedded plugin/UI API | Rich in-window product UI would likely require upstream Ghostty changes or a companion UI |

---

## Recommendation

R4 is viable if we narrow the claim.

We can satisfy R4 for v0 by defining “native to Ghostty” as:

- Ghostty is the terminal surface
- We orchestrate Ghostty externally
- We keep added UI minimal
- We prioritize continuity, restore, and onboarding over deep terminal layout control
- We ship macOS first, where the integration surface is strong enough to support the claim

What R4 should **not** mean yet:

- Cross-platform full control of every Ghostty primitive
- Embedded custom UI inside Ghostty
- Feature parity between macOS and Linux adapters

---

## Proposed Update To Shape C

If we ripple this back into [shaping.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/shaping.md), the most accurate refinement is:

- `C3` should explicitly be an adapter-based Ghostty integration, with macOS-first depth
- `C5` persistence should be defined above Ghostty primitives, not in terms of exact terminal resurrection
- `R4` can move toward ✅ only once Shape C says “Ghostty-native orchestrator” instead of implying embedded Ghostty extension behavior

---

## Decision

Proceed `macOS first`.

---

## Sources

- [Ghostty AppleScript docs](https://ghostty.org/docs/features/applescript)
- [Ghostty 1.3.0 release notes](https://ghostty.org/docs/install/release-notes/1-3-0)
- [Ghostty Linux systemd and D-Bus docs](https://ghostty.org/docs/linux/systemd)
- [Ghostty GTK single-instance docs](https://ghostty.org/docs/help/gtk-single-instance)
- [Ghostty keybind action reference](https://ghostty.org/docs/config/keybind/reference)
- [Ghostty macOS tiling window manager notes](https://ghostty.org/docs/help/macos-tiling-wms)
