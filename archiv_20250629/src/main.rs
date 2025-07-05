// https://ratatui.rs/examples/widgets/list/

use std::{
    io::{BufRead, BufReader, Read, Stdout},
    process::{Command, Stdio},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Stylize,
    style::Style,
    widgets::{Block, List, ListDirection, ListState},
    Frame,
};
use Constraint::{Fill, Length, Min};

fn handle_key(key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => should_exit = true,
        KeyCode::Char('h') | KeyCode::Left => select_none(),
        KeyCode::Char('j') | KeyCode::Down => select_next(),
        KeyCode::Char('k') | KeyCode::Up => select_previous(),
        KeyCode::Char('g') | KeyCode::Home => select_first(),
        KeyCode::Char('G') | KeyCode::End => select_last(),
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
            toggle_status();
        }
        _ => {}
    }
}

fn main() {
    let mut terminal = ratatui::init();
    loop {
        terminal.draw(draw).expect("failed to draw frame");
        if let Event::Key(key) = event::read()? {
            handle_key(key);
        };
    }
    ratatui::restore();
}

fn draw(frame: &mut Frame) {
    let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
    let [title_area, main_area, status_area] = vertical.areas(frame.area());
    let horizontal = Layout::horizontal([Fill(1); 2]);
    let [left_area, right_area] = horizontal.areas(main_area);

    let mut cmd = Command::new("bash")
        .arg("-c")
        .arg("ls -l")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut items: Vec<String> = vec![
        "Item 1".to_string(),
        "Item 2".to_string(),
        "Item 3".to_string(),
    ];

    if let Some(stdout) = cmd.stdout.take() {
        let reader = BufReader::new(stdout);
        for item in reader.lines() {
            let item = item.unwrap();
            items.push(item);
        }
    }

    let list = List::new(items)
        .block(Block::bordered().title("List"))
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    let mut state = ListState::default();
    state.select(Some(3));

    frame.render_widget(Block::bordered().title("Title Bar"), title_area);
    frame.render_widget(Block::bordered().title("Status Bar"), status_area);
    frame.render_widget(Block::bordered().title("Left"), left_area);
    frame.render_widget(Block::bordered().title("Right"), right_area);
    frame.render_stateful_widget(list, left_area, &mut state);
}
