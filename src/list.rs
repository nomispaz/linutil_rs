fn select_none(&mut self) {
    self.state.select(None);
}

fn select_next(&mut self) {
    self.state.select_next();
}
fn select_previous(&mut self) {
    self.state.select_previous();
}

fn select_first(&mut self) {
    self.state.select_first();
}

fn select_last(&mut self) {
    self.state.select_last();
}

// Changes the status of the selected list item
fn toggle_status(&mut self) {
    if let Some(i) = self.state.selected() {
        self.items[i].status = match self.items[i].status {
            Status::Completed => Status::Todo,
            Status::Todo => Status::Completed,
        }
    }
}
