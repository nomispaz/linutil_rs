// import the local ressource file events that contains the Events-struct
mod list_screen;
use list_screen::ListWidget;

mod ui;

use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::{Stylize, Style, Color, Modifier},
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Paragraph, List, Block, ListDirection, ListItem, Clear},
    DefaultTerminal,
};

use std::io::{BufReader, BufRead, Error, ErrorKind};
use std::process::{Command, Stdio, Output};

pub enum CurrentScreen {
    Main,
    List,
}

pub enum CurrentMode {
    CloneMode,
    None,
    PushMode,
    CloneExecute,
    PushExecute,
}

// contains the state of the app:
// currently used screen widget,
// current configuration of the list widget,
// infomation if the app should be restartet after closing
pub struct App {
    pub current_screen: CurrentScreen,
    pub list_screen: ListWidget,
    pub restart: bool,
    pub current_mode: CurrentMode,
    pub password_prompt: bool,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            list_screen: ListWidget::new(vec!["Clone repos".to_string(), "Push repos".to_string()]),
            restart: false,
            current_mode: CurrentMode::None,
            password_prompt: false,
        }
    }
}

fn handle_mode_selection(app: &mut App) -> Result<bool, Error> {
    let mut commands = Vec::new();

    match app.current_mode {
        CurrentMode::CloneMode => {
            commands.push("curl https://api.github.com/users/nomispaz/repos | grep full_name | cut -d':' -f 2 | cut -d'\"' -f 2 | cut -d'/' -f 2");
            let output = run_commands_stdout(&commands).expect("TODO");
            let vec_out =String::from_utf8_lossy(&output.stdout).into_owned();
            let lines: Vec<String> = vec_out.split('\n').map(|s| s.into()).collect();
            app.list_screen.set_items(lines);
            return Ok(false)
        }
        CurrentMode::PushMode => {
            commands.push("ls /home/simonheise/git_repos/");
            let output = run_commands_stdout(&commands).expect("TODO");
            let vec_out =String::from_utf8_lossy(&output.stdout).into_owned();
            let lines: Vec<String> = vec_out.split('\n').map(|s| s.into()).collect();
            app.list_screen.set_items(lines);
            return Ok(false)
        }
        CurrentMode::CloneExecute | CurrentMode::PushExecute => {
            app.restart = true;
            return Ok(true)
        }
        // handle all other possibilities
        _ => return Ok(false)
    }
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> io::Result<()> {
    //! takes a terminal and runs the app.

    loop {

        // draw the main frame
        terminal.draw(|frame| {        

            // generate text for start window
            let mut main_text = "Keyboard shortcuts:\n".to_string();
            main_text.push_str("-h: return to this page\n");
            main_text.push_str("-l: open list of possible operations\n");
            main_text.push_str("-m: return to default list\n");
            main_text.push_str("-arrow keys: select entry in list\n");
            main_text.push_str("-Enter: Perform selected action\n");
            main_text.push_str("\n");


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
   
            let main = Paragraph::new(main_text)
                .white()
                .on_black();

            let area = frame.area();

            // draw the widget corresponding to the current_screen
            match app.current_screen {
                CurrentScreen::Main => frame.render_widget(main, frame.area()),
                CurrentScreen::List => frame.render_stateful_widget(list, frame.area(), &mut app.list_screen.state)
            }

            if app.password_prompt {
                let block = Block::bordered().title("Popup");
                let area = ui::popup_area(area, 60, 20);
                frame.render_widget(Clear, area); //this clears out the background
                frame.render_widget(block, area);
            }

        })?;
      
        // define Key events
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        app.restart = false;
                        return Ok(())
                    }                    
                    KeyCode::Char('l') => app.current_screen = CurrentScreen::List,
                    KeyCode::Char('h') => app.current_screen = CurrentScreen::Main,
                    KeyCode::Char('m') => {
                        app.list_screen.set_items(vec!["Clone repos".to_string(), "Push repos".to_string()]);
                        app.current_mode = CurrentMode::None;
                    }
                    KeyCode::Char('p') => app.password_prompt = !app.password_prompt,
                    KeyCode::Up | KeyCode::Left => app.list_screen.previous(),
                    KeyCode::Down | KeyCode::Right => app.list_screen.next(),
                    KeyCode::Backspace => {
                        app.restart = true;
                        return Ok(())
                    }
                    KeyCode::Enter => {
                        match app.current_mode {
                            CurrentMode::None => {
                                let current_mode = app.list_screen.selected_item().unwrap();
                                if current_mode == "Clone repos" {
                                    app.current_mode = CurrentMode::CloneMode
                                }
                                if current_mode == "Push repos" {
                                    app.current_mode = CurrentMode::PushMode
                                }
                            }
                            CurrentMode::CloneMode => {
                                app.current_mode = CurrentMode::CloneExecute
                            }
                            CurrentMode::PushMode => {
                                app.current_mode = CurrentMode::PushExecute
                            }
                            _ => ()
                        }
                        let close_tui = handle_mode_selection(app).unwrap();
                        if close_tui {
                            return Ok(())
                        }
                    }                    
                    KeyCode::Tab => {
                        if let Some(index) = app.list_screen.state.selected() {
                            if index < app.list_screen.items.len() {
                                // read output of command, convert it to vec<str> and update the
                                // list
                                let output = run_commands_stdout(&vec!["ls -l"]).expect("TODO");
                                let vec_out =String::from_utf8_lossy(&output.stdout).into_owned();
                                let lines: Vec<String> = vec_out.split('\n').map(|s| s.into()).collect();
                                app.list_screen.set_items(lines);
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

fn run_commands(commands: Vec<&str>) {
    //! takes vector of bash commands and executes the commands
    let joined_command = commands.join("; ");
    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(joined_command)
        .spawn()
        .expect("Error: Failed to run editor")
        .wait()
        .expect("Error: Editor returned a non-zero status");
}

fn run_commands_stdout(commands: &Vec<&str>) -> Result<Output, std::io::Error> {
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

        let mut commands: Vec<&str> = Vec::new();
        let selected_item: String = app.list_screen.selected_item().unwrap().to_string();
        let mut command_prep: String = String::new();
        match app.current_mode {
            CurrentMode::CloneExecute => {
                command_prep.push_str(&format!("git clone https://github.com/nomispaz/{selected_item} /home/simonheise/git_repos/{selected_item}"));
            }
            CurrentMode::PushExecute => {
                let mut input = String::new();
                println!("\nEnter commit message:  ");
                io::stdin().read_line(&mut input)?;

                command_prep.push_str(&format!("pushd /home/simonheise/git_repos/{selected_item};"));
                command_prep.push_str(&format!("git add .;"));
                command_prep.push_str(&format!("git commit -m \"{}\";", input));
                command_prep.push_str(&format!("git push;"));
                command_prep.push_str(&format!("popd"));
            }
            _ => ()
        }
        commands.push(&command_prep);
        run_commands(commands);
       
        println!("\nHit ENTER to return to continue or type 'exit' to quit:");

        let stdin = io::stdin();
        let mut input = String::new();

        print!(">> ");
        stdin.read_line(&mut input)?;
        if input.trim() == "exit" {
            break;
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
