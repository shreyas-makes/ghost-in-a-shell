---
shaping: true
---

# X3 Spike: Raycast As Minimal UI Launcher

## Context

[spike-x3-minimal-companion-ui.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x3-minimal-companion-ui.md) concluded that v0 needs exactly three surfaces outside Ghostty:

- workspace launcher
- recovery view
- lightweight status entry point

This spike asks whether a Raycast extension could provide those surfaces closely enough to count as the X3 minimal UI launcher, at least for an early implementation.

## Question

Can a Raycast extension act as the minimal companion UI launcher described in X3, without forcing us into a heavier standalone desktop UI too early?

## Acceptance

Spike is complete when we can say:

- which X3 surfaces map cleanly onto Raycast
- which requirements are only partial fits
- which constraints would make Raycast a poor long-term home even if it works for v0

---

## Findings

### F1: Raycast is structurally a good fit for the launcher itself

Raycast extensions are built with React, Node.js, and TypeScript, and their primary UI surfaces are `view`, `no-view`, and `menu-bar` commands. That lines up well with a keyboard-first launcher and a lightweight status entry point.

For the X3 launcher requirements, Raycast gives us the right primitives:

- searchable `List` views for workspace search and switching
- `Form` views for create and rename flows
- `ActionPanel` actions with keyboard shortcuts
- `launchCommand` for moving between commands inside the extension
- deeplinks and Raycast hotkeys for fast re-entry

This makes the following X3 launcher actions straightforward:

- create workspace
- open workspace
- switch workspace
- rename workspace
- inspect workspace
- relaunch terminals from saved intent

### F2: Raycast can plausibly host the recovery flow, but only inside Raycast’s chrome

X3 requires more than one palette row for recovery. Raycast supports both standalone `Detail` views and `List` views with a built-in detail pane (`List isShowingDetail` plus `List.Item.Detail` metadata/markdown). That is enough to build a compact recovery experience with:

- one row per workspace
- visible top-level status
- terminal summaries in the detail pane
- relaunch actions per workspace or terminal

So the recovery experience is feasible in Raycast.

However, it will look like a Raycast command, not like our own native sheet or lightweight app window. That matters because X3 described recovery as a dedicated product surface opened on startup after interruption or from the launcher. Raycast can provide the "from the launcher" part cleanly. The "open automatically as our own recovery surface" part is much weaker.

### F3: Raycast is a strong fit for the lightweight status entry point

Raycast explicitly supports `menu-bar` commands via `MenuBarExtra`, and background refresh can update them on a schedule. That maps well onto X3’s lightweight status surface:

- current or recent workspace
- recoverable/attention-needed count
- quick actions like `Open launcher`, `Recover`, and `New workspace`

This is one of the best matches in the whole spike.

Two constraints matter:

- menu bar commands are macOS-only
- they are not long-lived processes; Raycast loads and unloads them on demand, so this is suitable for cached status, not high-confidence live telemetry

That is still aligned with X3, which already warns against implying stronger runtime certainty than we actually have.

### F4: Raycast weakens the “product-owned surface” part of X3

Raycast’s official command modes are `view`, `no-view`, and `menu-bar`. In practice that means our UI lives inside Raycast unless we leave Raycast entirely and open another app or URL.

This creates several product constraints:

- the launcher is a Raycast command, not our own app surface
- the recovery flow is a Raycast view, not our own startup sheet
- first-run onboarding is possible, but it is onboarding inside Raycast
- any always-available status entry point depends on Raycast being installed, enabled, and part of the user’s workflow

This is acceptable only if we treat Raycast as an opinionated host shell for early validation, not as proof that the product no longer needs a product-owned entry point.

### F5: Automatic recovery surfacing is the main gap

Raycast does support background refresh for `no-view` and `menu-bar` commands, but scheduled commands are initially disabled until the user opens them once or enables them in preferences. Deeplinks can open commands, but they require confirmation when launched.

Those constraints suggest the following:

- periodic background status refresh is feasible
- fast explicit reopening is feasible
- unprompted, product-owned interruption handling is not a strong fit

This is an inference from the documented lifecycle model: Raycast is optimized for user-invoked commands and refreshed menu-bar utilities, not for an app that autonomously presents its own recovery window at exactly the right moment.

### F6: Raycast is viable for an internal spike or early macOS-only beta

