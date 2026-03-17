---
shaping: true
---

# Ghostty-Native Session Continuity — Shaping

## Requirements (R)

| ID | Requirement | Status |
|----|-------------|--------|
| R0 | Persistence is the default mental model, not an advanced feature users must discover later | Core goal |
| R1 | Common workflows require little or no multiplexer vocabulary such as detach, attach, prefix, or mode names | Must-have |
| R2 | 🟡 Users can resume interrupted work with enough context to understand what each workspace and terminal was for through a recognition-first continuity contract rather than exact process resurrection | Must-have |
| R3 | New users can succeed without reading docs before getting value | Must-have |
| R4 | 🟡 The product feels native to Ghostty on macOS first by using Ghostty as the terminal surface and adding continuity above Ghostty rather than replacing it with a separate terminal identity | Must-have |
| R5 | Keyboard-first operation remains fast after onboarding | Must-have |
| R6 | Setup and configuration stay minimal for the default path | Must-have |
| R7 | The product emphasizes continuity and recovery over broad tmux-style feature parity | Must-have |
| R8 | The visual and interaction model should feel more refined and on-brand than current terminal multiplexer UX | Must-have |

---

## CURRENT: tmux / zellij baseline

| Part | Mechanism | Flag |
|------|-----------|:----:|
| **CURRENT1** | `tmux` provides mature sessions, windows, panes, and persistence primitives, but exposes them through expert-oriented commands and vocabulary | |
| **CURRENT2** | `tmux` relies on recall-heavy usage: users must know what state exists and how to query or reattach it | |
| **CURRENT3** | `zellij` improves discoverability and presents more visible UI, but still frames the product primarily as a multiplexer | |
| **CURRENT4** | `zellij` visual language and interaction style do not feel aligned with the intended Ghostty + Linear-esque aesthetic | |
| **CURRENT5** | Both tools require users to understand sessions as a terminal concept before they get the continuity benefit they actually want | |

---

## A: Better tmux

| Part | Mechanism | Flag |
|------|-----------|:----:|
| **A1** | Keep the classic multiplexer model: sessions, windows, panes, attach/detach | |
| **A2** | Add a cleaner command surface, visible hints, and improved discoverability around existing multiplexer operations | |
| **A3** | Improve status presentation so users can see more of the current state without querying manually | |
| **A4** | Keep persistence as a feature within the multiplexer model rather than the primary mental model | |

## B: Polished TUI multiplexer

| Part | Mechanism | Flag |
|------|-----------|:----:|
| **B1** | Build a more approachable, aesthetically refined TUI-first multiplexer | |
| **B2** | Offer visible navigation, onboarding hints, and streamlined common actions | |
| **B3** | Present sessions, panes, and layouts with stronger defaults and a more coherent UI | |
| **B4** | Keep the product centered on pane/tab/session management, with persistence as one capability among many | |

## C: macOS-first Ghostty-native workspace orchestrator

| Part | Mechanism | Flag |
|------|-----------|:----:|
| **C1** | Define the primary object as a named workspace that represents recoverable terminal work, not a raw pane tree | |
| **C2** | Make continuity the default lifecycle: create, reopen, and resume workspaces without teaching detach/attach first | |
| **C3** | 🟡 Use Ghostty as the visible terminal surface and orchestrate it externally through a macOS-first adapter rather than building a separate terminal emulator | |
| 🟡 **C4** | 🟡 Provide a simple in-terminal command surface for save, switch, and recover actions | |
| **C5** | Persist enough state to restore project context, layout intent, and interruption context in a legible way | ⚠️ |
| **C6** | Expose clear status cues so users can understand which workspace is active and what each terminal is doing | ⚠️ |
| **C7** | Keep the default path zero- or low-config, with keyboard acceleration available after the initial success path | |
| **C8** | 🟡 Defer non-macOS adapters until the workspace and recovery model are proven on macOS | |

---

## Fit Check

