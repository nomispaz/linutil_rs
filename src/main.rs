// https://github.com/Thodin/ratatui-background-process-example/blob/master/src/main.rs

use std::{
    fmt::format, io::{self, stderr, stdout}, time::Duration
};

use ratatui::{
    self,
    crossterm::{
        event::{self, EnableMouseCapture, Event, KeyCode, KeyModifiers},
        execute,
        terminal::{enable_raw_mode, EnterAlternateScreen},
    },
    prelude::{Backend, CrosstermBackend},
    Terminal,
};
use tokio::sync::{mpsc, Mutex};

// arc
use std::sync::Arc;

// include other rs-files in source-directory
mod app;
mod functions;
mod ui;
use crate::{
    app::{App, CurrentScreen},
    functions::run_command,
};

/// main function. wrapper for terminal setup, start of app, clearing terminal and handling errors
/// of the app
#[tokio::main]
async fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let stderr = stderr();
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create and run the app
    let mut app = App::new();
    app.items.push("Clone repo".to_string());
    app.items.push("Push repo".to_string());

    let _result = run_app(&mut terminal, &mut app).await;

    // cleanup terminal
    ratatui::restore();

    Ok(())
}

/// Run the app with a generic terminal backend. Necessary so that we e.g. can use stderr instead of stdout for the terminal backend. Opens the possibility to switch crossterm with a different backend
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a channel to communicate between threads
    let (tx_output, mut rx_output) = mpsc::channel::<String>(5); // for stdout/stderr from command to UI
    let (tx_input, rx_input) = mpsc::channel::<String>(5); // for user input from UI to command stdin

    // Arc/Mutex necessary sind the receiver needs to be moved to the async command thread in the
    // ui-loop
    let rx_input_arc = Arc::new(Mutex::new(rx_input));

    loop {
        // read messages from the async command process and update the display
        while let Ok(msg) = rx_output.try_recv() {
            app.items.push(msg.clone());
        }

        // redraw the ui
        let _ = terminal.draw(|frame| ui::ui(frame, app));

        // handle key-events (non blocking)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Skip events that are not KeyEventKind::Press
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                // Commands that should be available anywhere
                // close the app
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('q') {
                    return Ok(());
                }
                if key.code == KeyCode::Esc {
                    app.back_to_start();
                }
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    app.activate_input_field = true;
                    let tmp_command = format!("read varname").to_string();

                    let mut command: Vec<String> = vec![tmp_command];
                    command.append(&mut vec![format!("echo $varname").to_string()]);

                    // Spawn a thread to run the Bash command asynchronously
                    // clone of command necessary to print it on error
                    let tx = tx_output.clone();
                    let rx = Arc::clone(&rx_input_arc);

                    let _handle = tokio::spawn(run_command(tx, rx, command));
                }
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('s') {
                    app.show_password_prompt = true;
                    app.activate_input_field = true;
                }

                // handle key events according to the current screen
                match app.current_screen {
                    CurrentScreen::Input => {
                        // read inputs and send them via sender to the subthread
                        if key.modifiers.is_empty() && app.activate_input_field {
                            match key.code {
                                KeyCode::Char(c) => app.input_buffer.push(c),
                                KeyCode::Backspace => {
                                    app.input_buffer.pop();
                                }
                                KeyCode::Enter => {
                                    let input_to_send = format!("{}\n", app.input_buffer);
                                    tx_input.send(input_to_send).await?;
                                    app.input_buffer.clear();
                                    app.activate_input_field = false;
                                    app.show_password_prompt = false;
                                }
                                // unknown key -> do nothing
                                _ => {}
                            }
                        }
                    }
                    CurrentScreen::Start => {
                        if key.modifiers.is_empty() && !app.activate_input_field {
                            match key.code {
                                KeyCode::Down => app.next(),
                                KeyCode::Up => app.previous(),
                                KeyCode::Enter => app.select(),
                                _ => {}
                            }
                        }
                    }
                    // screen not defined
                    _ => {}
                }
            }
        }
    }
}
