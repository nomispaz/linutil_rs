// https://github.com/Thodin/ratatui-background-process-example/blob/master/src/main.rs

use std::{
    io::{self, stderr, stdout},
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};

use ratatui::{
    self,
    crossterm::{
        event::{self, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{enable_raw_mode, EnterAlternateScreen},
    },
    prelude::{Backend, CrosstermBackend},
    Terminal,
};

// arc
use std::sync::Arc;

// include other rs-files in source-directory
mod app;
mod functions;
mod ui;
use crate::app::{App, CurrentScreen};

/// main function. wrapper for terminal setup, start of app, clearing terminal and handling errors
/// of the app
fn main() -> Result<(), io::Error> {

    // setup terminal
    enable_raw_mode()?;
    let stderr = stderr();
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create and run the app
    let mut app = App::new();

    let _result = run_app(&mut terminal, &mut app);

    // cleanup terminal
    ratatui::restore();

    Ok(())
}

/// Run the app with a generic terminal backend. Necessary so that we e.g. can use stderr instead of stdout for the terminal backend. Opens the possibility to switch crossterm with a different backend
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a channel to communicate between threads
    let (tx_output, rx_output) = mpsc::channel::<String>(); // for stdout/stderr from command to UI
    let (tx_input, rx_input) = mpsc::channel::<String>(); // for user input from UI to command stdin

    let rx_input_arc = Arc::new(Mutex::new(rx_input)); // move only once into thread

    loop {
        
        while let Ok(msg) = rx_output.try_recv() {
            app.pairs.insert(msg.clone(), msg);
        }

        let _ = terminal.draw(|frame| ui::ui(frame, app));

        if app.waiting_for_input {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char(c) => {
                        app.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
                    KeyCode::Enter => {
                        let input_to_send = format!("{}\n", app.input_buffer);
                        tx_input.send(input_to_send).unwrap();
                        app.input_buffer.clear();
                        app.waiting_for_input = false;
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            // handle key-events (non blocking)
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // Skip events that are not KeyEventKind::Press
                    if key.kind == event::KeyEventKind::Release {
                        continue;
                    }

                    // handle key events according to the current screen
                    match app.current_screen {
                        CurrentScreen::Main => match key.code {
                            // close the app
                            KeyCode::Char('q') => return Ok(()),

                            KeyCode::Char('c') => {
                                let tmp_command = format!("echo inputtest").to_string();

                                let mut command: Vec<String> = vec![tmp_command];
                                command.append(&mut vec![format!("read varname").to_string()]);
                                command.append(&mut vec![format!("echo $varname").to_string()]);
                                command.append(&mut vec![format!("echo cmd finished").to_string()]);

                                // Spawn a thread to run the Bash command asynchronously
                                // clone of command necessary to print it on error
                                let tx = tx_output.clone();
                                let rx = Arc::clone(&rx_input_arc);
                                let _command_result = thread::spawn(move || {
                                    match functions::run_command(tx, rx, command.clone()) {
                                        Ok(_) => print!(
                                            "Successfully ran command {}",
                                            command.join("; ")
                                        ),
                                        Err(e) => eprintln!(
                                            "Failed to run command {}: {:?}",
                                            command.join("; "),
                                            e
                                        ),
                                    }
                                });
                                app.waiting_for_input = true;
                            }

                            // unknown key -> do nothing
                            _ => {}
                        },
                    }
                }
            }
        }
    }
}
