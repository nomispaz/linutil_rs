use std::io;

use ratatui::{self, prelude::CrosstermBackend, Terminal};
use crossterm::{self, event::EnableMouseCapture, execute, terminal::{enable_raw_mode, EnterAlternateScreen}};
use tokio;

fn main() {
    // setup terminal
    let _ = enable_raw_mode();
    let mut stderr = io::stderr();
    let _ = execute!(stderr, EnterAlternateScreen, EnableMouseCapture).unwrap();

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend);

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);


}
