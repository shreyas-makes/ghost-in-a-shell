use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotSource {
    ManualSave,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RestoreStatus {
    Restored,
    NeedsRerun,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub workspace_id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: SnapshotSource,
    pub windows: Vec<WindowSnapshot>,
    pub terminals: Vec<TerminalSnapshot>,
}

impl WorkspaceSnapshot {
    pub fn new(
        name: String,
        slug: String,
        windows: Vec<WindowSnapshot>,
        terminals: Vec<TerminalSnapshot>,
    ) -> Self {
        let now = Utc::now();
        Self {
            workspace_id: Uuid::new_v4(),
            name,
            slug,
            created_at: now,
            updated_at: now,
            source: SnapshotSource::ManualSave,
            windows,
            terminals,
        }
    }

    pub fn terminal_count(&self) -> usize {
        self.terminals.len()
    }

    pub fn summary_line(&self) -> String {
        let window_count = self.windows.len();
        let tab_count = self
            .windows
            .iter()
            .map(|window| window.tabs.len())
            .sum::<usize>();
        format!(
            "{window_count}w {tab_count}t {} terminals",
            self.terminal_count()
        )
    }

    pub fn preview_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!(
                "Updated {}",
                self.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
            ),
            self.summary_line(),
            String::new(),
        ];

        for window in &self.windows {
            lines.push(format!("Window {}", window.window_index + 1));
            for tab in &window.tabs {
                let title = tab.title.as_deref().unwrap_or("untitled");
                lines.push(format!("  Tab {}: {}", tab.tab_index + 1, title));
                for pane in &tab.panes {
                    if let Some(terminal) = self.terminal_by_id(pane.terminal_id) {
                        let cwd = terminal
                            .working_directory
                            .clone()
                            .unwrap_or_else(|| "<unknown cwd>".to_string());
                        let intent = terminal
                            .launch_intent
                            .clone()
                            .unwrap_or_else(|| "shell only".to_string());
                        lines.push(format!(
                            "    {} [{}] {}",
                            terminal.display_label(),
                            cwd,
                            intent
                        ));
                    }
                }
            }
            lines.push(String::new());
        }

        lines
    }

    pub fn terminal_by_id(&self, terminal_id: Uuid) -> Option<&TerminalSnapshot> {
        self.terminals
            .iter()
            .find(|terminal| terminal.terminal_id == terminal_id)
    }

    pub fn terminal_by_id_mut(&mut self, terminal_id: Uuid) -> Option<&mut TerminalSnapshot> {
        self.terminals
            .iter_mut()
            .find(|terminal| terminal.terminal_id == terminal_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowSnapshot {
    pub window_index: usize,
    pub tabs: Vec<TabSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TabSnapshot {
    pub tab_index: usize,
    pub title: Option<String>,
    pub panes: Vec<PaneSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaneSnapshot {
    pub pane_index: usize,
    pub terminal_id: Uuid,
    pub layout_slot: String,
    #[serde(default)]
    pub topology: PaneTopologyHint,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaneTopologyHint {
    pub left: Option<Uuid>,
    pub right: Option<Uuid>,
    pub up: Option<Uuid>,
    pub down: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalSnapshot {
    pub terminal_id: Uuid,
    pub label: Option<String>,
    pub role: Option<String>,
    pub working_directory: Option<String>,
    pub surface_title: Option<String>,
    pub launch_intent: Option<String>,
    pub restore_status: Option<RestoreStatus>,
    pub restore_note: Option<String>,
    pub last_seen_at: DateTime<Utc>,
}

impl TerminalSnapshot {
    pub fn new(
        label: Option<String>,
        role: Option<String>,
        working_directory: Option<String>,
        surface_title: Option<String>,
        launch_intent: Option<String>,
    ) -> Self {
        Self {
            terminal_id: Uuid::new_v4(),
            label,
            role,
            working_directory,
            surface_title,
            launch_intent,
            restore_status: None,
            restore_note: None,
            last_seen_at: Utc::now(),
        }
    }

    pub fn display_label(&self) -> String {
        self.label
            .clone()
            .or_else(|| self.surface_title.clone())
            .unwrap_or_else(|| format!("terminal {}", self.terminal_id))
    }
}
