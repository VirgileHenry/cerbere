mod login_panel;

type Backend = ratatui::backend::CrosstermBackend<std::io::Stdout>;
type Terminal = ratatui::Terminal<Backend>;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::io::{self, Stdout, Write};

fn main() -> io::Result<()> {
    let mut terminal = ratatui_init()?;
    let res = ratatui_main(&mut terminal);
    ratatui_restore(terminal)?;
    res
}

fn ratatui_init() -> std::io::Result<Terminal> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    Ok(terminal)
}

fn ratatui_restore(mut terminal: Terminal) -> std::io::Result<()> {
    terminal.show_cursor()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn ratatui_main(terminal: &mut Terminal) -> std::io::Result<()> {
    let mut login_info = login_panel::LoginInfo::with_username("eclipse");
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(&login_info, area);
        })?;
        let event = crossterm::event::read()?;
        match event {
            crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Esc,
                ..
            }) => break Ok(()),
            crossterm::event::Event::Key(key_event) => login_info.handle_key_event(key_event)?,
            _ => {}
        }
    }
}

fn run_app(terminal: &mut Terminal) -> io::Result<()> {
    let mut username = String::new();
    let mut password = String::new();
    let mut focus = Focus::Username;

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Centered popup
            let area = centered_rect(40, 7, size);

            let block = Block::default().title(" Login ").borders(Borders::ALL);

            f.render_widget(&block, area);

            let inner = block.inner(area);

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
                .split(inner);

            let user_style = if focus == Focus::Username {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let pass_style = if focus == Focus::Password {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let user = Paragraph::new(format!("Username: {}", username))
                .style(user_style)
                .alignment(Alignment::Left);

            let masked = "*".repeat(password.len());
            let pass = Paragraph::new(format!("Password: {}", masked))
                .style(pass_style)
                .alignment(Alignment::Left);

            let hint = Paragraph::new("Enter = submit · Tab = switch · Esc = quit").alignment(Alignment::Center);

            f.render_widget(user, rows[0]);
            f.render_widget(pass, rows[1]);
            f.render_widget(hint, rows[2]);
        })?;

        // ---- input handling ----
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => break,

                KeyCode::Tab => {
                    focus = match focus {
                        Focus::Username => Focus::Password,
                        Focus::Password => Focus::Username,
                    }
                }

                KeyCode::Enter => {
                    // placeholder for PAM auth
                    break;
                }

                KeyCode::Backspace => match focus {
                    Focus::Username => {
                        username.pop();
                    }
                    Focus::Password => {
                        password.pop();
                    }
                },

                KeyCode::Char(c) => match focus {
                    Focus::Username => username.push(c),
                    Focus::Password => password.push(c),
                },

                _ => {}
            }
        }
    }

    Ok(())
}

#[derive(Copy, Clone, PartialEq)]
enum Focus {
    Username,
    Password,
}

// ---- helper: centered rectangle ----
fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn login() -> Result<(), Box<dyn std::error::Error>> {
    // Ask for username
    print!("Username:");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();

    print!("Password:");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    Ok(())
}