| Req | Requirement | Status | A | B | C |
|-----|-------------|--------|---|---|---|
| R0 | Persistence is the default mental model, not an advanced feature users must discover later | Core goal | ❌ | ❌ | ✅ |
| R1 | Common workflows require little or no multiplexer vocabulary such as detach, attach, prefix, or mode names | Must-have | ❌ | ❌ | ✅ |
| R2 | 🟡 Users can resume interrupted work with enough context to understand what each workspace and terminal was for through a recognition-first continuity contract rather than exact process resurrection | Must-have | ❌ | ❌ | 🟡✅ |
| R3 | New users can succeed without reading docs before getting value | Must-have | ❌ | ❌ | ✅ |
| R4 | 🟡 The product feels native to Ghostty on macOS first by using Ghostty as the terminal surface and adding continuity above Ghostty rather than replacing it with a separate terminal identity | Must-have | ❌ | ❌ | 🟡✅ |
| R5 | Keyboard-first operation remains fast after onboarding | Must-have | ✅ | ✅ | ✅ |
| R6 | Setup and configuration stay minimal for the default path | Must-have | ❌ | ❌ | ✅ |
| R7 | The product emphasizes continuity and recovery over broad tmux-style feature parity | Must-have | ❌ | ❌ | ✅ |
| R8 | The visual and interaction model should feel more refined and on-brand than current terminal multiplexer UX | Must-have | ❌ | ✅ | ✅ |

**Notes:**
- A fails R0, R1, R3, R6, and R7 because it keeps the traditional multiplexer mental model and only improves the surface.
- B fails R0, R1, R3, and R7 because a more polished multiplexer still teaches the wrong primary abstraction.
- A and B fail R4 because both imply a distinct terminal product identity rather than a Ghostty-native one.
- 🟡 C can satisfy R2 on a macOS-first basis because the product now has a concrete workspace metadata contract, recognition-first recovery model, and explicit `live` / `stopped` / `unknown` / `relaunchable` status vocabulary.
- 🟡 C now satisfies R4 on macOS first because the product can use Ghostty as the visible surface and orchestrate windows, tabs, and splits externally through Ghostty's AppleScript automation rather than replacing the terminal.

---

## Selected Shape

Selected shape: **🟡 C: macOS-first Ghostty-native workspace orchestrator**

Product promise: **Ghostty-native persistent workspace orchestration**

Why this shape:
- It targets the real user job: preserving, resuming, and understanding terminal work through interruptions.
- It keeps the product native to Ghostty instead of competing with Ghostty on terminal rendering.
- 🟡 It has a concrete macOS integration path: Ghostty remains the terminal surface while a Rust layer owns workspace continuity and uses AppleScript automation to materialize or recover workspaces.
- It creates room for better onboarding because users start from workspaces and recovery, not multiplexer vocabulary.

What remains unsolved:
- 🟡 `R4` is shaped enough for macOS-first v0, but carries implementation risk because Ghostty's AppleScript surface is new and still evolving.
- 🟡 The exact v0 entry point still needs to be proven: opening Ghostty may not itself trigger recovery automatically, so the fallback path is an in-terminal command or TUI that makes named snapshot restore and switching fast and clear.

---

## C x R

| Shape | R0 | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 |
|-------|----|----|----|----|----|----|----|----|----|
| C | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**Notes:**
- `R2` passes for Shape `C` because the current selected shape now has a concrete recognition-first continuity contract: `workspace_snapshot` plus `terminal_snapshot`, explicit recovery states, and relaunch actions where exact continuity cannot be proven.
- `R4` passes for Shape `C` on a macOS-first basis because Ghostty remains the visible terminal surface while the product adds continuity through an external orchestration layer using Ghostty's AppleScript automation.
- Remaining uncertainty is now implementation risk inside Shape `C`, not a fit-check failure: `C5`, `C6`, `C2.2`, `C2.3`, and `C6.3` still carry `⚠️` and should be resolved or further spiked during detailing and slicing.

---

## Detail C: Concrete Product Shape

