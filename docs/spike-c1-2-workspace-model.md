---
shaping: true
---

# C1.2 Spike: Workspace Metadata Model

## Context

`C1.2` in [shaping.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/shaping.md) is the unresolved boundary between Ghostty's native terminal primitives and our continuity layer:

> Workspace stores cwd targets, labels, terminal roles, launch intent, layout intent, and recovery metadata above Ghostty primitives.

That sentence is directionally right but too vague to build from. If `R2` is going to be honest, we need a concrete answer to four questions:

- What fields belong to the workspace object versus an individual terminal?
- Which fields come from Ghostty versus our own layer?
- Which fields are authoritative versus hints?
- Which fields are shown during recovery so users can recognize work quickly?

## Goal

Define a v0 workspace metadata model that is strong enough to support creation, recovery, inspection, and relaunch without depending on exact live-process resurrection.

## Questions

| # | Question |
|---|----------|
| **C12-Q1** | What fields must a workspace own directly so it remains legible above Ghostty windows, tabs, and splits? |
| **C12-Q2** | What fields must each terminal snapshot own so users can tell what it was for? |
| **C12-Q3** | How should launch intent, layout intent, and recovery state be represented so the model survives Ghostty API changes? |
| **C12-Q4** | Which fields are authoritative, which are observed, and which are user-authored hints? |

## Acceptance

Spike is complete when we can hand a developer one page that defines:

- `workspace_snapshot`
- `terminal_snapshot`
- field ownership and staleness rules
- the exact recovery labels exposed to users
- the boundary between Ghostty primitives and our continuity layer

---

## Findings

### F1: The workspace object has to sit above Ghostty surface IDs

From [spike-r4-ghostty-native.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-r4-ghostty-native.md), Ghostty is the renderer and surface orchestrator, but its windows, tabs, and splits are not a stable product-level identity. So the workspace cannot be "a Ghostty window tree." It has to be a product object that can be re-materialized into Ghostty repeatedly.

Implication:

- Workspace identity must be independent of Ghostty window, tab, and terminal handles
- Layout must be stored as intent, not as a Ghostty-specific serialized tree
- Recovery must tolerate missing or changed Ghostty surfaces

### F2: Recognition beats precision for v0 continuity

The selected shape already says users need to know what each workspace and terminal was for after interruptions. That does not require byte-perfect terminal resurrection. It does require enough metadata that a user can answer quickly:

- Is this the workspace I want?
- What was each terminal doing?
- Can I resume it directly, or do I need to relaunch something?

Implication:

- v0 fields should bias toward recognition metadata
- Fields that are expensive or unreliable to capture should not be part of the core promise

### F3: Cwd, title, and layout are useful but not sufficient

Ghostty can expose working directory and surface naming/title behavior on macOS, and can materialize native windows/tabs/splits there. Those are important inputs, but they do not explain purpose on their own. A terminal in `/app` titled `zsh` is not legible enough.

Implication:

- We need explicit terminal roles such as `server`, `editor`, `tests`, or `notes`
- We need launch intent such as "run `bin/dev` in `/app`" or "open shell in `/infra`"
- We need recovery labels that distinguish "still live" from "only reconstructable"

### F4: Field provenance matters as much as the fields themselves

Recovery UI will be misleading unless we can say where each field came from. A user-authored terminal role is more reliable than an inferred title. A directly launched command template is more reliable than a best-effort "last seen command" observation.

Implication:

- The model should encode provenance classes, not just values
- v0 only needs a few provenance classes: user-authored, configured launch intent, observed from Ghostty/shell, inferred by the app

### F5: We need two related schemas, not one overloaded object

`C1.2` mixes workspace-level meaning with per-terminal meaning. Those need separate snapshots with a narrow relationship:

- A workspace snapshot explains the project-level unit of work
- Terminal snapshots explain the role and recoverability of each surface within that workspace

This keeps the model legible and avoids leaking Ghostty implementation details into the primary user object.

---

## Recommended v0 Model

### `workspace_snapshot`

```json
{
  "id": "ws_01J...",
  "slug": "frontend",
  "display_name": "Frontend",
  "description": "Customer app work",
  "tags": ["web", "customer"],
  "cwd_roots": [
    {
      "path": "/Users/shreyas/src/acme/frontend",
      "label": "app",
      "is_primary": true
    }
  ],
  "layout_intent": {
    "template": "main-plus-stack",
    "terminals": ["editor", "server", "tests", "notes"]
  },
  "launch_mode": "manual",
  "recovery": {
    "status": "relaunchable",
    "interruption_reason": "app_quit",
    "last_active_at": "2026-03-17T09:12:33Z",
    "last_seen_platform": "macos",
    "ghostty_restore_capable": true
  },
  "terminals": ["term_01J...", "term_01K..."]
}
```

