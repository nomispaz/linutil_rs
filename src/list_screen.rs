use ratatui::widgets::ListState;

// Let's say we have some events to display.
// https://docs.rs/ratatui/latest/ratatui/widgets/trait.StatefulWidget.html#associatedtype.State
pub struct ListWidget {
    // `items` is the state managed by your application.
    pub items: Vec<String>,
    // `state` is the state that can be modified by the UI. It stores the index of the selected
    // item as well as the offset computed during the previous draw call (used to implement
    // natural scrolling).
    pub state: ListState,
}

impl ListWidget {
    pub fn new(items: Vec<String>) -> ListWidget {
        ListWidget {
            items,
            state: ListState::default(),
        }
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        // We reset the state as the associated items have changed. This effectively reset
        // the selection as well as the stored offset.
        self.state = ListState::default();
    }

    // Select the next item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Select the previous item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Unselect the currently selected item if any. The implementation of `ListState` makes
    // sure that the stored offset is also reset.
    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