| Part | Mechanism | Flag |
|------|-----------|:----:|
| **C1** | **Workspace model** | |
| 🟡 C1.1 | 🟡 A workspace is the saved Ghostty setup a user recognizes as “my work from last time,” for example: several windows, tabs, and panes arranged for one project or task, each with a human-readable label like `frontend`, `api`, or `notes` | |
| 🟡 C1.2 | 🟡 The saved workspace remembers where each terminal was opened, what it was for, how the windows/tabs/panes were arranged, and what command or task each terminal was last meant to run. See [spike-c1-2-workspace-model.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-c1-2-workspace-model.md). | ⚠️ |
| 🟡 C1.3 | 🟡 Workspace is the primary thing users restore, switch, and save; users should not need to think in terms of raw pane trees or attach/detach concepts | |
| **C2** | **Continuity-first lifecycle** | |
| C2.1 | New workspace creation defaults to recoverable behavior | |
| 🟡 C2.2 | 🟡 Resume flow should let users bring back any named saved Ghostty setup through one obvious in-terminal command or TUI, without asking them to remember mux vocabulary | ⚠️ |
| 🟡 C2.3 | 🟡 After Ghostty is closed, the laptop crashes, or the machine restarts, the product should offer recent named snapshots and show what could be restored exactly versus what must be manually rerun | ⚠️ |
| **C3** | **Ghostty-native surface (macOS first)** | ⚠️ |
| C3.1 | 🟡 Ghostty remains the only terminal renderer and visible shell surface users interact with | |
| 🟡 C3.2 | 🟡 A small Rust service or CLI keeps the saved record of each workspace: window layout, tab layout, pane layout, labels, remembered locations, and recovery logic above Ghostty's own built-in window/tab/split concepts | |
| 🟡 C3.3 | 🟡 macOS automation uses Ghostty's AppleScript hooks to open windows, create tabs, split panes, focus the right view, and read back enough visible state to rebuild the saved setup. See [spike-x5-ghostty-restore-fidelity.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x5-ghostty-restore-fidelity.md). | ⚠️ |
| 🟡 C3.4 | 🟡 Any extra product UI in v0 should live inside the terminal as a CLI or TUI for restore and save flows, rather than starting with a separate app surface | |
| C3.5 | 🟡 Ghostty preview-scriptability risk is accepted, so adapter boundaries must stay narrow and swappable | |
| C3.6 | 🟡 Linux and other platforms are deferred until the workspace model and recovery UX are proven on macOS | |
| C3.7 | The product avoids a second competing visual shell where possible and adds continuity rather than replacing terminal interaction | |
| **C4** | **Human command surface** | |
| 🟡 C4.1 | 🟡 The primary command surface in v0 is an in-terminal CLI or TUI for save, restore, switch, and rename actions | |
| C4.2 | Recognition-first action labels replace obscure mux verbs | |
| C4.3 | Shortcut hints appear in context after first-run success rather than front-loading complexity | |
| 🟡 C4.4 | 🟡 Recovery is a dedicated terminal flow showing recent named snapshots, terminal labels, and simple restore or rerun choices | |
| 🟡 C4.5 | 🟡 Any v0 quick-entry affordance should stay minimal and optional; it is less important than making restore work reliably from inside Ghostty | |
| **C5** | **Legible state model** | ⚠️ |
| 🟡 C5.1 | 🟡 The restore flow must show the selected named Ghostty snapshot clearly: which windows existed, which tabs and panes existed inside them, and the labels for what each terminal was being used for | ⚠️ |
| 🟡 C5.2 | 🟡 Each restored terminal should show enough plain-language context for recognition, such as a label, remembered folder or location, and the last intended command or task | ⚠️ |
| 🟡 C5.3 | 🟡 The restore flow must make it obvious which terminals were restored as-is and which ones need the user to rerun something, including partial Ghostty adapter failures at window, tab, and pane granularity | ⚠️ |
| 🟡 C5.4 | 🟡 The product should avoid overexplaining with dashboard-like status ideas in v0 and instead focus on “restored” versus “needs rerun” at the moment of recovery | ⚠️ |
| **C6** | **Low-config onboarding** | |
| C6.1 | First run focuses on one value proposition: your workspaces come back with enough context to continue | |
| 🟡 C6.2 | 🟡 Instead of a canned template, users should be able to arrange Ghostty the way they want and then save that exact arrangement as the workspace to bring back later | |
| C6.3 | 🟡 Shell integration is guided or auto-detected because Ghostty recovery quality depends on cwd and title metadata being available | ⚠️ |
| C6.4 | First-run onboarding walks the user through one successful creation-and-recovery loop before introducing advanced controls | |
| 🟡 C6.5 | 🟡 v0 does not need extra power-user features beyond reliable save, restore, and switch flows | |

---

## Detail C: User Journeys

