use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub struct App {
    pub modal: Option<Modal>,
}

impl App {
    pub fn new() -> Self {
        App { modal: None }
    }

    pub fn toggle_popup(&mut self) {
        if let Some(modal) = &self.modal {
            if !modal.is_open() {
                self.modal = Some(Modal::new(true));
            } else {
                self.modal = Some(Modal::new(false));
            }
        } else {
            self.modal = Some(Modal::new(true));
        }
    }

    pub fn draw<B: Backend>(&self, frame: &mut Frame<B>) {
        let size = frame.size();
        let block = Block::default().title("Terminal App");

        frame.render_widget(block, size);
        if let Some(modal) = &self.modal {
            modal.draw(frame);
        }
    }
}

pub struct Modal {
    open: bool,
}

impl Modal {
    pub fn new(open: bool) -> Self {
        Modal { open }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn draw<B: Backend>(&self, frame: &mut Frame<B>) {
        if self.open {
            let size = frame.size();
            let modal_block = Block::default()
                .title("Popup")
                .border_style(Style::default().fg(Color::LightCyan))
                .borders(Borders::ALL);

            let paragraph = Paragraph::new(vec![
                "This is a popup!".into(),
                "Press 'q' to quit or 'p' to toggle the popup.".into(),
            ])
            .alignment(Alignment::Center)
            .block(modal_block);

            frame.render_widget(paragraph, size);
        }
    }
}
