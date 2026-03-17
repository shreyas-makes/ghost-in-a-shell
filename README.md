# ghost-in-a-shell

`ghost-in-a-shell` is a Ghostty-native workspace continuity tool for people who live in the terminal.

It is built around one belief: terminal work should be easy to reopen, recognize, and continue after interruptions without forcing users to think like tmux experts.

## The Problem

Terminal work is fragile in practice.

A laptop sleeps. A machine reboots. Ghostty quits. You switch projects for a few hours and lose the thread of what every shell was doing. The problem is not just that processes stop. The problem is that context disappears:

- which project was open
- what each terminal was for
- which command was supposed to be running
- which workspace mattered most
- what can be resumed directly vs what needs to be relaunched

Existing multiplexers solve parts of this, but they usually center the wrong mental model for many users. They ask you to manage sessions, panes, prefixes, attach/detach flows, and terminal topology first, then persistence becomes something you learn later.

`ghost-in-a-shell` flips that around.

The default model is not "manage terminal state."

The default model is "my workspaces come back with enough context to continue."

## What It Does

`ghost-in-a-shell` is designed to sit above Ghostty and provide continuity for terminal work.

Core product goals:

- create named workspaces for real tasks, not raw terminal grids
- preserve enough context to recognize interrupted work quickly
- reopen workspaces inside Ghostty with clear status and relaunch paths
- make recovery legible instead of magical
- keep the workflow keyboard-first without making onboarding expert-only

In practice, that means a workspace can carry:

- project location
- terminal roles like `server`, `editor`, `tests`, or `notes`
- launch intent for each terminal
- layout intent
- last-known recovery state

## Why It Is Valuable

The value is not "more terminal features."

The value is:

- less time reconstructing your setup after interruptions
- less cognitive load when switching between projects
- faster recognition of what was going on before context was lost
- a cleaner path for users who want continuity without adopting mux jargon
- a more native experience for Ghostty users who do not want a separate terminal product

This is a tool for preserving momentum.

## Planned Features

- Named workspaces instead of session-first UX
- Recovery-first reopening flows
- Terminal roles and labels for recognition
- Relaunchable terminal intents for common commands
- Workspace launcher for create, open, switch, recover, and inspect
- Lightweight status surface for orientation and attention
- Ghostty-native orchestration on macOS
- Honest recovery states such as `live`, `stopped`, `unknown`, and `relaunchable`

## Design Principles

- Continuity is the primary job
- Recognition beats recall
- Recovery should be honest, not magical
- Ghostty should remain the terminal surface
- Common workflows should not require multiplexer vocabulary
- The default path should be low-config
- Onboarding should teach one useful mental model first
- Keyboard speed should come after first success, not before it

## Who It Is For

`ghost-in-a-shell` is for:

- developers who spend most of their day in Ghostty
- people who want persistence without adopting heavy tmux workflows
- users who regularly juggle multiple projects and lose terminal context
- anyone who wants terminal continuity to feel native, not bolted on

It is especially aimed at people who think:

- "I do not want to memorize mux commands just to get persistence"
- "I want to reopen work and immediately understand what I was doing"
- "I like Ghostty and want continuity on top of it, not a replacement for it"

## Current Status

This repository is currently a product-shaping and architecture repo. It documents the product direction and the continuity model, but it is not yet a runnable implementation.

The current direction is:

- macOS first
- Ghostty-native orchestration
- workspace-first continuity instead of multiplexer-first UX

## Repository Contents

- [`shaping.md`](./shaping.md): product direction and selected shape
- [`spike-c1-2-workspace-model.md`](./spike-c1-2-workspace-model.md): workspace and terminal continuity model
- [`spike-r4-ghostty-native.md`](./spike-r4-ghostty-native.md): Ghostty-native integration direction
- [`spike-x3-minimal-companion-ui.md`](./spike-x3-minimal-companion-ui.md): minimal companion UX
- [`spike-x3-raycast-launcher.md`](./spike-x3-raycast-launcher.md): Raycast as an early launcher/status host

## Direction

The product is intentionally not trying to become a full terminal multiplexer.

It is trying to become the best way for Ghostty users to:

- start structured work quickly
- survive interruptions
- recover context fast
- continue without rebuilding everything from scratch

## License

No license has been added yet.