| ID | Journey | Entry Point | Outcome |
|----|---------|-------------|---------|
| 🟡 JY1 | 🟡 Save the current Ghostty setup as one workspace, even when it spans multiple windows, tabs, panes, and folders | 🟡 In-terminal save command or TUI | 🟡 The exact arrangement the user is happy with becomes the workspace they can bring back later |
| 🟡 JY2 | 🟡 Reopen recent work after closing Ghostty, after a crash, or after restarting the laptop | 🟡 Opening Ghostty plus an obvious restore command or terminal TUI | 🟡 User sees recent named snapshots and can restore the right one with plain-language choices instead of rebuilding it manually |
| 🟡 JY3 | 🟡 Switch from one saved Ghostty setup to another, where one setup may represent a whole project with many windows and tabs | 🟡 In-terminal switch command or TUI | 🟡 Focus moves to the chosen saved setup and the user regains the intended arrangement quickly |
| 🟡 JY4 | 🟡 Review one named snapshot just enough to confirm “yes, bring that back” | 🟡 Restore flow | 🟡 User sees a simple summary of windows, tabs, panes, and labels before restoring, without needing a separate inspect screen |
| 🟡 JY5 | 🟡 Recover when some prior processes cannot be proven alive | 🟡 Recovery flow | 🟡 Product preserves recognition of prior intent and offers explicit relaunch choices instead of pretending exact continuity exists |
| 🟡 JY6 | 🟡 Learn the product through a Chrome-like “bring my setup back” mental model instead of terminal jargon | 🟡 Onboarding flow after install or first launch | 🟡 User completes one save-and-restore loop and understands that Ghostty can reopen like a browser restoring tabs |

### Journey Notes

- `JY1` and `JY6` are the default-path proof that `R0`, `R1`, `R3`, and `R6` are being met together rather than separately.
- `JY2` and `JY5` are the core proof that Shape `C` is about recognition-first continuity, not hidden-session resurrection theater.
- `JY3` and `JY4` are what make the product feel better than a mux: users switch and orient through recognizable saved setups, not by mentally reconstructing pane topology.

---

## Detail C: Places

| # | Place | Description |
|---|-------|-------------|
| 🟡 P1 | 🟡 Terminal Launcher TUI | 🟡 Primary in-terminal entry point for save, restore, switch, and rename actions |
| 🟡 P2 | 🟡 Ghostty Workspace Surface | 🟡 The active terminal experience inside Ghostty where labeled terminals, titles, and workspace context are visible |
| 🟡 P3 | 🟡 Recovery Flow | 🟡 Terminal-hosted recovery view that appears after interruption or on explicit recover action |
| 🟡 P5 | 🟡 Onboarding Flow | 🟡 First-run guided path that teaches one successful create-and-recover loop |
| 🟡 P6 | 🟡 Workspace Registry + Recovery Engine | 🟡 Non-UI continuity layer that stores snapshots, computes state, and drives orchestration |
| 🟡 P7 | 🟡 Ghostty macOS Adapter | 🟡 AppleScript-backed boundary for creating, focusing, and querying Ghostty windows, tabs, and splits |

---

## Detail C: UI Affordances

| # | Place | Affordance | Control | Wires Out | Returns To |
|---|-------|------------|---------|-----------|------------|
| 🟡 U1 | 🟡 P1 | 🟡 Saved setup list | 🟡 arrow keys / search | 🟡 → N1 | 🟡 → P1 |
| 🟡 U2 | 🟡 P1 | 🟡 Save current Ghostty setup | 🟡 submit | 🟡 → N2 | |
| 🟡 U3 | 🟡 P1 | 🟡 Restore selected saved setup | 🟡 submit | 🟡 → N3 | |
| 🟡 U4 | 🟡 P1 | 🟡 Switch to another saved setup | 🟡 submit | 🟡 → N4 | |
| 🟡 U5 | 🟡 P1 | 🟡 Rename saved setup | 🟡 submit | 🟡 → N5 | 🟡 → P1 |
| 🟡 U7 | 🟡 P2 | 🟡 Ghostty window title shows workspace name and active terminal role | 🟡 visible state | | 🟡 → P2 |
| 🟡 U8 | 🟡 P2 | 🟡 Labeled terminals or tabs showing role and remembered location | 🟡 visible state | | 🟡 → P2 |
| 🟡 U9 | 🟡 P2 | 🟡 Open save / restore TUI command | 🟡 command | 🟡 → P1 | |
| 🟡 U10 | 🟡 P2 | 🟡 Open recovery TUI command | 🟡 command | 🟡 → P3 | |
| 🟡 U11 | 🟡 P3 | 🟡 Restore summary showing saved windows, tabs, panes, and labels | 🟡 visible state | | 🟡 → P3 |
| 🟡 U12 | 🟡 P3 | 🟡 Restore everything possible | 🟡 submit | 🟡 → N6 | |
| 🟡 U13 | 🟡 P3 | 🟡 Rerun one missing terminal | 🟡 submit | 🟡 → N7 | 🟡 → P3 |
| 🟡 U14 | 🟡 P3 | 🟡 Skip restore for now | 🟡 submit | 🟡 → N8 | |
| 🟡 U15 | 🟡 P5 | 🟡 Save-your-current-setup prompt | 🟡 submit | 🟡 → N9 | |
| 🟡 U16 | 🟡 P5 | 🟡 Location and label helper prompt | 🟡 submit | 🟡 → N10 | 🟡 → P5 |
| 🟡 U17 | 🟡 P5 | 🟡 Complete first restore walkthrough | 🟡 submit | 🟡 → N11 | |