### `terminal_snapshot`

```json
{
  "id": "term_01J...",
  "workspace_id": "ws_01J...",
  "role": "server",
  "label": "Rails server",
  "cwd_target": "/Users/shreyas/src/acme/frontend",
  "launch_intent": {
    "kind": "command",
    "command": "bin/dev",
    "relaunchable": true
  },
  "layout_slot": "right_top",
  "ghostty": {
    "surface_title": "bin/dev",
    "last_known_working_directory": "/Users/shreyas/src/acme/frontend"
  },
  "runtime": {
    "status": "unknown",
    "last_seen_at": "2026-03-17T09:12:11Z",
    "exit_code": null
  },
  "provenance": {
    "role": "user_authored",
    "label": "user_authored",
    "cwd_target": "configured",
    "surface_title": "observed",
    "runtime_status": "observed"
  }
}
```

---

## Field Set

### Workspace fields

| Field | Why it exists | Source | Notes |
|------|----------------|--------|-------|
| `id` | Stable product identity | Our layer | Never a Ghostty ID |
| `slug` | Keyboard-friendly selector | Our layer | Human addressable |
| `display_name` | User-facing recognition | Our layer | Primary label in launcher/recovery |
| `description` | Optional purpose summary | User-authored | Useful for similar repos |
| `tags` | Grouping/filtering | User-authored | Secondary, not required |
| `cwd_roots[]` | Project anchors for recovery and creation | Configured or observed | One primary root, optional extras |
| `layout_intent` | Reconstruction plan above Ghostty primitives | Our layer | Store template/slots, not native handles |
| `launch_mode` | Whether workspace auto-launches terminals or opens blank shells | Our layer | `manual`, `template`, `hybrid` |
| `recovery.status` | Top-level confidence signal | Derived by our layer | See statuses below |
| `recovery.interruption_reason` | Explains why user is seeing recovery | Derived by our layer | `app_quit`, `crash`, `reboot`, `unknown` |
| `recovery.last_active_at` | Recency sorting | Observed by our layer | Core recovery signal |
| `recovery.last_seen_platform` | Adapter/debug context | Our layer | Important while macOS-first |
| `recovery.ghostty_restore_capable` | Honest restore affordance | Derived from adapter/platform | Avoids promising unsupported restore paths |

### Terminal fields

| Field | Why it exists | Source | Notes |
|------|----------------|--------|-------|
| `id` | Stable terminal identity within workspace history | Our layer | Persists across relaunches of the role |
| `workspace_id` | Association | Our layer | Foreign key |
| `role` | What this terminal is for | User-authored or template-authored | Required in v0 |
| `label` | Human-readable card title | User-authored or template-authored | Defaults from role |
| `cwd_target` | Where relaunch should begin | Configured or observed | Prefer configured over observed |
| `launch_intent` | How to reconstruct work | Our layer | Command, shell, task, notes, etc. |
| `layout_slot` | Where it belongs in intended layout | Our layer | Logical slot, not Ghostty object path |
| `ghostty.surface_title` | Recognition hint | Observed from Ghostty | Useful but not authoritative |
| `ghostty.last_known_working_directory` | Recognition/debug hint | Observed from Ghostty/shell integration | Can be stale |
| `runtime.status` | Recovery badge | Derived by our layer | `live`, `stopped`, `unknown`, `relaunchable` |
| `runtime.last_seen_at` | Freshness | Observed by our layer | Drives stale messaging |
| `runtime.exit_code` | Failure hint when known | Observed by our layer | Optional |
| `provenance.*` | Trust/staleness explanation | Our layer | Required for honest recovery UI |

---

## Status Model

The recovery surface should use only four terminal statuses in v0:

| Status | Meaning | User promise |
|--------|---------|--------------|
| `live` | We have strong evidence the terminal/process is still active and attached to a current Ghostty surface | User can switch to it directly |
| `stopped` | We know the prior process is gone | Context remains, but process is not running |
| `unknown` | We cannot honestly verify runtime state after interruption | Treat as context-first recovery |
| `relaunchable` | We cannot restore the live process, but we do have enough intent to recreate the terminal purposefully | Offer explicit relaunch action |

Workspace-level status should be derived from its terminals:

- `live` if any critical terminal is `live`
- `relaunchable` if no critical terminal is live but at least one is relaunchable
- `unknown` if evidence is stale or incomplete
- `stopped` if everything is known stopped and nothing is relaunchable

