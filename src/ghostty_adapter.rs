use std::process::Command;

use serde::Deserialize;

use crate::model::{
    PaneSnapshot, RestoreStatus, TabSnapshot, TerminalSnapshot, WindowSnapshot, WorkspaceSnapshot,
};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GhosttyCapture {
    pub windows: Vec<CapturedWindow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CapturedWindow {
    pub window_index: usize,
    pub tabs: Vec<CapturedTab>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CapturedTab {
    pub tab_index: usize,
    pub title: Option<String>,
    pub terminals: Vec<CapturedTerminal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CapturedTerminal {
    pub pane_index: usize,
    pub title: Option<String>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppleScriptGhosttyAdapter;

#[derive(Debug, Clone, Copy)]
struct GhosttyRefs {
    window_id: i64,
    _tab_id: i64,
    terminal_id: i64,
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
                let mut panes = Vec::new();
                for captured_terminal in captured_tab.terminals {
                    let terminal = TerminalSnapshot::new(
                        captured_terminal.title.clone(),
                        None,
                        captured_terminal.working_directory.clone(),
                        captured_terminal.title,
                        None,
                    );
                    panes.push(PaneSnapshot {
                        pane_index: captured_terminal.pane_index,
                        terminal_id: terminal.terminal_id,
                        layout_slot: format!("pane-{}", captured_terminal.pane_index + 1),
                    });
                    terminals.push(terminal);
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
                let mut tab_anchor_terminal: Option<GhosttyRefs> = None;
                for pane in &tab.panes {
                    let terminal = snapshot
                        .terminal_by_id(pane.terminal_id)
                        .ok_or_else(|| Error::Adapter("pane referenced unknown terminal".into()))?;

                    let creation_result = if window_ref.is_none() && pane.pane_index == 0 {
                        self.new_window(terminal.working_directory.as_deref())
                    } else if pane.pane_index == 0 {
                        self.new_tab(
                            window_ref
                                .ok_or_else(|| Error::Adapter("window ref missing".into()))?
                                .window_id,
                            terminal.working_directory.as_deref(),
                        )
                    } else {
                        self.split_terminal(
                            tab_anchor_terminal
                                .ok_or_else(|| {
                                    Error::Adapter("tab anchor terminal missing".into())
                                })?
                                .terminal_id,
                            direction_for_layout_slot(&pane.layout_slot),
                            terminal.working_directory.as_deref(),
                        )
                    };

                    let status = if let Ok(created_refs) = creation_result {
                        if window_ref.is_none() {
                            window_ref = Some(created_refs);
                        }
                        if tab_anchor_terminal.is_none() || pane.pane_index == 0 {
                            tab_anchor_terminal = Some(created_refs);
                        }
                        if run_commands {
                            if let Some(intent) = &terminal.launch_intent {
                                match self.send_text_to_terminal(created_refs.terminal_id, intent) {
                                    Ok(()) => RestoreStatus::Restored,
                                    Err(_) => RestoreStatus::NeedsRerun,
                                }
                            } else {
                                RestoreStatus::NeedsRerun
                            }
                        } else {
                            RestoreStatus::NeedsRerun
                        }
                    } else {
                        RestoreStatus::NeedsRerun
                    };

                    if let Some(restored_terminal) =
                        restored.terminal_by_id_mut(terminal.terminal_id)
                    {
                        restored_terminal.restore_status = Some(status);
                    }
                }
            }
        }

        Ok(restored)
    }

    pub fn send_text_to_terminal(&self, terminal_id: i64, text: &str) -> Result<()> {
        let escaped = applescript_string(text);
        let script = format!(
            "tell application \"Ghostty\"\n\
             set targetTerm to first terminal whose id is {terminal_id}\n\
             input text {escaped} to targetTerm\n\
             send key \"enter\" to targetTerm\n\
             end tell"
        );
        self.run_script(&script).map(|_| ())
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
        set output to output & "{\"window_index\":" & (window_index - 1) & ",\"tabs\":["
        set tab_count to count of tabs of win_ref
        repeat with tab_index from 1 to tab_count
            set tab_ref to tab tab_index of win_ref
            set tab_name to name of tab_ref
            if tab_name is missing value then
                set tab_json to "null"
            else
                set tab_json to my json_quote(tab_name as text)
            end if

            set output to output & "{\"tab_index\":" & (tab_index - 1) & ",\"title\":" & tab_json & ",\"terminals\":["
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

                set output to output & "{\"pane_index\":" & (pane_index - 1) & ",\"title\":" & term_name_json & ",\"working_directory\":" & term_cwd_json & "}"
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

    fn new_tab(&self, window_id: i64, cwd: Option<&str>) -> Result<GhosttyRefs> {
        let cfg = applescript_config(cwd);
        let script = format!(
            "tell application \"Ghostty\"\n\
             {cfg}\n\
             set winRef to first window whose id is {window_id}\n\
             set tabRef to new tab in winRef with configuration cfg\n\
             set termRef to terminal 1 of tabRef\n\
             return (id of winRef as text) & \"|\" & (id of tabRef as text) & \"|\" & (id of termRef as text)\n\
             end tell"
        );
        self.run_script(&script)
            .and_then(|raw| parse_ghostty_refs(&raw))
    }

    fn split_terminal(
        &self,
        terminal_id: i64,
        direction: &str,
        cwd: Option<&str>,
    ) -> Result<GhosttyRefs> {
        let cfg = applescript_config(cwd);
        let script = format!(
            "tell application \"Ghostty\"\n\
             {cfg}\n\
             set anchorTerm to first terminal whose id is {terminal_id}\n\
             set newTerm to split anchorTerm direction {direction} with configuration cfg\n\
             return (id of window of newTerm as text) & \"|\" & (id of tab of newTerm as text) & \"|\" & (id of newTerm as text)\n\
             end tell"
        );
        self.run_script(&script)
            .and_then(|raw| parse_ghostty_refs(&raw))
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
    format!("\"{}\"", input.replace('\\', "\\\\").replace('"', "\\\""))
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
        .parse::<i64>()
        .map_err(|error| Error::Adapter(format!("invalid Ghostty window id: {error}")))?;
    let tab_id = parts
        .next()
        .ok_or_else(|| Error::Adapter("missing Ghostty tab id".into()))?
        .parse::<i64>()
        .map_err(|error| Error::Adapter(format!("invalid Ghostty tab id: {error}")))?;
    let terminal_id = parts
        .next()
        .ok_or_else(|| Error::Adapter("missing Ghostty terminal id".into()))?
        .parse::<i64>()
        .map_err(|error| Error::Adapter(format!("invalid Ghostty terminal id: {error}")))?;
    Ok(GhosttyRefs {
        window_id,
        _tab_id: tab_id,
        terminal_id,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_capture_json() {
        let parsed = parse_capture_output(
            r#"{"windows":[{"window_index":0,"tabs":[{"tab_index":0,"title":"Main","terminals":[{"pane_index":0,"title":"server","working_directory":"/tmp/app"}]}]}]}"#,
        )
        .unwrap();

        assert_eq!(parsed.windows.len(), 1);
        assert_eq!(
            parsed.windows[0].tabs[0].terminals[0]
                .working_directory
                .as_deref(),
            Some("/tmp/app")
        );
    }

    #[test]
    fn parses_ghostty_ids() {
        let refs = parse_ghostty_refs("12|13|14").unwrap();
        assert_eq!(refs.window_id, 12);
        assert_eq!(refs._tab_id, 13);
        assert_eq!(refs.terminal_id, 14);
    }
}