---

## Detail C: Non-UI Affordances

| # | Place | Affordance | Wires Out | Returns To |
|---|-------|------------|-----------|------------|
| 🟡 N1 | 🟡 P6 | 🟡 List saved setups and filter by name, label, or remembered location | 🟡 → U1 | 🟡 → P1 |
| 🟡 N2 | 🟡 P6 | 🟡 Save the current Ghostty layout, labels, remembered locations, and intended commands as one workspace snapshot | 🟡 → N12 | |
| 🟡 N3 | 🟡 P6 | 🟡 Restore the selected saved setup by invoking the Ghostty adapter to rebuild windows, tabs, and panes | 🟡 → N12, 🟡 → N13 | |
| 🟡 N4 | 🟡 P6 | 🟡 Switch to another saved setup by focusing or rebuilding the selected Ghostty arrangement | 🟡 → N12, 🟡 → N13 | |
| 🟡 N5 | 🟡 P6 | 🟡 Rename a saved setup and persist the updated label | 🟡 → N12 | 🟡 → P1 |
| 🟡 N6 | 🟡 P6 | 🟡 Restore everything that can be brought back directly from the selected named snapshot | 🟡 → N13 | 🟡 → P2 |
| 🟡 N7 | 🟡 P6 | 🟡 Rerun one missing terminal from its last remembered command or task | 🟡 → N13 | 🟡 → P3 |
| 🟡 N8 | 🟡 P6 | 🟡 Skip restore while keeping the latest snapshot available for later | | |
| 🟡 N9 | 🟡 P6 | 🟡 Save the user's current Ghostty setup during onboarding as their first real workspace | 🟡 → N12 | 🟡 → P5 |
| 🟡 N10 | 🟡 P6 | 🟡 Capture helpful labels and remembered locations so the restore view is easy to recognize later | 🟡 → N12 | 🟡 → P5 |
| 🟡 N11 | 🟡 P6 | 🟡 Trigger a first restore walkthrough so the user sees the save-and-restore loop end to end | 🟡 → U11 | |
| 🟡 N12 | 🟡 P6 | 🟡 Persist `workspace_snapshot` and `terminal_snapshot` with labels, remembered locations, layout, and recovery hints | | |
| 🟡 N13 | 🟡 P7 | 🟡 Create, focus, and query Ghostty windows, tabs, and splits through AppleScript automation so a saved setup can be rebuilt inside Ghostty | 🟡 → U7, 🟡 → U8 | 🟡 → P2 |

---

## Detail C: Affordances Made Possible