---

## Ownership Boundary

### Ghostty owns

- Rendering
- Window, tab, and split materialization on supported platforms
- Native surface titles and working-directory reporting where available
- Focus and navigation inside live terminal surfaces

### Our layer owns

- Workspace identity
- Workspace naming, tags, and descriptions
- Terminal roles and labels
- Launch intent and relaunch templates
- Layout intent as logical slots/templates
- Recovery status and interruption explanation
- Freshness timestamps and provenance metadata

### We do not promise in v0

- Exact process resurrection
- Full scrollback recovery from Ghostty alone
- Exact replay of prior unmanaged ad hoc shell history
- Cross-platform parity for every recovery affordance

---

## Capture Rules

| Field family | Capture moment | Update path | Staleness risk |
|--------------|----------------|-------------|----------------|
| Workspace identity and labels | Create/edit time | Explicit user action | Low |
| Cwd roots and layout intent | Create/edit time, plus optional observe-and-suggest flow | Explicit edit or template apply | Medium if repos move |
| Ghostty titles and cwd hints | While terminal is live and queryable | Poll or event-driven adapter | Medium because shell integration may be incomplete |
| Runtime status | On create, focus, suspend, app quit, recovery scan | Adapter observation | High after crash/reboot |
| Launch intent | Create time or explicit "save this command as intent" action | Explicit edit | Low if user-authored |

The rule is simple: if a field would materially affect recovery behavior, prefer explicit user-authored or template-authored data over inference.

---

## Recovery Surface Consequences

Each workspace card should show:

- `display_name`
- primary cwd root label/path
- last active time
- top-level recovery status
- interruption reason when relevant
- 2-4 terminal cards with role, label, cwd, and status

Each terminal card should show:

- role or label first
- cwd target second
- launch intent summary third
- a badge for `live`, `stopped`, `unknown`, or `relaunchable`

This keeps recovery aligned with `R2`: recognition first, reconstruction second.

---

## Answers

| Question | Answer |
|----------|--------|
| **C12-Q1** | A workspace must own identity, display naming, cwd roots, layout intent, recovery metadata, and references to its terminal snapshots. |
| **C12-Q2** | A terminal snapshot must own role, label, cwd target, launch intent, logical layout slot, runtime status, and provenance for observed Ghostty fields. |
| **C12-Q3** | Launch and layout should be stored as intent, not as native Ghostty handles. That keeps the model stable if Ghostty's scripting surface changes. |
| **C12-Q4** | User-authored/configured fields are authoritative. Ghostty-observed fields are recognition hints. Derived runtime labels are honest confidence signals, not hard guarantees. |

---

## Risks

| ID | Risk | Consequence |
|----|------|-------------|
| **C12-K1** | Roles and labels require light user or template input | Purely inferred workspaces will feel less legible |
| **C12-K2** | Observed cwd/title metadata may be stale without shell integration | Recovery UI must expose uncertainty clearly |
| **C12-K3** | Terminal identity can drift if users repurpose a live shell ad hoc | "Save as intent" or relabel flows will matter |
| **C12-K4** | Storing too much runtime detail will create fake precision | v0 should stay narrow and recognition-focused |

---

## Recommendation

`C1.2` should be considered shaped for v0 if we adopt this rule:

`workspace = identity + project anchors + layout intent + recovery state`

`terminal = role + cwd target + launch intent + runtime confidence`

That gives us a concrete object model above Ghostty primitives, satisfies the open requirement behind `R2`, and keeps the product honest about what continuity means.

## Proposed Update To Shape C

The most accurate refinement to [shaping.md](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/shaping.md) is:

- `C1.2` should point to this spike as the definition of the workspace metadata contract
- `C5.2` should explicitly depend on terminal `role`, `label`, `cwd_target`, and `launch_intent`
- `X2` should treat `workspace_snapshot` and `terminal_snapshot` as the concrete prototype artifact, not just a placeholder

## Decision

Proceed with a two-layer snapshot model:

- `workspace_snapshot`
- `terminal_snapshot`

And treat Ghostty data as observed recovery context, not the primary source of workspace meaning.

## Sources

- [Ghostty AppleScript docs](https://ghostty.org/docs/features/applescript)
- [Ghostty Shell Integration docs](https://ghostty.org/docs/features/shell-integration)
- [Ghostty macOS save-state docs](https://ghostty.org/docs/help/macos-state-restoration)
- [R4 spike](/Users/shreyas/Desktop/Projects/ghost-in-a-shell/spike-r4-ghostty-native.md)
