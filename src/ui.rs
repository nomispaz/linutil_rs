use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{self, App};

pub fn ui(frame: &mut Frame, app: &App) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled("Linutil", Style::default().fg(Color::Green)))
        .block(title_block);

    // render the header
    frame.render_widget(title, chunks[0]);

    match app.current_screen {
        app::CurrentScreen::Start => {
            // create a list for the main section
            let list_items: Vec<ListItem> = app
                .items
                .iter()
                .enumerate()
                .map(|(i, list_item)| {
                    let style = if i == app.selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(list_item.clone()).style(style)
                })
                .collect();

            // Create a List from all list items and highlight the currently selected one
            let list = List::new(list_items).block(
                Block::default()
                    .title("Select an item")
                    .borders(Borders::ALL),
            );

            // render the main section
            frame.render_widget(list, chunks[1]);
        }
        app::CurrentScreen::Input => {
            // create a list for the main section
            let mut list_items = Vec::<ListItem>::new();
            for entry in &app.items {
                list_items.push(ListItem::new(Line::from(Span::styled(
                    format!("{}", entry),
                    Style::default().fg(Color::Yellow),
                ))));
            }

            let list = List::new(list_items);

            // render the main section
            frame.render_widget(list, chunks[1]);
        }
        _ => {}
    }

    let input_field: Line;

    if app.activate_input_field {
        if app.show_password_prompt {
            let masked_input: String = "*".repeat(app.input_buffer.len());
            input_field = Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Green)),
                Span::raw(masked_input),
            ]);
        } else {
            input_field = Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Green)),
                Span::raw(&app.input_buffer),
            ]);
        }

        let input_line = Paragraph::new(input_field)
            .block(Block::default().borders(Borders::ALL).title("Input"))
            .style(Style::default().fg(Color::White));
        frame.render_widget(&input_line, chunks[2]);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
