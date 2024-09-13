// import the local ressource file events that contains the Events-struct
mod events;
use events::Events;

use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::{Stylize, Style, Color, Modifier},
    widgets::{Paragraph, List, Block, ListDirection, ListItem},
    DefaultTerminal,
};

use std::io::{Write, BufReader, BufRead, Error, ErrorKind};
use std::process::{Command, Output, Stdio};


pub enum CurrentScreen {
    Main,
    List
}

pub struct App {
    pub current_screen: CurrentScreen
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main
        }
    }
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    //! takes a terminal and runs the app.
    
    let mut events = Events::new(vec![String::from("Item 1"), String::from("Item 2")]);
    let mut state: String = "greeting".to_string();

    let mut greeting_text = "Hello Ratatui! (press 'q' to quit)".to_string();

    loop {

        // draw the main frame
        terminal.draw(|frame| {        

            // The items managed by the application are transformed to something
            // that is understood by ratatui.
            let items: Vec<ListItem> = events
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

            if state == "greeting" {
                frame.render_widget(greeting, frame.area());
            }
            else {
                frame.render_stateful_widget(list, frame.area(), &mut events.state);
            }
        })?;
      
        // define Key events
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('l') => {
                        state = "list".to_string();
                        events.items.push("Test".to_string());
                    }
                    KeyCode::Char('g') => state = "greeting".to_string(),
                    KeyCode::Up | KeyCode::Left => events.previous(),
                    KeyCode::Down | KeyCode::Right => events.next(),
                    KeyCode::Enter => {
                        if let Some(index) = events.state.selected() {
                            if index < events.items.len() {
                                greeting_text = format!("Selected item: {}", &events.items[index]);
                                state = "greeting".to_string();
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
    let app_result = run(terminal);
    // restore terminal before running the app
    ratatui::restore();
    // return the result of the app run
    app_result
}
