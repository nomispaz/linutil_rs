use serde::{Deserialize, Serialize};

pub enum CurrentScreen {
    Start,
    Input,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    name: String,
    git_repo_dir: String,
}

pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub items: Vec<String>,
    pub input_buffer: String,
    pub show_password_prompt: bool,
    pub activate_input_field: bool,
    pub selected: usize,
    pub selected_item: String,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Start,
            items: Vec::new(),
            input_buffer: "".to_string(),
            show_password_prompt: false,
            activate_input_field: false,
            selected: 0,
            selected_item: "".to_string(),
        }
    }
    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.items.len();
    }

    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.items.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn select(&mut self) {
        self.selected_item = self.items[self.selected].clone();
        self.current_screen = CurrentScreen::Input;
    }

    pub fn back_to_start(&mut self) {
        self.current_screen = CurrentScreen::Start;
    }
}
