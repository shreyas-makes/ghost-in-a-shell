use std::collections::{HashMap, HashSet};
use std::process::Command;

use serde::Deserialize;
use uuid::Uuid;

use crate::model::{
    PaneSnapshot, PaneTopologyHint, RestoreStatus, TabSnapshot, TerminalSnapshot, WindowSnapshot,
    WorkspaceSnapshot,
};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GhosttyCapture {
    pub windows: Vec<CapturedWindow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CapturedWindow {
    pub ghostty_id: String,
    pub window_index: usize,
    pub tabs: Vec<CapturedTab>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CapturedTab {
    pub ghostty_id: String,
    pub tab_index: usize,
    pub title: Option<String>,
    pub terminals: Vec<CapturedTerminal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CapturedTerminal {
    pub ghostty_id: String,
    pub pane_index: usize,
    pub title: Option<String>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppleScriptGhosttyAdapter;

#[derive(Debug, Clone, PartialEq, Eq)]
struct GhosttyRefs {
    window_id: String,
    tab_id: String,
    terminal_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CapturedAdjacency {
    left: Option<String>,
    right: Option<String>,
    up: Option<String>,
    down: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RestorePlanStep {
    terminal_id: Uuid,
    kind: RestorePlanKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RestorePlanKind {
    WindowRoot,
    TabRoot,
    SplitFrom {
        anchor_terminal_id: Uuid,
        direction: &'static str,
    },
    FallbackLinear,
}

impl AppleScriptGhosttyAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn capture_workspace(&self, name: &str) -> Result<WorkspaceSnapshot> {
        let capture = self.capture_state()?;
        let slug = crate::store::normalize_slug(name)?;

        let mut windows = Vec::new();
        let mut terminals = Vec::new();

        for captured_window in capture.windows {
            let mut tabs = Vec::new();

            for captured_tab in captured_window.tabs {
                let neighbor_hints = self.capture_tab_adjacency(
                    captured_window.ghostty_id.as_str(),
                    captured_tab.ghostty_id.as_str(),
                    &captured_tab.terminals,
                )?;

                let mut panes = Vec::new();
                let mut saved_ids_by_ghostty_id = HashMap::new();

                for captured_terminal in &captured_tab.terminals {
                    let terminal = TerminalSnapshot::new(
                        captured_terminal.title.clone(),
                        None,
                        captured_terminal.working_directory.clone(),
                        captured_terminal.title.clone(),
                        None,
                    );
                    saved_ids_by_ghostty_id
                        .insert(captured_terminal.ghostty_id.clone(), terminal.terminal_id);
                    panes.push(PaneSnapshot {
                        pane_index: captured_terminal.pane_index,
                        terminal_id: terminal.terminal_id,
                        layout_slot: format!("pane-{}", captured_terminal.pane_index + 1),
                        topology: PaneTopologyHint::default(),
                    });
                    terminals.push(terminal);
                }

                for captured_terminal in &captured_tab.terminals {
                    if let Some(neighbors) =
                        neighbor_hints.get(captured_terminal.ghostty_id.as_str())
                    {
                        if let Some(saved_terminal_id) =
                            saved_ids_by_ghostty_id.get(captured_terminal.ghostty_id.as_str())
                        {
                            if let Some(pane) = panes
                                .iter_mut()
                                .find(|pane| pane.terminal_id == *saved_terminal_id)
                            {
                                pane.topology = PaneTopologyHint {
                                    left: neighbors
                                        .left
                                        .as_ref()
                                        .and_then(|id| saved_ids_by_ghostty_id.get(id))
                                        .copied(),
                                    right: neighbors
                                        .right
                                        .as_ref()
                                        .and_then(|id| saved_ids_by_ghostty_id.get(id))
                                        .copied(),
                                    up: neighbors
                                        .up
                                        .as_ref()
                                        .and_then(|id| saved_ids_by_ghostty_id.get(id))
                                        .copied(),
                                    down: neighbors
                                        .down
                                        .as_ref()
                                        .and_then(|id| saved_ids_by_ghostty_id.get(id))
                                        .copied(),
                                };
                            }
                        }
                    }
                }

                tabs.push(TabSnapshot {
                    tab_index: captured_tab.tab_index,
                    title: captured_tab.title,
                    panes,
                });
            }

            windows.push(WindowSnapshot {
                window_index: captured_window.window_index,
                tabs,
            });
        }

        if terminals.is_empty() {
            return Err(Error::Adapter(
                "Ghostty capture returned no terminals".into(),
            ));
        }

        Ok(WorkspaceSnapshot::new(
            name.into(),
            slug,
            windows,
            terminals,
        ))
    }

    pub fn restore_workspace(
        &self,
        snapshot: &WorkspaceSnapshot,
        run_commands: bool,
    ) -> Result<WorkspaceSnapshot> {
        let mut restored = snapshot.clone();

        for window in &snapshot.windows {
            let mut window_ref: Option<GhosttyRefs> = None;

            for tab in &window.tabs {
                let restore_plan = build_tab_restore_plan(tab);
                let mut tab_anchor: Option<GhosttyRefs> = None;
                let mut created_by_terminal = HashMap::<Uuid, GhosttyRefs>::new();

                for step in restore_plan {
                    let pane = tab
                        .panes
                        .iter()
                        .find(|pane| pane.terminal_id == step.terminal_id)
                        .ok_or_else(|| {
                            Error::Adapter("restore plan referenced unknown pane".into())
                        })?;
                    let terminal = snapshot
                        .terminal_by_id(pane.terminal_id)
                        .ok_or_else(|| Error::Adapter("pane referenced unknown terminal".into()))?;

                    let creation_result = match &step.kind {
                        RestorePlanKind::WindowRoot => {
                            self.new_window(terminal.working_directory.as_deref())
                        }
                        RestorePlanKind::TabRoot => match window_ref.as_ref() {
                            Some(window_ref) => self.new_tab(
                                window_ref.window_id.as_str(),
                                terminal.working_directory.as_deref(),
                            ),
                            None => Err(Error::Adapter("window ref missing".into())),
                        },
                        RestorePlanKind::SplitFrom {
                            anchor_terminal_id,
                            direction,
                        } => match created_by_terminal.get(anchor_terminal_id) {
                            Some(anchor) => self.split_terminal(
                                anchor,
                                direction,
                                terminal.working_directory.as_deref(),
                            ),
                            None => Err(Error::Adapter("planned split anchor missing".into())),
                        },
                        RestorePlanKind::FallbackLinear => match tab_anchor.as_ref() {
                            Some(anchor) => self.split_terminal(
                                anchor,
                                direction_for_layout_slot(&pane.layout_slot),
                                terminal.working_directory.as_deref(),
                            ),
                            None => Err(Error::Adapter("tab anchor terminal missing".into())),
                        },
                    };

                    let (status, note) = match creation_result {
                        Ok(created_refs) => {
                            if window_ref.is_none() {
                                window_ref = Some(created_refs.clone());
                            }
                            if tab_anchor.is_none() {
                                tab_anchor = Some(created_refs.clone());
                                if let Some(title) =
                                    tab.title.as_deref().filter(|title| !title.is_empty())
                                {
                                    let _ = self
                                        .set_tab_title(created_refs.terminal_id.as_str(), title);
                                }
                            }
                            created_by_terminal.insert(pane.terminal_id, created_refs.clone());

                            if let Some(title) = terminal
                                .surface_title
                                .as_deref()
                                .or(terminal.label.as_deref())
                                .filter(|title| !title.is_empty())
                            {
                                let _ = self
                                    .set_surface_title(created_refs.terminal_id.as_str(), title);
                            }

                            restore_status_for_terminal(terminal, run_commands, || {
                                self.send_text_to_terminal(
                                    created_refs.terminal_id.as_str(),
                                    terminal
                                        .launch_intent
                                        .as_deref()
                                        .expect("launch intent checked before execution"),
                                )
                            })
                        }
                        Err(error) => (
                            RestoreStatus::NeedsRerun,
                            Some(format!("surface restore failed: {error}")),
                        ),
                    };

                    if let Some(restored_terminal) =
                        restored.terminal_by_id_mut(terminal.terminal_id)
                    {
                        restored_terminal.restore_status = Some(status);
                        restored_terminal.restore_note = note;
                    }
                }
            }
        }

        Ok(restored)
    }

    pub fn send_text_to_terminal(&self, terminal_id: &str, text: &str) -> Result<()> {
        let script = format!(
            "tell application \"Ghostty\"\n\
             set targetTerm to first terminal whose id is {}\n\
             input text {} to targetTerm\n\
             send key \"enter\" to targetTerm\n\
             end tell",
            applescript_string(terminal_id),
            applescript_string(text)
        );
        self.run_script(&script).map(|_| ())
    }

    pub fn set_surface_title(&self, terminal_id: &str, title: &str) -> Result<()> {
        self.perform_action_on_terminal(terminal_id, &format!("set_surface_title:{title}"))
    }

    pub fn set_tab_title(&self, terminal_id: &str, title: &str) -> Result<()> {
        self.perform_action_on_terminal(terminal_id, &format!("set_tab_title:{title}"))
    }

    fn capture_state(&self) -> Result<GhosttyCapture> {
        let script = r#"
on replace_text(find_text, replace_text, subject)
    set AppleScript's text item delimiters to find_text
    set parts to every text item of subject
    set AppleScript's text item delimiters to replace_text
    set subject to parts as text
    set AppleScript's text item delimiters to ""
    return subject
end replace_text

on json_quote(value_text)
    set escaped_text to my replace_text("\\", "\\\\", value_text)
    set escaped_text to my replace_text("\"", "\\\"", escaped_text)
    return "\"" & escaped_text & "\""
end json_quote

tell application "Ghostty"
    set output to "{\"windows\":["
    set window_count to count of windows
    repeat with window_index from 1 to window_count
        set win_ref to window window_index
        set output to output & "{\"ghostty_id\":" & my json_quote((id of win_ref as text)) & ",\"window_index\":" & (window_index - 1) & ",\"tabs\":["
        set tab_count to count of tabs of win_ref
        repeat with tab_index from 1 to tab_count
            set tab_ref to tab tab_index of win_ref
            set tab_name to name of tab_ref
            if tab_name is missing value then
                set tab_json to "null"
            else
                set tab_json to my json_quote(tab_name as text)
            end if

            set output to output & "{\"ghostty_id\":" & my json_quote((id of tab_ref as text)) & ",\"tab_index\":" & (tab_index - 1) & ",\"title\":" & tab_json & ",\"terminals\":["
            set term_count to count of terminals of tab_ref
            repeat with pane_index from 1 to term_count
                set term_ref to terminal pane_index of tab_ref
                set term_name to name of term_ref
                set term_cwd to working directory of term_ref

                if term_name is missing value then
                    set term_name_json to "null"
                else
                    set term_name_json to my json_quote(term_name as text)
                end if

                if term_cwd is missing value then
                    set term_cwd_json to "null"
                else
                    set term_cwd_json to my json_quote(term_cwd as text)
                end if

                set output to output & "{\"ghostty_id\":" & my json_quote((id of term_ref as text)) & ",\"pane_index\":" & (pane_index - 1) & ",\"title\":" & term_name_json & ",\"working_directory\":" & term_cwd_json & "}"
                if pane_index is not term_count then
                    set output to output & ","
                end if
            end repeat
            set output to output & "]}"
            if tab_index is not tab_count then
                set output to output & ","
            end if
        end repeat
        set output to output & "]}"
        if window_index is not window_count then
            set output to output & ","
        end if
    end repeat
    set output to output & "]}"
    return output
end tell
"#;
        let raw = self.run_script(script)?;
        parse_capture_output(&raw)
    }

    fn capture_tab_adjacency(
        &self,
        window_id: &str,
        tab_id: &str,
        terminals: &[CapturedTerminal],
    ) -> Result<HashMap<String, CapturedAdjacency>> {
        let mut adjacency = HashMap::new();
        for terminal in terminals {
            let neighbors =
                self.capture_terminal_neighbors(window_id, tab_id, terminal.ghostty_id.as_str())?;
            adjacency.insert(terminal.ghostty_id.clone(), neighbors);
        }
        Ok(adjacency)
    }

    fn capture_terminal_neighbors(
        &self,
        window_id: &str,
        tab_id: &str,
        terminal_id: &str,
    ) -> Result<CapturedAdjacency> {
        let script = format!(
            "tell application \"Ghostty\"\n\
             set winRef to first window whose id is {}\n\
             activate window winRef\n\
             set tabRef to first tab of winRef whose id is {}\n\
             select tab tabRef\n\
             set targetTerm to first terminal of tabRef whose id is {}\n\
             set out to \"\"\n\
             repeat with dir in {{\"left\", \"right\", \"up\", \"down\"}}\n\
               focus targetTerm\n\
               delay 0.03\n\
               perform action (\"goto_split:\" & dir) on targetTerm\n\
               delay 0.05\n\
               set focusedId to id of focused terminal of tabRef as text\n\
               if focusedId is not (id of targetTerm as text) then\n\
                 set out to out & (dir as text) & \":\" & focusedId & \"|\"\n\
               end if\n\
             end repeat\n\
             return out\n\
             end tell",
            applescript_string(window_id),
            applescript_string(tab_id),
            applescript_string(terminal_id)
        );
        let raw = self.run_script(&script)?;
        Ok(parse_captured_adjacency(raw.trim()))
    }

    fn new_window(&self, cwd: Option<&str>) -> Result<GhosttyRefs> {
        let cfg = applescript_config(cwd);
        let script = format!(
            "tell application \"Ghostty\"\n\
             activate\n\
             {cfg}\n\
             set winRef to new window with configuration cfg\n\
             set tabRef to selected tab of winRef\n\
             set termRef to terminal 1 of tabRef\n\
             return (id of winRef as text) & \"|\" & (id of tabRef as text) & \"|\" & (id of termRef as text)\n\
             end tell"
        );
        self.run_script(&script)
            .and_then(|raw| parse_ghostty_refs(&raw))
    }

    fn new_tab(&self, window_id: &str, cwd: Option<&str>) -> Result<GhosttyRefs> {
        let cfg = applescript_config(cwd);
        let script = format!(
            "tell application \"Ghostty\"\n\
             {cfg}\n\
             set winRef to first window whose id is {}\n\
             set tabRef to new tab in winRef with configuration cfg\n\
             set termRef to terminal 1 of tabRef\n\
             return (id of winRef as text) & \"|\" & (id of tabRef as text) & \"|\" & (id of termRef as text)\n\
             end tell",
            applescript_string(window_id)
        );
        self.run_script(&script)
            .and_then(|raw| parse_ghostty_refs(&raw))
    }

    fn split_terminal(
        &self,
        anchor: &GhosttyRefs,
        direction: &str,
        cwd: Option<&str>,
    ) -> Result<GhosttyRefs> {
        let cfg = applescript_config(cwd);
        let script = format!(
            "tell application \"Ghostty\"\n\
             {cfg}\n\
             set anchorTerm to first terminal whose id is {}\n\
             set newTerm to split anchorTerm direction {direction} with configuration cfg\n\
             return {} & \"|\" & {} & \"|\" & (id of newTerm as text)\n\
             end tell",
            applescript_string(anchor.terminal_id.as_str()),
            applescript_string(anchor.window_id.as_str()),
            applescript_string(anchor.tab_id.as_str())
        );
        self.run_script(&script)
            .and_then(|raw| parse_ghostty_refs(&raw))
    }

    fn perform_action_on_terminal(&self, terminal_id: &str, action: &str) -> Result<()> {
        let script = format!(
            "tell application \"Ghostty\"\n\
             set targetTerm to first terminal whose id is {}\n\
             perform action {} on targetTerm\n\
             end tell",
            applescript_string(terminal_id),
            applescript_string(action)
        );
        self.run_script(&script).map(|_| ())
    }

    fn run_script(&self, script: &str) -> Result<String> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|error| Error::Adapter(format!("failed to invoke osascript: {error}")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(Error::Adapter(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }
}

pub fn parse_capture_output(raw: &str) -> Result<GhosttyCapture> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(Error::Adapter("empty Ghostty capture output".into()));
    }
    Ok(serde_json::from_str(trimmed)?)
}

fn applescript_string(input: &str) -> String {
    format!(
        "\"{}\"",
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}

fn applescript_config(cwd: Option<&str>) -> String {
    let mut lines = vec!["set cfg to new surface configuration".to_string()];
    if let Some(cwd) = cwd {
        lines.push(format!(
            "set initial working directory of cfg to {}",
            applescript_string(cwd)
        ));
    }
    lines.join("\n")
}

fn parse_ghostty_refs(raw: &str) -> Result<GhosttyRefs> {
    let mut parts = raw.trim().split('|');
    let window_id = parts
        .next()
        .ok_or_else(|| Error::Adapter("missing Ghostty window id".into()))?
        .to_string();
    let tab_id = parts
        .next()
        .ok_or_else(|| Error::Adapter("missing Ghostty tab id".into()))?
        .to_string();
    let terminal_id = parts
        .next()
        .ok_or_else(|| Error::Adapter("missing Ghostty terminal id".into()))?
        .to_string();
    Ok(GhosttyRefs {
        window_id,
        tab_id,
        terminal_id,
    })
}

fn parse_captured_adjacency(raw: &str) -> CapturedAdjacency {
    let mut adjacency = CapturedAdjacency::default();
    for entry in raw.split('|').filter(|entry| !entry.is_empty()) {
        if let Some((dir, target)) = entry.split_once(':') {
            match dir {
                "left" => adjacency.left = Some(target.to_string()),
                "right" => adjacency.right = Some(target.to_string()),
                "up" => adjacency.up = Some(target.to_string()),
                "down" => adjacency.down = Some(target.to_string()),
                _ => {}
            }
        }
    }
    adjacency
}

fn direction_for_layout_slot(layout_slot: &str) -> &str {
    if layout_slot.contains("left") {
        "left"
    } else if layout_slot.contains("up") || layout_slot.contains("top") {
        "up"
    } else if layout_slot.contains("down") || layout_slot.contains("bottom") {
        "down"
    } else {
        "right"
    }
}

fn restore_status_for_terminal(
    terminal: &TerminalSnapshot,
    run_commands: bool,
    run_launch_intent: impl FnOnce() -> Result<()>,
) -> (RestoreStatus, Option<String>) {
    if run_commands {
        if let Some(intent) = terminal.launch_intent.as_deref() {
            if run_launch_intent().is_ok() {
                (
                    RestoreStatus::Restored,
                    Some(format!("restored and launched: {intent}")),
                )
            } else {
                (
                    RestoreStatus::NeedsRerun,
                    Some(format!("shell restored; launch failed: {intent}")),
                )
            }
        } else {
            (
                RestoreStatus::NeedsRerun,
                Some("shell restored; no launch intent saved".into()),
            )
        }
    } else {
        (
            RestoreStatus::NeedsRerun,
            terminal
                .launch_intent
                .as_ref()
                .map(|intent| format!("shell restored; run now available: {intent}"))
                .or_else(|| Some("shell restored; no launch intent saved".into())),
        )
    }
}

fn build_tab_restore_plan(tab: &TabSnapshot) -> Vec<RestorePlanStep> {
    if tab.panes.is_empty() {
        return Vec::new();
    }

    if let Some(archetype_plan) = build_archetype_restore_plan(tab) {
        return finalize_tab_plan(tab, archetype_plan);
    }

    let root = choose_root_pane(tab);

    let mut plan = vec![RestorePlanStep {
        terminal_id: root.terminal_id,
        kind: RestorePlanKind::TabRoot,
    }];
    let mut planned = HashSet::from([root.terminal_id]);
    let mut remaining: Vec<_> = tab
        .panes
        .iter()
        .filter(|pane| pane.terminal_id != root.terminal_id)
        .collect();

    while !remaining.is_empty() {
        let mut progress = false;
        remaining.sort_by_key(|pane| topology_priority(pane));
        let mut next_remaining = Vec::new();

        for pane in remaining {
            if let Some((anchor_terminal_id, direction)) = plan_from_topology(pane, &planned) {
                plan.push(RestorePlanStep {
                    terminal_id: pane.terminal_id,
                    kind: RestorePlanKind::SplitFrom {
                        anchor_terminal_id,
                        direction,
                    },
                });
                planned.insert(pane.terminal_id);
                progress = true;
            } else {
                next_remaining.push(pane);
            }
        }

        if !progress {
            for pane in next_remaining {
                plan.push(RestorePlanStep {
                    terminal_id: pane.terminal_id,
                    kind: RestorePlanKind::FallbackLinear,
                });
                planned.insert(pane.terminal_id);
            }
            break;
        }

        remaining = next_remaining;
    }

    finalize_tab_plan(tab, plan)
}

fn plan_from_topology(
    pane: &PaneSnapshot,
    planned: &HashSet<Uuid>,
) -> Option<(Uuid, &'static str)> {
    let mut candidates = Vec::new();

    if let Some(left) = pane.topology.left.filter(|id| planned.contains(id)) {
        candidates.push((topology_anchor_score(pane, "right"), left, "right"));
    }
    if let Some(up) = pane.topology.up.filter(|id| planned.contains(id)) {
        candidates.push((topology_anchor_score(pane, "down"), up, "down"));
    }
    if let Some(right) = pane.topology.right.filter(|id| planned.contains(id)) {
        candidates.push((topology_anchor_score(pane, "left"), right, "left"));
    }
    if let Some(down) = pane.topology.down.filter(|id| planned.contains(id)) {
        candidates.push((topology_anchor_score(pane, "up"), down, "up"));
    }

    candidates.sort_by_key(|(score, _, direction)| (*score, direction_rank(direction)));
    candidates
        .first()
        .map(|(_, anchor, direction)| (*anchor, *direction))
}

fn choose_root_pane(tab: &TabSnapshot) -> &PaneSnapshot {
    tab.panes
        .iter()
        .max_by_key(|pane| root_priority(pane))
        .unwrap_or(&tab.panes[0])
}

fn build_archetype_restore_plan(tab: &TabSnapshot) -> Option<Vec<RestorePlanStep>> {
    match tab.panes.len() {
        4 => infer_dominant_left_nested_right_plan(tab),
        5 => infer_dominant_left_stacked_right_bottom_split_plan(tab),
        _ => None,
    }
}

fn infer_dominant_left_nested_right_plan(tab: &TabSnapshot) -> Option<Vec<RestorePlanStep>> {
    let root = choose_root_pane(tab);

    let top_right = tab.panes.iter().find(|pane| {
        pane.terminal_id != root.terminal_id
            && pane.topology.left == Some(root.terminal_id)
            && pane.topology.up.is_none()
            && pane.topology.down.is_some()
    })?;

    let mid_right = tab.panes.iter().find(|pane| {
        pane.terminal_id != root.terminal_id
            && pane.terminal_id != top_right.terminal_id
            && pane.topology.up == Some(top_right.terminal_id)
            && pane.topology.left == Some(root.terminal_id)
            && pane.topology.right.is_some()
    })?;

    let bottom_right = tab.panes.iter().find(|pane| {
        pane.terminal_id != root.terminal_id
            && pane.terminal_id != top_right.terminal_id
            && pane.terminal_id != mid_right.terminal_id
            && pane.topology.left == Some(mid_right.terminal_id)
            && pane.topology.up == Some(top_right.terminal_id)
    })?;

    Some(vec![
        RestorePlanStep {
            terminal_id: root.terminal_id,
            kind: RestorePlanKind::TabRoot,
        },
        RestorePlanStep {
            terminal_id: top_right.terminal_id,
            kind: RestorePlanKind::SplitFrom {
                anchor_terminal_id: root.terminal_id,
                direction: "right",
            },
        },
        RestorePlanStep {
            terminal_id: mid_right.terminal_id,
            kind: RestorePlanKind::SplitFrom {
                anchor_terminal_id: top_right.terminal_id,
                direction: "down",
            },
        },
        RestorePlanStep {
            terminal_id: bottom_right.terminal_id,
            kind: RestorePlanKind::SplitFrom {
                anchor_terminal_id: mid_right.terminal_id,
                direction: "right",
            },
        },
    ])
}

fn infer_dominant_left_stacked_right_bottom_split_plan(
    tab: &TabSnapshot,
) -> Option<Vec<RestorePlanStep>> {
    let root = choose_root_pane(tab);

    let top_right = tab.panes.iter().find(|pane| {
        pane.terminal_id != root.terminal_id
            && pane.topology.left == Some(root.terminal_id)
            && pane.topology.up.is_none()
            && pane.topology.down.is_some()
    })?;

    let mid_right = tab.panes.iter().find(|pane| {
        pane.terminal_id != root.terminal_id
            && pane.terminal_id != top_right.terminal_id
            && pane.topology.left == Some(root.terminal_id)
            && pane.topology.up == Some(top_right.terminal_id)
            && pane.topology.down.is_some()
    })?;

    let bottom_left = tab.panes.iter().find(|pane| {
        pane.terminal_id != root.terminal_id
            && pane.terminal_id != top_right.terminal_id
            && pane.terminal_id != mid_right.terminal_id
            && pane.topology.left == Some(root.terminal_id)
            && pane.topology.up == Some(mid_right.terminal_id)
            && pane.topology.right.is_some()
    })?;

    let bottom_right = tab.panes.iter().find(|pane| {
        pane.terminal_id != root.terminal_id
            && pane.terminal_id != top_right.terminal_id
            && pane.terminal_id != mid_right.terminal_id
            && pane.terminal_id != bottom_left.terminal_id
            && pane.topology.left == Some(bottom_left.terminal_id)
            && pane.topology.up == Some(mid_right.terminal_id)
    })?;

    Some(vec![
        RestorePlanStep {
            terminal_id: root.terminal_id,
            kind: RestorePlanKind::TabRoot,
        },
        RestorePlanStep {
            terminal_id: top_right.terminal_id,
            kind: RestorePlanKind::SplitFrom {
                anchor_terminal_id: root.terminal_id,
                direction: "right",
            },
        },
        RestorePlanStep {
            terminal_id: mid_right.terminal_id,
            kind: RestorePlanKind::SplitFrom {
                anchor_terminal_id: top_right.terminal_id,
                direction: "down",
            },
        },
        RestorePlanStep {
            terminal_id: bottom_left.terminal_id,
            kind: RestorePlanKind::SplitFrom {
                anchor_terminal_id: mid_right.terminal_id,
                direction: "down",
            },
        },
        RestorePlanStep {
            terminal_id: bottom_right.terminal_id,
            kind: RestorePlanKind::SplitFrom {
                anchor_terminal_id: bottom_left.terminal_id,
                direction: "right",
            },
        },
    ])
}

fn finalize_tab_plan(tab: &TabSnapshot, mut plan: Vec<RestorePlanStep>) -> Vec<RestorePlanStep> {
    if let Some(first) = plan.first_mut() {
        if matches!(first.kind, RestorePlanKind::TabRoot) && tab.tab_index == 0 {
            first.kind = RestorePlanKind::WindowRoot;
        }
    }
    plan
}

fn root_priority(pane: &PaneSnapshot) -> (i32, i32, i32, i32) {
    let no_left_or_up = if pane.topology.left.is_none() && pane.topology.up.is_none() {
        1
    } else {
        0
    };
    let outward_edges = pane.topology.right.is_some() as i32 + pane.topology.down.is_some() as i32;
    let inward_edges = pane.topology.left.is_some() as i32 + pane.topology.up.is_some() as i32;
    (
        no_left_or_up,
        outward_edges,
        -inward_edges,
        -(pane.pane_index as i32),
    )
}

fn topology_priority(pane: &PaneSnapshot) -> (i32, i32, i32, i32, usize) {
    (
        pane.topology.left.is_some() as i32 + pane.topology.up.is_some() as i32,
        pane.topology.right.is_none() as i32 + pane.topology.down.is_none() as i32,
        pane.topology.left.is_some() as i32,
        pane.topology.up.is_some() as i32,
        pane.pane_index,
    )
}

fn topology_anchor_score(pane: &PaneSnapshot, direction: &str) -> i32 {
    match direction {
        "right" => 0 + pane.topology.up.is_some() as i32 + pane.topology.down.is_some() as i32 * 2,
        "down" => {
            0 + pane.topology.left.is_some() as i32 + pane.topology.right.is_some() as i32 * 2
        }
        "left" => 10 + pane.topology.up.is_some() as i32 + pane.topology.down.is_some() as i32 * 2,
        "up" => 10 + pane.topology.left.is_some() as i32 + pane.topology.right.is_some() as i32 * 2,
        _ => 100,
    }
}

fn direction_rank(direction: &str) -> i32 {
    match direction {
        "right" => 0,
        "down" => 1,
        "left" => 2,
        "up" => 3,
        _ => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_capture_json() {
        let parsed = parse_capture_output(
            r#"{"windows":[{"ghostty_id":"win-1","window_index":0,"tabs":[{"ghostty_id":"tab-1","tab_index":0,"title":"Main","terminals":[{"ghostty_id":"term-1","pane_index":0,"title":"server","working_directory":"/tmp/app"}]}]}]}"#,
        )
        .unwrap();

        assert_eq!(parsed.windows.len(), 1);
        assert_eq!(parsed.windows[0].ghostty_id, "win-1");
        assert_eq!(
            parsed.windows[0].tabs[0].terminals[0]
                .working_directory
                .as_deref(),
            Some("/tmp/app")
        );
    }

    #[test]
    fn parses_ghostty_ids() {
        let refs = parse_ghostty_refs("tab-group-123|tab-456|ABC-DEF").unwrap();
        assert_eq!(refs.window_id, "tab-group-123");
        assert_eq!(refs.tab_id, "tab-456");
        assert_eq!(refs.terminal_id, "ABC-DEF");
    }

    #[test]
    fn parses_directional_adjacency() {
        let parsed = parse_captured_adjacency("left:term-a|down:term-b|");
        assert_eq!(parsed.left.as_deref(), Some("term-a"));
        assert_eq!(parsed.down.as_deref(), Some("term-b"));
        assert_eq!(parsed.right, None);
    }

    #[test]
    fn restore_status_is_restored_only_when_launch_runs() {
        let terminal = TerminalSnapshot::new(
            Some("Server".into()),
            None,
            Some("/tmp/app".into()),
            Some("server".into()),
            Some("bin/dev".into()),
        );

        let restored = restore_status_for_terminal(&terminal, true, || Ok(()));
        let rerun = restore_status_for_terminal(&terminal, false, || Ok(()));
        let failed =
            restore_status_for_terminal(&terminal, true, || Err(Error::Adapter("x".into())));

        assert_eq!(restored.0, RestoreStatus::Restored);
        assert_eq!(rerun.0, RestoreStatus::NeedsRerun);
        assert_eq!(failed.0, RestoreStatus::NeedsRerun);
        assert_eq!(
            rerun.1.as_deref(),
            Some("shell restored; run now available: bin/dev")
        );
    }

    #[test]
    fn builds_restore_plan_from_topology_hints() {
        let root = Uuid::new_v4();
        let right = Uuid::new_v4();
        let lower = Uuid::new_v4();
        let tab = TabSnapshot {
            tab_index: 0,
            title: Some("Main".into()),
            panes: vec![
                PaneSnapshot {
                    pane_index: 0,
                    terminal_id: root,
                    layout_slot: "pane-1".into(),
                    topology: PaneTopologyHint::default(),
                },
                PaneSnapshot {
                    pane_index: 1,
                    terminal_id: right,
                    layout_slot: "pane-2".into(),
                    topology: PaneTopologyHint {
                        left: Some(root),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 2,
                    terminal_id: lower,
                    layout_slot: "pane-3".into(),
                    topology: PaneTopologyHint {
                        up: Some(right),
                        ..PaneTopologyHint::default()
                    },
                },
            ],
        };

        let plan = build_tab_restore_plan(&tab);
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].terminal_id, root);
        assert_eq!(plan[0].kind, RestorePlanKind::WindowRoot);
        assert_eq!(
            plan[1].kind,
            RestorePlanKind::SplitFrom {
                anchor_terminal_id: root,
                direction: "right",
            }
        );
        assert_eq!(
            plan[2].kind,
            RestorePlanKind::SplitFrom {
                anchor_terminal_id: right,
                direction: "down",
            }
        );
    }

    #[test]
    fn chooses_root_from_topology_not_only_pane_index() {
        let root = Uuid::new_v4();
        let child = Uuid::new_v4();
        let misleading_first = Uuid::new_v4();

        let tab = TabSnapshot {
            tab_index: 0,
            title: Some("Main".into()),
            panes: vec![
                PaneSnapshot {
                    pane_index: 0,
                    terminal_id: misleading_first,
                    layout_slot: "pane-1".into(),
                    topology: PaneTopologyHint {
                        left: Some(root),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 1,
                    terminal_id: root,
                    layout_slot: "pane-2".into(),
                    topology: PaneTopologyHint {
                        right: Some(misleading_first),
                        down: Some(child),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 2,
                    terminal_id: child,
                    layout_slot: "pane-3".into(),
                    topology: PaneTopologyHint {
                        up: Some(root),
                        ..PaneTopologyHint::default()
                    },
                },
            ],
        };

        let plan = build_tab_restore_plan(&tab);
        assert_eq!(plan[0].terminal_id, root);
        assert_eq!(plan[0].kind, RestorePlanKind::WindowRoot);
    }

    #[test]
    fn infers_dominant_left_nested_right_archetype() {
        let root = Uuid::new_v4();
        let top_right = Uuid::new_v4();
        let mid_right = Uuid::new_v4();
        let bottom_right = Uuid::new_v4();

        let tab = TabSnapshot {
            tab_index: 0,
            title: Some("Main".into()),
            panes: vec![
                PaneSnapshot {
                    pane_index: 0,
                    terminal_id: root,
                    layout_slot: "pane-1".into(),
                    topology: PaneTopologyHint {
                        right: Some(top_right),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 1,
                    terminal_id: top_right,
                    layout_slot: "pane-2".into(),
                    topology: PaneTopologyHint {
                        left: Some(root),
                        down: Some(mid_right),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 2,
                    terminal_id: mid_right,
                    layout_slot: "pane-3".into(),
                    topology: PaneTopologyHint {
                        left: Some(root),
                        right: Some(bottom_right),
                        up: Some(top_right),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 3,
                    terminal_id: bottom_right,
                    layout_slot: "pane-4".into(),
                    topology: PaneTopologyHint {
                        left: Some(mid_right),
                        up: Some(top_right),
                        ..PaneTopologyHint::default()
                    },
                },
            ],
        };

        let plan = build_tab_restore_plan(&tab);
        assert_eq!(plan.len(), 4);
        assert_eq!(plan[0].terminal_id, root);
        assert_eq!(
            plan[1].kind,
            RestorePlanKind::SplitFrom {
                anchor_terminal_id: root,
                direction: "right",
            }
        );
        assert_eq!(
            plan[2].kind,
            RestorePlanKind::SplitFrom {
                anchor_terminal_id: top_right,
                direction: "down",
            }
        );
        assert_eq!(
            plan[3].kind,
            RestorePlanKind::SplitFrom {
                anchor_terminal_id: mid_right,
                direction: "right",
            }
        );
    }

    #[test]
    fn infers_dominant_left_stacked_right_bottom_split_archetype() {
        let root = Uuid::new_v4();
        let top_right = Uuid::new_v4();
        let mid_right = Uuid::new_v4();
        let bottom_left = Uuid::new_v4();
        let bottom_right = Uuid::new_v4();
        let tab = TabSnapshot {
            tab_index: 0,
            title: Some("Main".into()),
            panes: vec![
                PaneSnapshot {
                    pane_index: 0,
                    terminal_id: root,
                    layout_slot: "pane-1".into(),
                    topology: PaneTopologyHint {
                        right: Some(top_right),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 1,
                    terminal_id: top_right,
                    layout_slot: "pane-2".into(),
                    topology: PaneTopologyHint {
                        left: Some(root),
                        down: Some(mid_right),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 2,
                    terminal_id: mid_right,
                    layout_slot: "pane-3".into(),
                    topology: PaneTopologyHint {
                        left: Some(root),
                        up: Some(top_right),
                        down: Some(bottom_left),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 3,
                    terminal_id: bottom_left,
                    layout_slot: "pane-4".into(),
                    topology: PaneTopologyHint {
                        left: Some(root),
                        up: Some(mid_right),
                        right: Some(bottom_right),
                        ..PaneTopologyHint::default()
                    },
                },
                PaneSnapshot {
                    pane_index: 4,
                    terminal_id: bottom_right,
                    layout_slot: "pane-5".into(),
                    topology: PaneTopologyHint {
                        left: Some(bottom_left),
                        up: Some(mid_right),
                        ..PaneTopologyHint::default()
                    },
                },
            ],
        };

        let plan = build_tab_restore_plan(&tab);
        assert_eq!(plan.len(), 5);
        assert!(matches!(plan[0].kind, RestorePlanKind::WindowRoot));
        assert_eq!(plan[0].terminal_id, root);
        assert_eq!(
            plan[1].kind,
            RestorePlanKind::SplitFrom {
                anchor_terminal_id: root,
                direction: "right",
            }
        );
        assert_eq!(plan[1].terminal_id, top_right);
        assert_eq!(
            plan[2].kind,
            RestorePlanKind::SplitFrom {
                anchor_terminal_id: top_right,
                direction: "down",
            }
        );
        assert_eq!(plan[2].terminal_id, mid_right);
        assert_eq!(
            plan[3].kind,
            RestorePlanKind::SplitFrom {
                anchor_terminal_id: mid_right,
                direction: "down",
            }
        );
        assert_eq!(plan[3].terminal_id, bottom_left);
        assert_eq!(
            plan[4].kind,
            RestorePlanKind::SplitFrom {
                anchor_terminal_id: bottom_left,
                direction: "right",
            }
        );
        assert_eq!(plan[4].terminal_id, bottom_right);
    }
}