Raycast supports local/private extension development and team distribution, so if the purpose is to validate the X3 launcher shape quickly with low UI engineering cost, it is a practical choice.

It is especially attractive if we want to validate:

- whether users actually prefer a keyboard launcher for workspace operations
- whether a list-plus-detail recovery view is sufficient
- whether a tiny menu bar status entry point is enough for orientation

It is much less attractive if we want to validate:

- a branded product surface distinct from Raycast
- startup recovery behavior we control directly
- onboarding for users who do not already live in Raycast

---

## Requirement Fit

| X3 surface / requirement | Raycast fit | Notes |
|---|---|---|
| Workspace launcher | **Strong** | Natural fit with `List`, `Form`, actions, shortcuts, and search |
| Search by workspace/cwd/labels | **Strong** | Built-in list search is a direct match |
| Rename/create flows | **Strong** | `Form` commands are sufficient |
| Inspect workspace | **Strong** | `Detail` or list detail pane works well |
| Recovery view | **Partial to strong** | Feasible in Raycast, but visually and behaviorally constrained by Raycast |
| Terminal summaries + relaunch actions | **Strong** | Good fit for list detail metadata and actions |
| Lightweight status entry point | **Strong** | `menu-bar` commands match this well on macOS |
| Cached status refresh | **Strong** | Background refresh supports this |
| High-confidence live status | **Weak** | Menu-bar lifecycle is not a persistent runtime surface |
| Auto-open recovery on interruption | **Weak** | Raycast lifecycle and deeplink confirmation make this awkward |
| First-run onboarding | **Partial** | Possible, but inside Raycast rather than a product-owned shell |
| Product feels native to Ghostty rather than to Raycast | **Partial** | Depends on whether we accept Raycast as the host UX |

---

## Recommendation

Raycast can credibly act as the **workspace launcher** from X3 and as the **lightweight status entry point**.

Raycast can also host a **good enough recovery view** for an early spike, but only as a Raycast-native command surface. It is not a strong fit for the stronger interpretation of X3 where recovery is a product-owned surface that can present itself at the right time without depending on Raycast interaction patterns.

So the practical recommendation is:

- use Raycast if the goal is to validate X3 cheaply on macOS with users who already use Raycast
- do not treat Raycast as closing the product question for long-term recovery UX or onboarding

## Decision

**Yes, with limits:** a Raycast extension is a credible vehicle for an X3 launcher spike and could act as the minimal UI launcher for an internal or early beta implementation.

**No, as a final answer:** Raycast should not be assumed to satisfy the full long-term X3 companion surface, because the biggest unresolved gap is product-owned recovery surfacing.

## What This Means For Next Step

If we pursue this path, the right bounded prototype is:

- one Raycast `List` command for create/open/switch/recover/search
- one Raycast recovery command using `List` detail or `Detail`
- one Raycast `menu-bar` command for status and quick actions
- workspace state stored outside Raycast’s small encrypted key-value store, with Raycast storage used only for lightweight preferences or cached UI state

That would let us validate whether the X3 three-surface model works in practice before paying for a standalone macOS companion app.

## Sources

- [X3 minimal companion UI spike](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-x3-minimal-companion-ui.md)
- [Raycast Introduction](https://developers.raycast.com/)
- [Raycast Manifest](https://developers.raycast.com/information/manifest)
- [Raycast User Interface](https://developers.raycast.com/api-reference/user-interface)
- [Raycast List](https://developers.raycast.com/api-reference/user-interface/list)
- [Raycast Detail](https://developers.raycast.com/api-reference/user-interface/detail)
- [Raycast Action Panel](https://developers.raycast.com/api-reference/user-interface/action-panel)
- [Raycast Command / `launchCommand`](https://developers.raycast.com/api-reference/command)
- [Raycast Menu Bar Commands](https://developers.raycast.com/api-reference/menu-bar-commands)
- [Raycast Background Refresh](https://developers.raycast.com/information/lifecycle/background-refresh)
- [Raycast Deeplinks](https://developers.raycast.com/information/lifecycle/deeplinks)
- [Raycast Storage](https://developers.raycast.com/api-reference/storage)
- [Raycast Getting Started](https://developers.raycast.com/basics/getting-started)
- [Raycast Teams: Publish a Private Extension](https://developers.raycast.com/teams/publish-a-private-extension)
