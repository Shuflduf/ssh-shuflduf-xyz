use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::{TerminalPane, key_label, make_block},
    colours::SCHEME,
    types::{Counter, Focus, MainPane, ServerMessage, ServerState},
};

impl StatefulWidget for &Counter {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        let block = make_block(*focus == Focus::Pane)
            .title_top(key_label("2", "Counter").centered())
            .title_bottom(key_label("Enter", "Increment").centered());

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".fg(SCHEME.text),
            self.count.to_string().fg(SCHEME.accent),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[async_trait]
impl TerminalPane for Counter {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        _current_pane: &mut MainPane,
        _focus: &mut Focus,
        server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        if key_event.code == KeyCode::Enter {
            *server_state.current_value.lock().await += 1;
            let _ = server_state.broadcast_sender.send(ServerMessage::Increment);
            *needs_redraw = true;
        }
    }
}