| ID | Affordance Made Possible | Enabled By |
|----|--------------------------|------------|
| 🟡 AF1 | 🟡 “Open `frontend`” can mean “focus the right Ghostty context” instead of “reattach a session you must remember exists” | 🟡 `C1`, `C2`, `N2`, `N12` |
| 🟡 AF2 | 🟡 Recovery can present plain-language terminal labels such as `server`, `tests`, or `notes` rather than opaque pane geometry | 🟡 `C1.2`, `C5.2`, `N12`, `N13` |
| 🟡 AF3 | 🟡 The restore flow can say “this came back” versus “rerun this” without falsely claiming perfect persistence | 🟡 `C2`, `C5`, `N6`, `N7` |
| 🟡 AF4 | 🟡 A user can save the exact Ghostty arrangement they have already built instead of starting from a canned template | 🟡 `C6.2`, `U2`, `N2`, `N9` |
| 🟡 AF5 | 🟡 Onboarding can teach one save-and-restore loop in-product instead of teaching mux vocabulary or configuration upfront | 🟡 `C6`, `U15`, `U16`, `U17`, `N11` |
| 🟡 AF6 | 🟡 Launcher and recovery can both live inside the terminal as TUIs instead of requiring a separate app surface | 🟡 `C3.4`, `C4.1`, `C4.4`, `P1`, `P3` |
| 🟡 AF7 | 🟡 One saved setup can represent a whole project across multiple windows, tabs, panes, and folders | 🟡 `C1.1`, `C1.2`, `JY1`, `JY3` |

---

## Product Jobs Enabled by Shape C

| ID | Job |
|----|-----|
| J1 | Start a structured coding workspace quickly without assembling panes and commands from scratch every time |
| J2 | Survive interruptions such as sleep, reboot, crash, or task switching without losing the shape of work |
| J3 | Resume work with enough context to know what each terminal and workspace was doing |
| J4 | Switch between projects safely without mentally bookkeeping hidden session state |
| J5 | Learn the system through visible actions and defaults instead of docs and memorized commands |
| J6 | Stay keyboard-fast once the basic workflow is already understood |

---

## Architectural Stance

| Part | Mechanism | Flag |
|------|-----------|:----:|
| **ARCH1** | 🟡 Product category is not “general-purpose multiplexer”; it is “Ghostty-native persistent workspace orchestrator” | |
| **ARCH2** | 🟡 Ghostty owns rendering and native terminal affordances; Rust owns workspace identity, terminal roles, launch intent, persistence, recovery, and onboarding | |
| **ARCH3** | 🟡 Workspace model sits above Ghostty-specific tabs/splits so the product can survive Ghostty UI/API evolution | |
| **ARCH4** | 🟡 macOS-first scope is a product decision, not just an implementation shortcut, because it makes the Ghostty-native claim honest in v0 | |
| **ARCH5** | 🟡 The product should enable continuity, relaunch, and recovery through Ghostty, not reimplement a terminal emulator or promise exact process resurrection | |
| **ARCH6** | 🟡 The concrete continuity contract is the two-layer snapshot model: `workspace_snapshot` plus `terminal_snapshot` with explicit provenance and recovery status | |
| 🟡 ARCH7 | 🟡 The minimal v0 surface is two terminal-hosted capabilities only: save or switch flow, and recovery flow | |
| 🟡 ARCH8 | 🟡 v0 launcher and recovery should both be hostable directly in the terminal as a CLI or TUI flow | |
| 🟡 ARCH9 | 🟡 The workspace model and recovery contract must stay separate from any one UI host so the core save-and-restore logic remains stable over time | |

---

## Spikes

## X1 Spike: Ghostty control surface

Standalone spike: [spike-r4-ghostty-native.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-r4-ghostty-native.md)

### Result

| Item | Outcome |
|------|---------|
| **X1-R1** | 🟡 macOS is the v0 platform because Ghostty exposes a meaningful AppleScript automation surface there |
| **X1-R2** | 🟡 Shape C should be implemented as an external Ghostty orchestrator, not an embedded Ghostty extension |
| **X1-R3** | 🟡 `R4` is satisfied for macOS-first shaping, with implementation risk noted but no longer conceptually blocked |

## X2 Spike: R2 continuity contract

Standalone spike: [spike-c1-2-workspace-model.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-c1-2-workspace-model.md)

### Result

| Item | Outcome |
|------|---------|
| **X2-R1** | 🟡 `R2` now has a concrete v0 continuity contract based on recognition-first recovery rather than exact process resurrection |
| **X2-R2** | 🟡 The product model is a two-layer snapshot contract: `workspace_snapshot` plus `terminal_snapshot` |
| **X2-R3** | 🟡 Recovery state is now explicit: `live`, `stopped`, `unknown`, and `relaunchable` |
| **X2-R4** | 🟡 Ghostty-observed data such as cwd and title are recovery hints; workspace identity, terminal roles, launch intent, and recovery metadata are owned by our layer |
| **X2-R5** | 🟡 macOS is the primary supported restore target in v0 because Ghostty's restore and query surface is materially stronger there |

