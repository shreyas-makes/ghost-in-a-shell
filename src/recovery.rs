use uuid::Uuid;

use crate::model::{RestoreStatus, WorkspaceSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreOutcomeKind {
    Restored,
    NeedsRerun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalRestoreOutcome {
    pub terminal_id: Uuid,
    pub label: String,
    pub kind: RestoreOutcomeKind,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySummary {
    pub workspace_name: String,
    pub outcomes: Vec<TerminalRestoreOutcome>,
}

impl RecoverySummary {
    pub fn from_snapshot(snapshot: &WorkspaceSnapshot) -> Self {
        let outcomes = snapshot
            .terminals
            .iter()
            .map(|terminal| {
                let label = terminal.display_label();
                let kind = match terminal
                    .restore_status
                    .clone()
                    .unwrap_or(RestoreStatus::NeedsRerun)
                {
                    RestoreStatus::Restored => RestoreOutcomeKind::Restored,
                    RestoreStatus::NeedsRerun => RestoreOutcomeKind::NeedsRerun,
                };
                let detail = match kind {
                    RestoreOutcomeKind::Restored => terminal
                        .restore_note
                        .clone()
                        .unwrap_or_else(|| "restored shell surface".to_string()),
                    RestoreOutcomeKind::NeedsRerun => {
                        terminal.restore_note.clone().unwrap_or_else(|| {
                            terminal
                                .launch_intent
                                .clone()
                                .map(|intent| format!("needs rerun: {intent}"))
                                .unwrap_or_else(|| {
                                    "needs rerun: no launch intent saved".to_string()
                                })
                        })
                    }
                };
                TerminalRestoreOutcome {
                    terminal_id: terminal.terminal_id,
                    label,
                    kind,
                    detail,
                }
            })
            .collect();

        Self {
            workspace_name: snapshot.name.clone(),
            outcomes,
        }
    }

    pub fn render_text(&self) -> String {
        let mut lines = vec![format!("Restore summary for {}", self.workspace_name)];
        for outcome in &self.outcomes {
            let state = match outcome.kind {
                RestoreOutcomeKind::Restored => "restored",
                RestoreOutcomeKind::NeedsRerun => "needs rerun",
            };
            lines.push(format!(
                "- {}: {} ({})",
                outcome.label, state, outcome.detail
            ));
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        PaneSnapshot, PaneTopologyHint, TabSnapshot, TerminalSnapshot, WindowSnapshot,
        WorkspaceSnapshot,
    };

    use super::*;

    #[test]
    fn computes_restore_summary_from_snapshot_statuses() {
        let mut restored = TerminalSnapshot::new(
            Some("Server".into()),
            None,
            Some("/tmp/app".into()),
            Some("bin/dev".into()),
            Some("bin/dev".into()),
        );
        restored.restore_status = Some(RestoreStatus::Restored);

        let mut needs_rerun = TerminalSnapshot::new(
            Some("Tests".into()),
            None,
            Some("/tmp/app".into()),
            None,
            None,
        );
        needs_rerun.restore_status = Some(RestoreStatus::NeedsRerun);

        let snapshot = WorkspaceSnapshot::new(
            "Sample".into(),
            "sample".into(),
            vec![WindowSnapshot {
                window_index: 0,
                tabs: vec![TabSnapshot {
                    tab_index: 0,
                    title: Some("Main".into()),
                    panes: vec![
                        PaneSnapshot {
                            pane_index: 0,
                            terminal_id: restored.terminal_id,
                            layout_slot: "pane-1".into(),
                            topology: PaneTopologyHint::default(),
                        },
                        PaneSnapshot {
                            pane_index: 1,
                            terminal_id: needs_rerun.terminal_id,
                            layout_slot: "pane-2".into(),
                            topology: PaneTopologyHint::default(),
                        },
                    ],
                }],
            }],
            vec![restored, needs_rerun],
        );

        let summary = RecoverySummary::from_snapshot(&snapshot);
        assert_eq!(summary.outcomes.len(), 2);
        assert_eq!(summary.outcomes[0].kind, RestoreOutcomeKind::Restored);
        assert_eq!(summary.outcomes[1].kind, RestoreOutcomeKind::NeedsRerun);
    }
}
