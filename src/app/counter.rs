use async_trait::async_trait;
use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::{TerminalPane, make_block},
    types::{Counter, Focus, MainPane, ServerMessage, ServerState},
};

impl StatefulWidget for &Counter {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        let instructions = Line::from(vec![" [Enter]".blue().bold(), " Increment ".into()]);
        let block =
            make_block(*focus == Focus::Pane, 2, "Counter").title_bottom(instructions.centered());

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.count.to_string().yellow(),
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
        key_code: KeyCode,
        _current_pane: &mut MainPane,
        server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        if key_code == KeyCode::Enter {
            *server_state.current_value.lock().await += 1;
            let _ = server_state.broadcast_sender.send(ServerMessage::Increment);
            *needs_redraw = true;
        }
    }
}
