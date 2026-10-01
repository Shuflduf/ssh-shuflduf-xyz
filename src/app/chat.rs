use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use crate::{
    app::TerminalPane,
    types::{Content, Focus, ServerState},
};

#[derive(Default)]
pub struct Chat {}

#[async_trait]
impl TerminalPane for Chat {
    async fn handle_key(
        &mut self,
        key_event: KeyEvent,
        current_pane: &mut Content,
        focus: &mut Focus,
        server_state: &ServerState,
        needs_redraw: &mut bool,
    ) {
    }
}

impl StatefulWidget for &Chat {
    type State = Focus;
    fn render(self, area: Rect, buf: &mut Buffer, focus: &mut Self::State) {}
}
