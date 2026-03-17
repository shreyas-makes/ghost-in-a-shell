use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::Stylize,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::ghostty_adapter::AppleScriptGhosttyAdapter;
use crate::store::SnapshotStore;
use crate::{Error, Result};

pub fn run_tui(store: SnapshotStore, adapter: AppleScriptGhosttyAdapter) -> Result<()> {
    enable_raw_mode().map_err(|error| Error::Tui(error.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|error| Error::Tui(error.to_string()))?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|error| Error::Tui(error.to_string()))?;

    let mut app = App::new(store, adapter);
    let result = app.run(&mut terminal);

    disable_raw_mode().map_err(|error| Error::Tui(error.to_string()))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|error| Error::Tui(error.to_string()))?;
    terminal
        .show_cursor()
        .map_err(|error| Error::Tui(error.to_string()))?;
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    Normal,
    Filter,
    SaveName,
    Rename,
    ConfirmDelete,
}

struct App {
    store: SnapshotStore,
    adapter: AppleScriptGhosttyAdapter,
    snapshots: Vec<crate::model::WorkspaceSnapshot>,
    selected: usize,
    filter: String,
    message: String,
    input_mode: InputMode,
    input: String,
}

impl App {
    fn new(store: SnapshotStore, adapter: AppleScriptGhosttyAdapter) -> Self {
        let snapshots = store.list().unwrap_or_default();
        let message = if snapshots.is_empty() {
            "Save the Ghostty setup you want to bring back later. Press s to save the current setup.".into()
        } else {
            "Enter restores, / filters, s saves, R renames, d deletes, q quits.".into()
        };
        Self {
            store,
            adapter,
            snapshots,
            selected: 0,
            filter: String::new(),
            message,
            input_mode: InputMode::Normal,
            input: String::new(),
        }
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        loop {
            terminal
                .draw(|frame| self.draw(frame))
                .map_err(|error| Error::Tui(error.to_string()))?;

            if event::poll(Duration::from_millis(200))
                .map_err(|error| Error::Tui(error.to_string()))?
            {
                if let Event::Key(key) =
                    event::read().map_err(|error| Error::Tui(error.to_string()))?
                {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    if self.handle_key(key.code)? {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode) -> Result<bool> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(code),
            InputMode::Filter | InputMode::SaveName | InputMode::Rename => {
                self.handle_text_input(code)
            }
            InputMode::ConfirmDelete => self.handle_delete_confirm(code),
        }
    }

    fn handle_normal_key(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.filtered_snapshots().len() {
                    self.selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Filter;
                self.input = self.filter.clone();
            }
            KeyCode::Char('s') => {
                self.input_mode = InputMode::SaveName;
                self.input.clear();
                self.message = "Name the snapshot to save.".into();
            }
            KeyCode::Char('R') => {
                if let Some(snapshot) = self.selected_snapshot() {
                    self.input_mode = InputMode::Rename;
                    self.input = snapshot.name.clone();
                    self.message = format!("Rename {}.", snapshot.name);
                }
            }
            KeyCode::Char('d') => {
                if let Some(snapshot) = self.selected_snapshot() {
                    self.input_mode = InputMode::ConfirmDelete;
                    self.message = format!("Delete {}? Press y to confirm.", snapshot.name);
                }
            }
            KeyCode::Enter | KeyCode::Char('r') => {
                if let Some(snapshot) = self.selected_snapshot() {
                    let restored = self.adapter.restore_workspace(&snapshot, false)?;
                    self.message =
                        crate::recovery::RecoverySummary::from_snapshot(&restored).render_text();
                }
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_text_input(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input.clear();
            }
            KeyCode::Enter => match self.input_mode {
                InputMode::Filter => {
                    self.filter = self.input.clone();
                    self.selected = 0;
                    self.input_mode = InputMode::Normal;
                }
                InputMode::SaveName => {
                    let snapshot = self.adapter.capture_workspace(&self.input)?;
                    self.store.save(&snapshot, false)?;
                    self.snapshots = self.store.list()?;
                    self.selected = 0;
                    self.message = format!("Saved {}", snapshot.name);
                    self.input.clear();
                    self.input_mode = InputMode::Normal;
                }
                InputMode::Rename => {
                    if let Some(snapshot) = self.selected_snapshot() {
                        let renamed = self.store.rename(&snapshot.name, &self.input)?;
                        self.snapshots = self.store.list()?;
                        self.message = format!("Renamed snapshot to {}", renamed.name);
                    }
                    self.input.clear();
                    self.input_mode = InputMode::Normal;
                }
                InputMode::Normal | InputMode::ConfirmDelete => {}
            },
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(ch) => self.input.push(ch),
            _ => {}
        }
        Ok(false)
    }

    fn handle_delete_confirm(&mut self, code: KeyCode) -> Result<bool> {
        match code {
            KeyCode::Char('y') => {
                if let Some(snapshot) = self.selected_snapshot() {
                    self.store.delete(&snapshot.name)?;
                    self.snapshots = self.store.list()?;
                    self.selected = self.selected.min(self.snapshots.len().saturating_sub(1));
                    self.message = format!("Deleted {}", snapshot.name);
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                self.input_mode = InputMode::Normal;
                self.message = "Delete cancelled.".into();
            }
            _ => {}
        }
        Ok(false)
    }

    fn filtered_snapshots(&self) -> Vec<&crate::model::WorkspaceSnapshot> {
        if self.filter.trim().is_empty() {
            return self.snapshots.iter().collect();
        }

        let query = self.filter.to_lowercase();
        self.snapshots
            .iter()
            .filter(|snapshot| {
                snapshot.name.to_lowercase().contains(&query)
                    || snapshot.slug.contains(&query)
                    || snapshot
                        .terminals
                        .iter()
                        .any(|terminal| terminal.display_label().to_lowercase().contains(&query))
            })
            .collect()
    }

    fn selected_snapshot(&self) -> Option<crate::model::WorkspaceSnapshot> {
        self.filtered_snapshots()
            .get(self.selected)
            .map(|snapshot| (*snapshot).clone())
    }

    fn draw(&self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(4),
            ])
            .split(area);

        let header = Paragraph::new(format!(
            "ghost-in-a-shell  filter: {}",
            if self.filter.is_empty() {
                "<none>"
            } else {
                &self.filter
            }
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Workspace Launcher"),
        );
        frame.render_widget(header, chunks[0]);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(chunks[1]);

        let filtered = self.filtered_snapshots();
        let items: Vec<ListItem<'_>> = if filtered.is_empty() {
            vec![ListItem::new("No saved snapshots")]
        } else {
            filtered
                .iter()
                .map(|snapshot| {
                    ListItem::new(Line::from(vec![
                        snapshot.name.clone().into(),
                        format!("  {}", snapshot.summary_line()).fg(Color::DarkGray),
                    ]))
                })
                .collect()
        };
        let mut state = ListState::default();
        state.select(Some(self.selected.min(items.len().saturating_sub(1))));
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Snapshots"))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )
            .highlight_symbol("› ");
        frame.render_stateful_widget(list, body[0], &mut state);

        let preview_text = self
            .selected_snapshot()
            .map(|snapshot| Text::from(snapshot.preview_lines().join("\n")))
            .unwrap_or_else(|| Text::from("Save the Ghostty setup you want to bring back later."));
        let preview = Paragraph::new(preview_text)
            .block(Block::default().borders(Borders::ALL).title("Preview"))
            .wrap(Wrap { trim: false });
        frame.render_widget(preview, body[1]);

        let footer = Paragraph::new(self.message.clone())
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .wrap(Wrap { trim: false });
        frame.render_widget(footer, chunks[2]);

        if self.input_mode != InputMode::Normal {
            let popup = centered_rect(70, 20, area);
            frame.render_widget(Clear, popup);
            let title = match self.input_mode {
                InputMode::Filter => "Filter",
                InputMode::SaveName => "Save Current Setup",
                InputMode::Rename => "Rename Snapshot",
                InputMode::ConfirmDelete => "Delete Snapshot",
                InputMode::Normal => "",
            };
            let content = match self.input_mode {
                InputMode::ConfirmDelete => self.message.clone(),
                _ => self.input.clone(),
            };
            let block = Paragraph::new(content)
                .block(Block::default().borders(Borders::ALL).title(title))
                .wrap(Wrap { trim: false });
            frame.render_widget(block, popup);
        }
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
