use std::io::{self, Write};

fn _main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Read credentials ---
    let mut username = String::new();
    let mut password = String::new();

    print!("Username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    print!("Password: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut password)?;
    let password = password.trim().to_string();

    // --- PAM ---
    let mut client = pam::Client::with_password("cerbere")?;
    let conv = client.conversation_mut();
    conv.set_credentials(&username, &password);

    client.authenticate()?;

    /* Hopefully, logind takes over and manage session here */
    Ok(())
}

// previous code

mod background;
mod login_panel;

type Backend = ratatui::backend::CrosstermBackend<std::io::Stdout>;
type Terminal = ratatui::Terminal<Backend>;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui_init()?;
    let res = ratatui_main(&mut terminal);
    ratatui_restore(terminal)?;
    res
}

fn ratatui_init() -> std::io::Result<Terminal> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = Backend::new(stdout);
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
    let term_size = terminal.size()?;
    let width = std::num::NonZeroUsize::new(usize::from(term_size.width))
        .ok_or_else(|| std::io::Error::other("Terminal has invalid width 0"))?;
    let height = std::num::NonZeroUsize::new(usize::from(term_size.height))
        .ok_or_else(|| std::io::Error::other("Terminal has invalid height 0"))?;

    let mut background = background::BackgroundPanel::new(width, height);
    let mut login_info = login_panel::LoginInfo::with_username("eclipse");
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(&background, area);
            frame.render_widget(&login_info, area);
        })?;
        let event = crossterm::event::read()?;
        match event {
            crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Esc,
                ..
            }) => break Ok(()),
            crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Enter,
                ..
            }) => background.regenerate(),
            crossterm::event::Event::Key(key_event) => login_info.handle_key_event(key_event),
            _ => {}
        }
    }
}
