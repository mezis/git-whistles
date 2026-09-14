//! Ratatui loop on `/dev/tty` (never stdout).

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::state::{Action, Key, PickerRow, PickerState};
use super::teardown::{self, StderrStreamingRunner, TeardownPlan};
use crate::worktree::{self, DirtySample};

/// Run the picker. `Some(path)` if the user confirmed a selection.
pub fn run(rows: Vec<PickerRow>, cwd: PathBuf) -> Result<Option<PathBuf>, String> {
    let mut terminal = start_terminal()?;
    let restore = TtyRestore;
    let mut state = PickerState::new(rows, cwd);
    let outcome = event_loop(&mut terminal, &mut state);
    drop(restore);
    outcome
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::fs::File>>,
    state: &mut PickerState,
) -> Result<Option<PathBuf>, String> {
    loop {
        terminal
            .draw(|frame| draw(frame, state))
            .map_err(|err| err.to_string())?;
        let event = event::read().map_err(|err| err.to_string())?;
        let Event::Key(key_event) = event else {
            continue;
        };
        if key_event.kind != KeyEventKind::Press {
            continue;
        }
        match state.handle_key(key_from_event(key_event)) {
            Action::None => {}
            Action::Quit => return Ok(None),
            Action::Select(path) => return Ok(Some(path)),
            Action::OpenDestroyConfirm => {
                let selected_path = state.selected_row().map(|row| row.path.clone());
                if let Some(path) = selected_path {
                    let plan = teardown::detect(&path);
                    match worktree::status_sample(&path, 5) {
                        Ok(dirty) => state.begin_confirm(dirty, plan, None),
                        Err(message) => state.begin_confirm(
                            DirtySample {
                                paths: Vec::new(),
                                total: 0,
                            },
                            plan,
                            Some(message),
                        ),
                    }
                }
            }
            Action::Destroy {
                clone,
                path,
                force,
                locked,
                plan,
            } => {
                suspend_terminal(terminal)?;
                let destroy_result = destroy_worktree(&path, &clone, &plan, force, locked);
                resume_terminal(terminal)?;
                if let Err(message) = destroy_result {
                    eprintln!("{message}");
                    continue;
                }
                match worktree::list_worktrees(&clone) {
                    Ok(trees) => {
                        let listings = vec![(clone.clone(), Ok(trees))];
                        let (rows, _) = super::state::rows_from_listings(listings, &state.cwd);
                        state.replace_clone_rows(&clone, rows);
                    }
                    Err(message) => eprintln!("{message}"),
                }
            }
        }
    }
}

fn destroy_worktree(
    path: &Path,
    clone: &Path,
    plan: &TeardownPlan,
    force: bool,
    locked: bool,
) -> Result<(), String> {
    teardown::run_plan(path, plan, &StderrStreamingRunner)?;
    worktree::remove_worktree(clone, path, force, locked)
}

fn start_terminal() -> Result<Terminal<CrosstermBackend<std::fs::File>>, String> {
    let mut tty = open_tty()?;
    enable_raw_mode().map_err(|err| err.to_string())?;
    execute!(tty, EnterAlternateScreen).map_err(|err| err.to_string())?;
    let backend = CrosstermBackend::new(tty);
    Terminal::new(backend).map_err(|err| err.to_string())
}

fn suspend_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::fs::File>>,
) -> Result<(), String> {
    disable_raw_mode().map_err(|err| err.to_string())?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|err| err.to_string())?;
    let _ = terminal.show_cursor();
    Ok(())
}

fn resume_terminal(terminal: &mut Terminal<CrosstermBackend<std::fs::File>>) -> Result<(), String> {
    enable_raw_mode().map_err(|err| err.to_string())?;
    execute!(terminal.backend_mut(), EnterAlternateScreen).map_err(|err| err.to_string())?;
    terminal.clear().map_err(|err| err.to_string())?;
    Ok(())
}

struct TtyRestore;

impl Drop for TtyRestore {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        if let Ok(mut tty) = open_tty() {
            let _ = execute!(tty, LeaveAlternateScreen);
        }
    }
}

fn open_tty() -> Result<std::fs::File, String> {
    #[cfg(unix)]
    {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .map_err(|err| format!("worktree-list requires a terminal: {err}"))
    }
    #[cfg(not(unix))]
    {
        Err("worktree-list requires /dev/tty (Unix)".to_string())
    }
}

