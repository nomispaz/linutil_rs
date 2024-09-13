// import the local ressource file events that contains the Events-struct
mod list_screen;
use list_screen::ListWidget;

use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::{Stylize, Style, Color, Modifier},
    widgets::{Paragraph, List, Block, ListDirection, ListItem},
    DefaultTerminal,
};

use std::io::{BufReader, BufRead, Error, ErrorKind};
use std::process::{Command, Stdio, Output};

pub enum CurrentScreen {
    Main,
    List
}

// contains the state of the app:
// currently used screen widget,
// current configuration of the list widget,
// infomation if the app should be restartet after closing
pub struct App {
    pub current_screen: CurrentScreen,
    pub list_screen: ListWidget,
    pub restart: bool
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            list_screen: ListWidget::new(vec![String::from("Item 1"), String::from("Item 2")]),
            restart: false
        }
    }
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> io::Result<()> {
    //! takes a terminal and runs the app.

    let mut greeting_text = "Hello Ratatui! (press 'q' to quit)".to_string();

    loop {

        // draw the main frame
        terminal.draw(|frame| {        

            // The items managed by the application are transformed to something
            // that is understood by ratatui.
            let items: Vec<ListItem> = app.list_screen
                .items
                .iter()
                .map(|i| ListItem::new(i.as_str()))
                .collect();        

            let list = List::new(items)
                .block(Block::bordered().title("List"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);
   
            let greeting = Paragraph::new(greeting_text.clone())
                .white()
                .on_blue();

            // draw the widget corresponding to the current_screen
            match app.current_screen {
                CurrentScreen::Main => frame.render_widget(greeting, frame.area()),
                CurrentScreen::List => frame.render_stateful_widget(list, frame.area(), &mut app.list_screen.state)
            }
        })?;
      
        // define Key events
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        app.restart = false;
                        return Ok(())
                    }                    KeyCode::Char('l') => {
                        app.current_screen = CurrentScreen::List;
                        app.list_screen.items.push("Test".to_string());
                    }
                    KeyCode::Char('g') => app.current_screen = CurrentScreen::Main,
                    KeyCode::Up | KeyCode::Left => app.list_screen.previous(),
                    KeyCode::Down | KeyCode::Right => app.list_screen.next(),
                    KeyCode::Backspace => {
                        app.restart = true;
                        return Ok(())
                    }
                    KeyCode::Enter => {
                        if let Some(index) = app.list_screen.state.selected() {
                            if index < app.list_screen.items.len() {
                                // read output of command, convert it to vec<str> and update the
                                // list
                                let output = run_commands(vec!["ls -l"]).expect("Test");
                                let vec_out =String::from_utf8_lossy(&output.stdout).into_owned();
                                let lines: Vec<String> = vec_out.split('\n').map(|s| s.into()).collect();
                                app.list_screen.set_items(lines)
                            }
                        }

                    }
                    // catch all other key presses
                    _ => ()
                }
            }
        }
    }
}

fn run_commands(commands: Vec<&str>) -> Result<Output, std::io::Error> {
    //! takes vector of bash commands and executes the commands
    let joined_command = commands.join("; ");
    let mut cmd = Command::new("bash");
    cmd.arg("-c")
       .arg(joined_command)
       .output()
}

fn run_commands_stdio(commands: Vec<&str>) -> Result<(), Error> {
    //! takes vektor or bash commands and executes the commands while reading stdio as buffer and
    //! prints stdout in realtime
    let joined_command = commands.join("; ");

    let cmd = Command::new("bash")
        .arg("-c")
        .arg(joined_command)
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let reader = BufReader::new(cmd);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));

    Ok(())
}

fn main() -> io::Result<()> {
    // initiate terminal
    let mut terminal = ratatui::init();
    let mut app = App::new();

    // clear terminal
    terminal.clear()?;
    // run the app
    let mut app_result = run(terminal, &mut app);

    // restore terminal before running the app
    ratatui::restore();

    while app.restart {

        println!("Enter something (type 'exit' to quit):");

        let stdin = io::stdin();
        loop {
            let mut input = String::new();

            print!("> ");

            stdin.read_line(&mut input)?;

            if input.trim() == "exit" {
                break;
            }

            println!("You said: {}", input.trim());
        }

        terminal = ratatui::init();
        // clear terminal
        terminal.clear()?;

        app_result = run(terminal, &mut app);

    // restore terminal before running the app
    ratatui::restore();
    }
    // return the result of the app run
    app_result
}
