use std::collections::HashMap;

pub enum CurrentScreen {
    Main,
}

pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub pairs: HashMap<String, String>,
    pub waiting_for_input: bool,
    pub input_buffer: String,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            pairs: HashMap::new(),
            waiting_for_input: false,
            input_buffer: "".to_string(),
        }
    }
}
