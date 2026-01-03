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
    let mut login_info = login_panel::LoginInfo::with_username("eclipse");
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(&login_info, area);
        })?;
        let event = crossterm::event::read()?;
        match event {
            #[cfg(debug_assertions)]
            crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Esc,
                ..
            }) => break Ok(()),
            crossterm::event::Event::Key(key_event) => login_info.handle_key_event(key_event),
            _ => {}
        }
    }
}
