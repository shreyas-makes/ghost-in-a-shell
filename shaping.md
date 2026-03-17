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
| **C4** | Provide a simple launcher or command palette for create, switch, recover, and inspect actions | |
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
- 🟡 The companion UI shape is now clear, but the launcher and status hosts remain open: v0 recovery should be terminal-hosted first, while Raycast remains an optional macOS launcher and status host.

---

## C x R

| Shape | R0 | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 |
|-------|----|----|----|----|----|----|----|----|----|
| C | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**Notes:**
- `R2` passes for Shape `C` because the current selected shape now has a concrete recognition-first continuity contract: `workspace_snapshot` plus `terminal_snapshot`, explicit recovery states, and relaunch actions where exact continuity cannot be proven.
- `R4` passes for Shape `C` on a macOS-first basis because Ghostty remains the visible terminal surface while the product adds continuity through an external orchestration layer using Ghostty's AppleScript automation.
- Remaining uncertainty is now implementation risk inside Shape `C`, not a fit-check failure: `C5`, `C6`, `C2.2`, `C2.3`, `C4.6`, and `C6.3` still carry `⚠️` and should be resolved or further spiked during detailing and slicing.

---

## Detail C: Concrete Product Shape

| Part | Mechanism | Flag |
|------|-----------|:----:|
| **C1** | **Workspace model** | |
| C1.1 | Named workspace with user-facing identity such as `api`, `frontend`, or `notes` | |
| C1.2 | 🟡 Workspace stores cwd targets, labels, terminal roles, launch intent, layout intent, and recovery metadata above Ghostty primitives. See [spike-c1-2-workspace-model.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-c1-2-workspace-model.md). | ⚠️ |
| C1.3 | Workspace is the primary thing users create, switch, recover, and inspect | |
| **C2** | **Continuity-first lifecycle** | |
| C2.1 | New workspace creation defaults to recoverable behavior | |
| C2.2 | 🟡 Resume flow reopens prior work without requiring an attach command and prioritizes recognition over exact process resurrection, with relaunch actions where continuity cannot be proven | ⚠️ |
| C2.3 | Interruption states such as closed app, reboot, or crash reopen into a recovery flow instead of a blank terminal, with terminal-hosted recovery as the v0 default | ⚠️ |
| **C3** | **Ghostty-native surface (macOS first)** | ⚠️ |
| C3.1 | 🟡 Ghostty remains the only terminal renderer and visible shell surface users interact with | |
| C3.2 | 🟡 Rust daemon or CLI owns workspace registry, persistence store, recovery logic, and workspace metadata above Ghostty primitives | |
| C3.3 | 🟡 macOS adapter uses Ghostty AppleScript automation to create, focus, and query windows, tabs, and splits | |
| C3.4 | 🟡 Minimal companion UI exists only for workspace operations Ghostty does not already expose well, with v0 recovery allowed to render in-terminal and optional launcher or status surfaces kept outside Ghostty | |
| C3.5 | 🟡 Ghostty preview-scriptability risk is accepted, so adapter boundaries must stay narrow and swappable | |
| C3.6 | 🟡 Linux and other platforms are deferred until the workspace model and recovery UX are proven on macOS | |
| C3.7 | The product avoids a second competing visual shell where possible and adds continuity rather than replacing terminal interaction | |
| **C4** | **Human command surface** | |
| C4.1 | Command palette or launcher for create, open, recover, rename, and inspect; this is the primary product entry point outside Ghostty | |
| C4.2 | Recognition-first action labels replace obscure mux verbs | |
| C4.3 | Shortcut hints appear in context after first-run success rather than front-loading complexity | |
| C4.4 | 🟡 Recovery is a dedicated flow with workspace summaries, terminal summaries, status badges, and relaunch actions; in v0 this should render in the terminal rather than depend on Raycast | |
| C4.5 | 🟡 A lightweight status entry point provides quick orientation and access to launcher or recovery without becoming a persistent dashboard | |
| C4.6 | 🟡 A Raycast extension is an acceptable macOS-first delivery vehicle for launcher and status, but recovery should be terminal-hosted in v0 and not depend on Raycast | ⚠️ |
| **C5** | **Legible state model** | ⚠️ |
| C5.1 | Users can see active workspace, recent workspaces, whether a workspace needs attention, and whether the workspace is `live`, `unknown`, `stopped`, or `relaunchable` | ⚠️ |
| C5.2 | 🟡 Users can see what each terminal was for through explicit terminal roles, labels, cwd targets, Ghostty title, and last-known launch intent | ⚠️ |
| C5.3 | 🟡 Recovery flow distinguishes restored context from processes that must be relaunched, even when rendered in-terminal | ⚠️ |
| C5.4 | 🟡 Status labels such as `live`, `stopped`, `unknown`, and `relaunchable` communicate confidence instead of implying perfect continuity | ⚠️ |
| **C6** | **Low-config onboarding** | |
| C6.1 | First run focuses on one value proposition: your workspaces come back with enough context to continue | |
| C6.2 | Default template or starter workspace helps users succeed in under two minutes | |
| C6.3 | 🟡 Shell integration is guided or auto-detected because Ghostty recovery quality depends on cwd and title metadata being available | ⚠️ |
| C6.4 | First-run onboarding walks the user through one successful creation-and-recovery loop before introducing advanced controls | |
| C6.5 | Power-user features are present but not required to understand the product | |

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
| **ARCH7** | 🟡 The minimal companion surface is three capabilities only: workspace launcher, recovery flow, and lightweight status entry point | |
| **ARCH8** | 🟡 v0 recovery should be hostable directly in the terminal as a CLI or TUI flow, while launcher and status remain free to use Raycast or another macOS host | |
| **ARCH9** | 🟡 The workspace model and recovery contract must stay host-agnostic so terminal, Raycast, and future product-owned shells can all sit on the same continuity layer | |

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
| **X3-R1** | 🟡 v0 should have one primary launcher for create/open/switch/recover/rename/inspect actions |
| **X3-R2** | 🟡 Recovery requires a dedicated lightweight view with workspace cards, terminal summaries, status badges, and relaunch actions |
| **X3-R3** | 🟡 The always-available status surface should stay minimal: active or recent workspace, attention state, and quick entry back into launcher or recovery |
| **X3-R4** | 🟡 Onboarding should teach only one concept first: named workspaces come back with enough context to continue |
| **X3-R5** | 🟡 The product should avoid a persistent dashboard or second shell and instead rely on a three-surface companion model |

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

---

## Decision

Proceed with **🟡 Shape C** as a **🟡 macOS-first Ghostty-native workspace orchestrator**.

Delivery stance for v0 validation:
- Make recovery terminal-hosted in v0 through a CLI or TUI flow that can be invoked directly from the terminal experience.
- Use a Raycast extension only for launcher and status if that is the fastest way to validate macOS entry points.
- Do not let any host redefine the product boundary: the workspace model, recovery contract, and companion-surface architecture must stay portable across terminal, Raycast, and future native shells.

Spike outcomes so far:
- `X1` established the Ghostty-native integration direction and confirmed `macOS first`.
- `X2` is now sufficiently shaped through the continuity contract and workspace metadata model in [spike-c1-2-workspace-model.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-c1-2-workspace-model.md).
- `X3` is now sufficiently shaped through the minimal companion UI model in [spike-x3-minimal-companion-ui.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x3-minimal-companion-ui.md).
- `X4` established that Raycast is a viable macOS-first host for launcher and status, but not the right default host for v0 recovery.