fn key_from_event(event: KeyEvent) -> Key {
    match event.code {
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => Key::CtrlC,
        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => Key::CtrlD,
        KeyCode::Char(filter_char) if filter_char.is_ascii_alphanumeric() => {
            Key::FilterChar(filter_char)
        }
        _ => Key::Other,
    }
}

fn draw(frame: &mut Frame, state: &PickerState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let visible = state.visible();
    let mut lines: Vec<Line> = Vec::new();
    for (index, row) in visible.iter().enumerate() {
        let matched = state.match_indices(index);
        lines.push(highlighted_line(
            row.display_line(),
            matched,
            index == state.selected(),
        ));
    }
    if lines.is_empty() {
        lines.push(Line::from("(no matching worktrees)"));
    }
    let list =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("worktrees"));
    frame.render_widget(list, chunks[0]);

    let filter_text = format!("filter: {}", state.filter());
    frame.render_widget(Paragraph::new(filter_text), chunks[1]);

    let footer = match state.refuse_message() {
        Some(message) => message.to_string(),
        None => "↑↓ select  Enter cd  Ctrl-d destroy  Esc quit".to_string(),
    };
    frame.render_widget(Paragraph::new(footer), chunks[2]);

    if let Some(confirm) = state.confirm() {
        draw_confirm(frame, area, confirm);
    }
}

fn highlighted_line(line: &str, matched: &[usize], selected: bool) -> Line<'static> {
    let matched_set: std::collections::HashSet<usize> = matched.iter().copied().collect();
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut current_matched = false;
    let mut started = false;
    for (char_index, character) in line.chars().enumerate() {
        let is_matched = matched_set.contains(&char_index);
        if !started {
            current_matched = is_matched;
            started = true;
        } else if is_matched != current_matched {
            spans.push(Span::styled(
                std::mem::take(&mut current),
                highlight_style(selected, current_matched),
            ));
            current_matched = is_matched;
        }
        current.push(character);
    }
    if !current.is_empty() || spans.is_empty() {
        spans.push(Span::styled(
            current,
            highlight_style(selected, current_matched),
        ));
    }
    Line::from(spans)
}

fn highlight_style(selected: bool, matched: bool) -> Style {
    let mut style = Style::default();
    if selected {
        style = style.bg(Color::DarkGray);
    }
    if matched {
        style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
    }
    style
}

fn draw_confirm(frame: &mut Frame, area: Rect, confirm: &super::state::ConfirmState) {
    let popup = centered_rect(area, 70, 60);
    frame.render_widget(Clear, popup);
    let basename = confirm
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let mut text = vec![
        format!("Destroy worktree {} ({basename})?", confirm.path.display()),
        String::new(),
    ];
    if let Some(error) = &confirm.status_error {
        text.push(format!("Could not read git status: {error}"));
    } else if confirm.dirty.is_dirty() {
        text.push("This worktree has uncommitted changes:".to_string());
        for path in &confirm.dirty.paths {
            text.push(format!("  {path}"));
        }
        if confirm.dirty.total > confirm.dirty.paths.len() {
            let extra = confirm.dirty.total - confirm.dirty.paths.len();
            text.push(format!("  and {extra} more"));
        }
    } else {
        text.push("No uncommitted changes (including untracked).".to_string());
    }
    if confirm.locked {
        text.push("This worktree is locked; confirm uses --force --force.".to_string());
    }
    if !confirm.plan.is_empty() {
        text.push(will_run_line(&confirm.plan));
    }
    text.push(String::new());
    text.push("Enter confirm · Esc cancel".to_string());
    let paragraph = Paragraph::new(text.join("\n")).block(
        Block::default()
            .borders(Borders::ALL)
            .title("destroy worktree"),
    );
    frame.render_widget(paragraph, popup);
}

fn will_run_line(plan: &TeardownPlan) -> String {
    let mut parts = Vec::new();
    if plan.compose {
        parts.push("docker compose down");
    }
    if plan.teardown.is_some() {
        parts.push("bin/teardown");
    }
    format!("Will run: {}", parts.join(", then "))
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup = Layout::default()
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
        .split(popup[1])[1]
}