See [spike-c1-2-workspace-model.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-c1-2-workspace-model.md).

## X3 Spike: Native UX surface

Standalone spike: [spike-x3-minimal-companion-ui.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x3-minimal-companion-ui.md)

### Result

| Item | Outcome |
|------|---------|
| 🟡 **X3-R1** | 🟡 v0 should have one primary in-terminal flow for save, switch, recover, and rename actions |
| 🟡 **X3-R2** | 🟡 Recovery requires a dedicated lightweight terminal view showing the saved layout, labels, and relaunch choices |
| 🟡 **X3-R3** | 🟡 The product should avoid a persistent dashboard or extra status surface in v0 |
| 🟡 **X3-R4** | 🟡 Onboarding should teach only one concept first: your saved Ghostty setup comes back with enough context to continue |
| 🟡 **X3-R5** | 🟡 The product should avoid a second shell and instead rely on terminal-hosted save and recovery flows |

See [spike-x3-minimal-companion-ui.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x3-minimal-companion-ui.md).

## X4 Spike: Raycast launcher host

Standalone spike: [spike-x3-raycast-launcher.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x3-raycast-launcher.md)

### Result

| Item | Outcome |
|------|---------|
| **X4-R1** | 🟡 Raycast is a strong fit for the workspace launcher because `List`, `Form`, actions, shortcuts, and search map directly onto create/open/switch/recover flows |
| **X4-R2** | 🟡 Raycast is a strong fit for the lightweight status entry point through a macOS menu bar command and cached refresh model |
| **X4-R3** | 🟡 Raycast can still host a recovery command for internal validation, but that is now secondary because v0 recovery should be terminal-hosted first |
| **X4-R4** | 🟡 Raycast is weak for interruption-driven recovery because auto-open recovery and onboarding remain constrained by Raycast lifecycle and confirmation rules |
| **X4-R5** | 🟡 The correct interpretation is “Raycast is a credible early launcher and status host on macOS,” not “Raycast is the recovery default or final product surface” |

See [spike-x3-raycast-launcher.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x3-raycast-launcher.md).

## X5 Spike: Ghostty restore fidelity

Standalone spike: [spike-x5-ghostty-restore-fidelity.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x5-ghostty-restore-fidelity.md)

### Result

| Item | Outcome |
|------|---------|
| **X5-R1** | 🟡 Ghostty 1.3.1 does expose enough AppleScript structure to rebuild windows, tabs, and splits deterministically within one restore run |
| **X5-R2** | 🟡 The adapter algorithm is now concrete: replay saved windows in order, tabs in order, and panes in saved pane order while keeping returned runtime IDs as in-memory anchors |
| **X5-R3** | 🟡 Visible terminal and tab titles are restorable through `perform action`, while cwd, command, and initial input are directly restorable through `surface configuration` |
| **X5-R4** | 🟡 Partial-failure policy is now shaped: degrade at terminal, tab, or window scope rather than aborting the whole restore unless Ghostty cannot materialize any root surface |

---

## Decision

Proceed with **🟡 Shape C** as a **🟡 macOS-first Ghostty-native workspace orchestrator**.

Delivery stance for v0 validation:
- 🟡 Make save, switch, and recovery terminal-hosted in v0 through a CLI or TUI flow that can be invoked directly from Ghostty.
- 🟡 Accept that the first restore path may be “open Ghostty, run one obvious command, choose a named snapshot, get that setup back” rather than full automatic recovery on app launch.
- 🟡 Do not let any future host redefine the product boundary: the workspace model and recovery contract must stay separate from the UI entry point.

Spike outcomes so far:
- `X1` established the Ghostty-native integration direction and confirmed `macOS first`.
- `X2` is now sufficiently shaped through the continuity contract and workspace metadata model in [spike-c1-2-workspace-model.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-c1-2-workspace-model.md).
- `X3` is now sufficiently shaped through the minimal companion UI model in [spike-x3-minimal-companion-ui.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x3-minimal-companion-ui.md).
- 🟡 `X4` remains historical exploration only and is not part of the selected v0 path.
