use asthetic_clipboard::ClipboardStorage;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io};
use arboard::Clipboard;
// use chrono::{DateTime, Local};

struct App {
    storage: ClipboardStorage,
    state: ListState,
}

impl App {
    fn new() -> Result<Self, Box<dyn Error>> {
        let storage = ClipboardStorage::load().unwrap_or_default();
        let mut app = App {
            storage,
            state: ListState::default(),
        };
        if !app.storage.history.is_empty() {
            app.state.select(Some(0));
        }
        Ok(app)
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.storage.history.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.storage.history.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn copy_selected(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(i) = self.state.selected() {
            if let Some(item) = self.storage.history.get(i) {
                let mut clipboard = Clipboard::new()?;
                clipboard.set_text(&item.content)?;
            }
        }
        Ok(())
    }

    fn toggle_pin(&mut self) {
         if let Some(i) = self.state.selected() {
            if let Some(item) = self.storage.history.get_mut(i) {
                item.pinned = !item.pinned;
            }
            // Save immediately
            let _ = self.storage.save();
        }
    }

    fn delete_selected(&mut self) {
        if let Some(i) = self.state.selected() {
            if i < self.storage.history.len() {
                self.storage.history.remove(i);
                if self.storage.history.is_empty() {
                    self.state.select(None);
                } else if i >= self.storage.history.len() {
                    self.state.select(Some(self.storage.history.len() - 1));
                }
                 let _ = self.storage.save();
            }
        }
    }

    fn clear_all_unpinned(&mut self) {
        self.storage.history.retain(|i| i.pinned);
        self.state.select(if self.storage.history.is_empty() { None } else { Some(0) });
        let _ = self.storage.save();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let app = App::new()?;
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Enter => {
                    if let Err(_e) = app.copy_selected() {
                        // In TUI, maybe show error? For now print to stderr or ignore
                    }
                    return Ok(());
                }
                KeyCode::Char('p') => app.toggle_pin(),
                KeyCode::Char('d') | KeyCode::Delete => app.delete_selected(),
                KeyCode::Char('c') => app.clear_all_unpinned(),
                // Add Win+V equivalent? No, the OS handles the trigger.
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(size);

    let items: Vec<ListItem> = app
        .storage
        .history
        .iter()
        .map(|i| {
            let content = i.content.lines().next().unwrap_or("").to_string(); // Show first line only
            let pinned = if i.pinned { " [PIN]" } else { "" };
            // Format duration roughly? For now just raw time or simplified.
            let time = i.timestamp.format("%H:%M");
            let _line = format!("{} {}{}", time, content, pinned);
            let style = if i.pinned {
                 Style::default().fg(Color::Yellow)
            } else {
                 Style::default()
            };
            ListItem::new(Line::from(vec![
                 Span::styled(format!("{} ", time), Style::default().fg(Color::DarkGray)),
                 Span::styled(content, style),
                 Span::styled(pinned, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Clipboard History"))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.state);

    let help_text = "Up/Down: Navigate | Enter: Paste | p: Pin | d: Delete | c: Clear Unpinned | Esc: Quit";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(help, chunks[1]);
}
