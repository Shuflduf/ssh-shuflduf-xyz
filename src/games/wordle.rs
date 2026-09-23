use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{
    app::{TerminalPane, make_block},
    types::{Focus, MainPane, ServerState, Wordle},
};

#[async_trait]
impl TerminalPane for Wordle {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        _current_pane: &mut MainPane,
        _focus: &mut Focus,
        _server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
        let code = key_event.code;
        if (KeyCode::Char('a')..=KeyCode::Char('z')).contains(&code) {
            self.current_guess += &code.as_char().unwrap().to_string();
            *needs_redraw = true;
        } else {
            match code {
                KeyCode::Backspace | KeyCode::Delete => {
                    self.current_guess.pop();
                    *needs_redraw = true;
                }
                _ => {}
            }
        }
    }
}

impl StatefulWidget for &Wordle {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Focus) {
        let text = Line::from(self.current_guess.clone());
        Paragraph::new(text)
            .block(make_block(*focus == Focus::Pane, 2, "Wordle"))
            .render(area, buf);
    }
}
