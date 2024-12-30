use crossterm::{
    event, event::DisableMouseCapture, event::Event, event::KeyCode, execute, terminal,
};
use ratatui::{backend::CrosstermBackend, Terminal as TuiTerminal};

mod ui;
use ui::{App, Modal};

fn main() -> std::io::Result<()> {
    let mut app = App::new();

    // initiate terminal
    let mut terminal = ratatui::init();
    let mut app = App::new();

    // clear terminal
    terminal.clear()?;
    // run the app
    let mut app_result = run(terminal, &mut app);

    // restore terminal before running the app
    ratatui::restore();

    loop {
        terminal.draw(|frame| app.draw(frame))?;
        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('p') => app.toggle_popup(),
                _ => continue,
            }
        }
    }

    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    Ok(())
}
